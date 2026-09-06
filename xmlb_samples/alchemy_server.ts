/**
 * Athanor - Bun HTTP Server
 * Unified API for all Athanor engine formats
 */

import { existsSync, statSync, readFileSync, writeFileSync, readdirSync } from "fs";
import { join, basename } from "path";

const SHARED_DIR = process.env.XMEN2_SHARE || "D:/re-lab-share/xmen2_mod";
const GAME_DIR = process.env.XMEN2_GAME || "D:/My Games/X-Men Legends II Rise of Apocalypse";
const XMLB_DIR = join(SHARED_DIR, "xmlb_samples");

const MAGIC_XMLB = 0x000011B1;
const HEADER_SIZE = 24;
const NODE_SIZE = 32;

// ============================================================
// Format Parsers (ported from Python)
// ============================================================

function readUint32(data: Uint8Array, offset: number): number {
    return data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
}

function decodeString(data: Uint8Array, offset: number): string | null {
    if (offset >= data.length) return null;
    if (data[offset] === 0) return null;
    let end = offset;
    while (end < data.length && data[end] !== 0) end++;
    if (end <= offset) return null;
    return new TextDecoder("iso-8859-1").decode(data.slice(offset, end));
}

function buildStringTable(data: Uint8Array, strtabOff: number, strCount: number): Map<number, string> {
    const strings = new Map<number, string>();
    let pos = strtabOff;
    
    // Parse declared strings
    for (let i = 0; i < strCount; i++) {
        if (pos >= data.length) break;
        if (data[pos] === 0) { pos++; continue; }
        let end = pos;
        while (end < data.length && data[end] !== 0) end++;
        if (end > pos) {
            const s = new TextDecoder("iso-8859-1").decode(data.slice(pos, end));
            strings.set(pos, s);
        }
        pos = end + 1;
    }
    
    // Scan for additional strings
    while (pos < data.length - 4) {
        if (data[pos] === 0) { pos++; continue; }
        let end = pos;
        while (end < data.length && data[end] !== 0) end++;
        if (end > pos) {
            const s = new TextDecoder("iso-8859-1").decode(data.slice(pos, end));
            if (s.length >= 3) {
                let found = false;
                for (const [, v] of strings) if (v === s) { found = true; break; }
                if (!found) strings.set(pos, s);
            }
        }
        pos = end + 1;
    }
    
    return strings;
}

function parseXMLBLike(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const version = readUint32(data, 4);
    const strtabOff = readUint32(data, 8);
    const unk1 = readUint32(data, 12);
    const strCount = readUint32(data, 16);
    const unk2 = readUint32(data, 20);
    
    const strings = buildStringTable(data, strtabOff, strCount);
    
    // Parse nodes
    const maxNodes = Math.min(strCount, Math.floor((data.length - HEADER_SIZE) / NODE_SIZE));
    const nodes = [];
    
    for (let n = 0; n < maxNodes; n++) {
        const nodeStart = HEADER_SIZE + n * NODE_SIZE;
        const offsets: number[] = [];
        for (let i = 0; i < 8; i++) {
            offsets.push(readUint32(data, nodeStart + i * 4));
        }
        
        // Skip empty nodes
        if (offsets.every(o => o === 0xFFFFFFFF)) continue;
        
        const typeOff = offsets[0];
        const nameOff = offsets[1];
        
        let typeStr = strings.get(typeOff);
        if (typeOff === 0xFFFFFFFF) typeStr = null;
        else if (!typeStr) typeStr = `<0x${typeOff.toString(16).padStart(4, '0')}>`;
        
        let nameStr = strings.get(nameOff);
        if (nameOff === 0xFFFFFFFF) nameStr = null;
        else if (!nameStr) nameStr = `<0x${nameOff.toString(16).padStart(4, '0')}>`;
        
        const props = [];
        for (let i = 0; i < 3; i++) {
            const keyOff = offsets[2 + i * 2];
            const val = offsets[3 + i * 2];
            if (keyOff === 0xFFFFFFFF) continue;
            
            let key = strings.get(keyOff);
            if (!key) key = `<0x${keyOff.toString(16).padStart(4, '0')}>`;
            
            let valStr: string | null = null;
            let valRaw: number | null = null;
            
            if (val === 0xFFFFFFFF) {
                valStr = null;
            } else if (strings.has(val)) {
                valStr = strings.get(val)!;
            } else {
                valStr = val.toString();
                valRaw = val;
            }
            
            props.push({ key, value: valStr ?? valRaw, isRaw: valRaw !== null });
        }
        
        nodes.push({
            index: n,
            fileOffset: nodeStart,
            type: typeStr,
            name: nameStr,
            props
        });
    }
    
    return {
        filename,
        format: 'XMLB',
        size: data.length,
        magic,
        version,
        strtabOff,
        strCount: strings.size,
        nodes,
        strings: Object.fromEntries(strings)
    };
}

function parseBNX(data: Uint8Array, filename: string): any {
    const content = new TextDecoder("iso-8859-1").decode(data);
    const lines = content.trim().split('\n');
    const kvPairs = [];
    
    for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed.length > 0 && trimmed.includes('=') && !trimmed.startsWith('[')) {
            const [key, value] = trimmed.split('=', 2);
            kvPairs.push({ key: key.trim(), value: value.trim() });
        }
    }
    
    const strings: Record<string, string> = {};
    for (let i = 0; i < kvPairs.length; i++) {
        strings[i.toString()] = `${kvPairs[i].key}=${kvPairs[i].value}`;
    }
    
    return {
        filename,
        format: 'BNX',
        size: data.length,
        magic: 0,
        version: 0,
        strtabOff: 0,
        strCount: kvPairs.length,
        nodes: [],
        strings
    };
}

function parseIGB(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const strings: Record<string, string> = {};
    
    for (let i = 0; i < data.length - 4; i++) {
        if (data[i] >= 0x20 && data[i] < 0x7F) {
            let s = '';
            let j = i;
            while (j < data.length && data[j] >= 0x20 && data[j] < 0x7F && s.length < 64) {
                s += String.fromCharCode(data[j]);
                j++;
            }
            if (s.length >= 3 && !Object.values(strings).includes(s)) {
                strings[i.toString()] = s;
            }
            i = j + 1;
        }
    }
    
    return {
        filename,
        format: 'IGB',
        size: data.length,
        magic,
        version: 0,
        strtabOff: 8,
        strCount: Object.keys(strings).length,
        nodes: [],
        strings
    };
}

function parseZSM(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const strings: Record<string, string> = {};
    
    for (let i = 0; i < data.length - 4; i++) {
        if (data[i] >= 0x20 && data[i] < 0x7F) {
            let s = '';
            let j = i;
            while (j < data.length && data[j] >= 0x20 && data[j] < 0x7F && s.length < 64) {
                s += String.fromCharCode(data[j]);
                j++;
            }
            if (s.length >= 3 && !Object.values(strings).includes(s)) {
                strings[i.toString()] = s;
            }
            i = j + 1;
        }
    }
    
    return {
        filename,
        format: data[4] === 0x58 ? 'ZSS' : 'ZSM', // Check for 'XY,' signature
        size: data.length,
        magic,
        version: 0,
        strtabOff: 8,
        strCount: Object.keys(strings).length,
        nodes: [],
        strings
    };
}

function parseZAM(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const entryCount = magic;
    const strings: Record<string, string> = {};
    
    return {
        filename,
        format: 'ZAM',
        size: data.length,
        magic,
        version: 0,
        strtabOff: 0,
        strCount: 0,
        nodes: [],
        strings
    };
}

function detectFormat(filename: string, data: Uint8Array): string {
    const ext = filename.toLowerCase().split('.').pop() || '';
    
    if (data.length >= 4) {
        const magic = readUint32(data, 0);
        if (magic === MAGIC_XMLB) return 'XMLB';
        if (data[0] === 0x5A && data[1] === 0x53 && data[2] === 0x4E && data[3] === 0x44) return 'ZSM';
    }
    
    if (data[0] === 0xA4 && data[1] === 0x02) return 'IGB';
    
    try {
        const text = new TextDecoder("iso-8859-1").decode(data.slice(0, 200));
        if (text.includes('=') && text.trim().length > 0) return 'BNX';
    } catch {}
    
    if (ext === 'zam') return 'ZAM';
    if (ext === 'anm') return 'ANIM';
    if (ext === 'phys') return 'PHYS';
    if (ext === 'bus') return 'AUD';
    if (ext === 'comp') return 'COMP';
    if (ext === 'pbr') return 'PBR';
    if (ext === 'plgn') return 'PLGN';
    if (ext === 'save') return 'SAVE';
    if (ext === 'pipe') return 'PIPE';
    
    const formatMap: Record<string, string> = {
        'xmlb': 'XMLB', 'pkgb': 'XMLB', 'engb': 'XMLB',
        'chrb': 'XMLB', 'navb': 'XMLB', 'boyb': 'XMLB',
        'zsm': 'ZSM', 'zss': 'ZSS', 'bnx': 'BNX', 'igb': 'IGB',
        'anm': 'ANIM', 'phys': 'PHYS', 'bus': 'AUD', 'comp': 'COMP',
        'pbr': 'PBR', 'plgn': 'PLGN', 'save': 'SAVE', 'pipe': 'PIPE'
    };
    
    return formatMap[ext] || 'UNKNOWN';
}

// ============================================================
// New Format Parsers (Athanor)
// ============================================================

function parseANIM(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const version = readUint32(data, 4);
    const strtabOff = readUint32(data, 8);
    
    const stateCount = readUint32(data, 24);
    const transitionCount = readUint32(data, 28);
    
    const states = [];
    for (let i = 0; i < stateCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            states.push({
                index: i,
                nameOffset: readUint32(data, pos),
                clipOffset: readUint32(data, pos + 4),
                loop: readUint32(data, pos + 8) === 1,
                speed: readFloat32(data, pos + 12),
            });
        }
    }
    
    return {
        filename, format: 'ANIM', size: data.length, magic, version,
        stateCount, transitionCount, states, strings: extractStrings(data, strtabOff)
    };
}

function parsePHYS(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const version = readUint32(data, 4);
    const shapeCount = readUint32(data, 24);
    
    const shapes = [];
    const typeNames = ['box', 'sphere', 'capsule', 'convex_hull', 'triangle_mesh'];
    
    for (let i = 0; i < shapeCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            shapes.push({
                index: i,
                type: typeNames[readUint32(data, pos)] || 'unknown',
                mass: readFloat32(data, pos + 4),
                isTrigger: readUint32(data, pos + 8) === 1,
            });
        }
    }
    
    return {
        filename, format: 'PHYS', size: data.length, magic, version,
        shapeCount, shapes
    };
}

function parseAUD(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const busCount = readUint32(data, 24);
    const effectCount = readUint32(data, 28);
    
    const buses = [];
    for (let i = 0; i < busCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            buses.push({
                index: i,
                nameOffset: readUint32(data, pos),
                volume: readFloat32(data, pos + 4),
                parentOffset: readUint32(data, pos + 8),
                effectCount: readUint32(data, pos + 12),
            });
        }
    }
    
    return {
        filename, format: 'AUD', size: data.length, magic, version: 1,
        busCount, buses
    };
}

function parseCOMP(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const effectCount = readUint32(data, 24);
    
    const effects = [];
    for (let i = 0; i < effectCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            effects.push({
                index: i,
                nameOffset: readUint32(data, pos),
                enabled: readUint32(data, pos + 4) === 1,
                paramCount: readUint32(data, pos + 8),
            });
        }
    }
    
    return {
        filename, format: 'COMP', size: data.length, magic, version: 1,
        effectCount, effects
    };
}

function parsePBR(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const materialCount = readUint32(data, 24);
    
    const materials = [];
    for (let i = 0; i < materialCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            materials.push({
                index: i,
                nameOffset: readUint32(data, pos),
                baseColor: [readFloat32(data, pos + 4), readFloat32(data, pos + 8), readFloat32(data, pos + 12), readFloat32(data, pos + 16)],
                metallic: readFloat32(data, pos + 20),
                roughness: readFloat32(data, pos + 24),
            });
        }
    }
    
    return {
        filename, format: 'PBR', size: data.length, magic, version: 1,
        materialCount, materials
    };
}

function parsePLGN(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const pluginCount = readUint32(data, 24);
    
    const plugins = [];
    for (let i = 0; i < pluginCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            plugins.push({
                index: i,
                nameOffset: readUint32(data, pos),
                versionOffset: readUint32(data, pos + 4),
                authorOffset: readUint32(data, pos + 8),
                entryOffset: readUint32(data, pos + 12),
                enabled: readUint32(data, pos + 16) === 1,
            });
        }
    }
    
    return {
        filename, format: 'PLGN', size: data.length, magic, version: 1,
        pluginCount, plugins
    };
}

function parseSAVE(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const sectionCount = readUint32(data, 24);
    
    const sections = [];
    for (let i = 0; i < sectionCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            sections.push({
                index: i,
                nameOffset: readUint32(data, pos),
                version: readUint32(data, pos + 4),
                compressed: readUint32(data, pos + 8) === 1,
                size: readUint32(data, pos + 12),
            });
        }
    }
    
    return {
        filename, format: 'SAVE', size: data.length, magic, version: 1,
        sectionCount, sections
    };
}

function parsePIPE(data: Uint8Array, filename: string): any {
    const magic = readUint32(data, 0);
    const importerCount = readUint32(data, 24);
    
    const importers = [];
    for (let i = 0; i < importerCount; i++) {
        const pos = 32 + i * 32;
        if (pos + 32 <= data.length) {
            importers.push({
                index: i,
                extOffset: readUint32(data, pos),
                processorOffset: readUint32(data, pos + 4),
            });
        }
    }
    
    return {
        filename, format: 'PIPE', size: data.length, magic, version: 1,
        importerCount, importers
    };
}

function readFloat32(data: Uint8Array, offset: number): number {
    const buf = Buffer.alloc(4);
    buf.writeUInt32LE(data[offset] | (data[offset+1] << 8) | (data[offset+2] << 16) | (data[offset+3] << 24), 0);
    return buf.readFloatLE(0);
}

function extractStrings(data: Uint8Array, strtabOff: number): Record<string, string> {
    const strings: Record<string, string> = {};
    let pos = strtabOff;
    while (pos < data.length) {
        if (data[pos] === 0) { pos++; continue; }
        let end = pos;
        while (end < data.length && data[end] !== 0) end++;
        if (end > pos) {
            const s = Buffer.from(data.slice(pos, end)).toString('latin1');
            strings[pos.toString()] = s;
        }
        pos = end + 1;
    }
    return strings;
}

function parseFormat(filename: string, data: Uint8Array): any {
    const fmt = detectFormat(filename, data);
    
    switch (fmt) {
        case 'XMLB': return parseXMLBLike(data, filename);
        case 'BNX': return parseBNX(data, filename);
        case 'IGB': return parseIGB(data, filename);
        case 'ZSM': return parseZSM(data, filename);
        case 'ZSS': return parseZSM(data, filename);
        case 'ZAM': return parseZAM(data, filename);
        case 'ANIM': return parseANIM(data, filename);
        case 'PHYS': return parsePHYS(data, filename);
        case 'AUD': return parseAUD(data, filename);
        case 'COMP': return parseCOMP(data, filename);
        case 'PBR': return parsePBR(data, filename);
        case 'PLGN': return parsePLGN(data, filename);
        case 'SAVE': return parseSAVE(data, filename);
        case 'PIPE': return parsePIPE(data, filename);
        default:
            throw new Error(`Unknown format: ${fmt}`);
    }
}

// ============================================================
// XMLB Builder
// ============================================================

function buildXMLBLike(data: any): Uint8Array {
    const headerSize = 24;
    const nodeSize = 32;
    
    // Estimate initial sizes
    const stringTableStart = headerSize + (data.nodes.length * nodeSize);
    let stringTableSize = 0;
    
    // First pass: collect all strings and calculate total string table size
    const stringsToAdd: { offset: number; value: string }[] = [];
    
    // Add all strings from data.strings
    for (const [offsetStr, value] of Object.entries(data.strings)) {
        const offset = parseInt(offsetStr);
        if (!isNaN(offset)) {
            stringsToAdd.push({ offset, value });
            stringTableSize += Buffer.byteLength(value, 'latin1') + 1; // +1 for null terminator
        }
    }
    
    // Add strings from nodes that might not be in data.strings (if modified)
    // For simplicity, we'll rebuild everything from nodes
    const allStrings: string[] = [];
    for (const node of data.nodes) {
        if (node.type && !allStrings.includes(node.type)) allStrings.push(node.type);
        if (node.name && !allStrings.includes(node.name)) allStrings.push(node.name);
        for (const prop of node.props || []) {
            if (typeof prop.value === 'string' && !allStrings.includes(prop.value)) {
                allStrings.push(prop.value);
            }
        }
    }
    
    // Rebuild string table from scratch
    stringsToAdd.length = 0;
    stringTableSize = 0;
    for (const str of allStrings) {
        stringsToAdd.push({ offset: stringTableSize, value: str });
        stringTableSize += Buffer.byteLength(str, 'iso-8859-1') + 1; // +1 for null terminator
    }
    
    // Calculate buffer sizes
    const stringTableOffset = headerSize + (data.nodes.length * nodeSize);
    
    // Create string lookup map for offset->string (we need string->offset for building)
    const stringToOffset: Map<string, number> = new Map();
    let currentOffset = 0;
    for (const { value } of stringsToAdd) {
        stringToOffset.set(value, stringTableOffset + currentOffset);
        currentOffset += Buffer.byteLength(value, 'latin1') + 1;
    }
    
    const totalSize = stringTableOffset + stringTableSize;
    
    // Create buffer
    const buffer = Buffer.alloc(totalSize);
    let pos = 0;
    
    // Write header
    buffer.writeUInt32LE(MAGIC_XMLB, pos); pos += 4; // magic
    buffer.writeUInt32LE(data.version || 1, pos); pos += 4; // version
    buffer.writeUInt32LE(stringTableOffset, pos); pos += 4; // strtab_off
    buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4; // unk1
    buffer.writeUInt32LE(allStrings.length, pos); pos += 4; // str_count
    buffer.writeUInt32LE(0, pos); pos += 4; // unk2
    
    // Write nodes
    for (const node of data.nodes) {
        const nodeStart = pos;
        
        // Type offset
        const typeOffset = node.type ? stringToOffset.get(node.type) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        buffer.writeUInt32LE(typeOffset, pos); pos += 4;
        
        // Name offset
        const nameOffset = node.name ? stringToOffset.get(node.name) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        buffer.writeUInt32LE(nameOffset, pos); pos += 4;
        
        // Properties (3 key-value pairs)
        const nodeProps = node.props || [];
        for (let i = 0; i < 3; i++) {
            if (i < nodeProps.length) {
                const prop = nodeProps[i];
                const keyOffset = prop.key ? stringToOffset.get(prop.key) ?? 0xFFFFFFFF : 0xFFFFFFFF;
                buffer.writeUInt32LE(keyOffset, pos); pos += 4;
                
                let valueOffset: number;
                if (prop.value === null) {
                    valueOffset = 0xFFFFFFFF;
                } else if (typeof prop.value === 'string') {
                    valueOffset = stringToOffset.get(prop.value) ?? 0xFFFFFFFF;
                } else {
                    // Raw numeric value
                    valueOffset = prop.value;
                }
                buffer.writeUInt32LE(valueOffset, pos); pos += 4;
            } else {
                // Empty property slots
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
            }
        }
        
        // Pad to NODE_SIZE if needed (should already be 32 bytes = 8*4)
        const nodeEnd = pos;
        if (nodeEnd - nodeStart < nodeSize) {
            pos = nodeStart + nodeSize;
        }
    }
    
    // Write string table
    pos = stringTableOffset;
    for (const { value } of stringsToAdd) {
        buffer.write(value, pos, 'latin1');
        pos += Buffer.byteLength(value, 'latin1');
        buffer.writeUInt8(0, pos); // null terminator
        pos += 1;
    }
    
    return buffer;
}

// ============================================================
// File System Utilities
// ============================================================

function findAllGameFiles(): any[] {
    const results: any[] = [];
    
    function walk(dir: string, relDir: string = '') {
        if (!existsSync(dir)) return;
        const entries = readdirSync(dir);
        for (const entry of entries) {
            const full = join(dir, entry);
            const stat = statSync(full);
            if (stat.isDirectory()) {
                walk(full, join(relDir, entry));
            } else if (stat.isFile()) {
                const ext = entry.toLowerCase().split('.').pop() || '';
                if (['xmlb', 'pkgb', 'engb', 'chrb', 'navb', 'boyb', 'bnx', 'igb', 'zsm', 'zss', 'zam',
        'anm', 'phys', 'bus', 'comp', 'pbr', 'plgn', 'save', 'pipe'].includes(ext)) {
                    results.push({
                        path: full,
                        relPath: join(relDir, entry),
                        name: entry,
                        size: stat.size,
                        ext: '.' + ext
                    });
                }
            }
        }
    }
    
    const gameDir = GAME_DIR;
    walk(gameDir);
    return results;
}

// ============================================================
// Bun Server
// ============================================================

const server = Bun.serve({
    port: 3457,
    development: true,
    async fetch(req) {
        const url = new URL(req.url);
        const path = url.pathname;
        const method = req.method;
        
        const corsHeaders = {
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Allow-Headers": "Content-Type",
        };
        
        if (method === "OPTIONS") {
            return new Response(null, { status: 204, headers: corsHeaders });
        }
        
        try {
            // API: List all game files
            if (path === "/api/files" && method === "GET") {
                const files = findAllGameFiles();
                return Response.json({ files, count: files.length }, { headers: corsHeaders });
            }
            
            // API: List files by extension
            const listMatch = path.match(/^\/api\/files\/(.+)$/);
            if (listMatch && method === "GET") {
                const ext = listMatch[1];
                const files = findAllGameFiles().filter(f => f.ext === '.' + ext);
                return Response.json({ files, count: files.length }, { headers: corsHeaders });
            }
            
            // API: Parse file
            const parseMatch = path.match(/^\/api\/parse\/(.+)$/);
            if (parseMatch && method === "GET") {
                let relPath = decodeURIComponent(parseMatch[1]);
                // Normalize path separators
                relPath = relPath.replace(/\//g, '\\');
                
                const fullPath = join(GAME_DIR, relPath);
                
                if (!existsSync(fullPath)) {
                    return Response.json({ error: "File not found: " + fullPath }, { status: 404, headers: corsHeaders });
                }
                
                const data = new Uint8Array(readFileSync(fullPath));
                const result = parseFormat(relPath, data);
                return Response.json(result, { headers: corsHeaders });
            }
            
            // Serve editor UI
            if (path === "/editor" || path === "/") {
                try {
                    const editorHtml = await Bun.file('alchemy_editor_v2.html').text();
                    return new Response(editorHtml, {
                        headers: { ...corsHeaders, "Content-Type": "text/html" }
                    });
                } catch {
                    return Response.json({ error: "Editor not found" }, { status: 404, headers: corsHeaders });
                }
            }
            
            // API: Save modified XMLB - inserts new_nodes into the gap between existing nodes and string table
            if (path === "/api/save" && method === "POST") {
                try {
                    const body = await req.json() as any;
                    const { relPath, newNodes } = body;
                    const fullPath = join(GAME_DIR, relPath);
                    
                    if (!existsSync(fullPath)) {
                        return Response.json({ error: "File not found" }, { status: 404, headers: corsHeaders });
                    }
                    
                    // Read original binary
                    const origBuffer = readFileSync(fullPath);
                    const origData = new Uint8Array(origBuffer);
                    const strtabOff = readUint32(origData, 8);
                    const strCount = readUint32(origData, 16);
                    
                    // Calculate positions
                    const NODE_SIZE = 32;
                    const nodesEnd = 24 + strCount * NODE_SIZE;
                    const gapSize = strtabOff - nodesEnd;
                    
                    if (!newNodes || newNodes.length === 0) {
                        return Response.json({ error: "No newNodes provided" }, { status: 400, headers: corsHeaders });
                    }
                    
                    // Check if we have space in the gap to insert new nodes
                    const newNodeCount = newNodes.length;
                    
                    if (gapSize >= newNodeCount * NODE_SIZE) {
                        // Parse original to get string offsets
                        const parsed = parseFormat(relPath, origData);
                        const stringOffsets: Record<string, number> = {};
                        
                        // Collect string offsets from parsed result
                        if (parsed.strings) {
                            for (const [offStr, val] of Object.entries(parsed.strings)) {
                                const off = parseInt(offStr);
                                if (!isNaN(off)) {
                                    stringOffsets[val] = off;
                                }
                            }
                        }
                        
                        // Collect new strings from new nodes
                        const newStrings: string[] = [];
                        for (const node of newNodes) {
                            if (node.type && !newStrings.includes(node.type)) newStrings.push(node.type);
                            if (node.name && !newStrings.includes(node.name)) newStrings.push(node.name);
                            for (const prop of node.props || []) {
                                if (typeof prop.value === 'string' && !newStrings.includes(prop.value)) {
                                    newStrings.push(prop.value);
                                }
                                if (prop.key && !newStrings.includes(prop.key)) {
                                    newStrings.push(prop.key);
                                }
                            }
                        }
                        
                        // Assign offsets to new strings (append after original string table)
                        const origStrTabSize = origBuffer.length - strtabOff;
                        let nextOffset = origBuffer.length;
                        for (const s of newStrings) {
                            stringOffsets[s] = nextOffset;
                            nextOffset += Buffer.byteLength(s, 'latin1') + 1;
                        }
                        
                        // Calculate new total size
                        const newStrTabSize = origStrTabSize + newStrings.reduce(
                            (sum: number, s: string) => sum + Buffer.byteLength(s, 'latin1') + 1, 0
                        );
                        const newTotalSize = origBuffer.length + newStrTabSize;
                        
// Extend the buffer
                const newBuffer = Buffer.alloc(newTotalSize);
                newBuffer.set(origBuffer, 0); // copy original
                        
                        // Write new nodes in the gap
                        for (let i = 0; i < newNodeCount; i++) {
                            const node = newNodes[i];
                            const nodePos = nodesEnd + i * NODE_SIZE;
                            
                            newBuffer.writeUInt32LE(stringOffsets[node.type] ?? 0xFFFFFFFF, nodePos);
                            newBuffer.writeUInt32LE(stringOffsets[node.name] ?? 0xFFFFFFFF, nodePos + 4);
                            
                            const props = node.props || [];
                            for (let j = 0; j < 3; j++) {
                                if (j < props.length) {
                                    const prop = props[j];
                                    const keyOff = stringOffsets[prop.key] ?? 0xFFFFFFFF;
                                    const val = typeof prop.value === 'string' 
                                        ? stringOffsets[prop.value] 
                                        : (prop.value ?? 0xFFFFFFFF);
                                    newBuffer.writeUInt32LE(keyOff, nodePos + 8 + j * 8);
                                    newBuffer.writeUInt32LE(val, nodePos + 12 + j * 8);
                                } else {
                                    newBuffer.writeUInt32LE(0xFFFFFFFF, nodePos + 8 + j * 8);
                                    newBuffer.writeUInt32LE(0xFFFFFFFF, nodePos + 12 + j * 8);
                                }
                            }
                        }
                        
                        // Append new strings after original string table
                        let strtabPos = origBuffer.length;
                        for (const s of newStrings) {
                            newBuffer.write(s, strtabPos, 'latin1');
                            strtabPos += Buffer.byteLength(s, 'latin1');
                            newBuffer[strtabPos] = 0; // null terminator
                            strtabPos += 1;
                        }
                        
                        // Update header: increase str_count
                        newBuffer.writeUInt32LE(strCount + newNodeCount, 16);
                        
                        writeFileSync(fullPath, newBuffer);
                        return Response.json({ 
                            ok: true, 
                            path: fullPath, 
                            size: newTotalSize,
                            nodes: newNodeCount 
                        }, { headers: corsHeaders });
                    } else {
                        return Response.json({ 
                            error: "Not enough space in gap (" + gapSize + " < " + newNodeCount * NODE_SIZE + ")",
                            gapSize: gapSize,
                            needed: newNodeCount * NODE_SIZE 
                        }, { status: 400, headers: corsHeaders });
                    }
                } catch (e: any) {
                    console.error("Save error:", e);
                    return Response.json({ error: e.message || String(e) }, { status: 500, headers: corsHeaders });
                }
            }
            
            // API: Format info
            if (path === "/api/formats" && method === "GET") {
                return Response.json({
                    formats: [
                        { ext: '.xmlb', name: 'UI Binary Format', desc: 'Menu/UI layouts, settings' },
                        { ext: '.pkgb', name: 'Package Format', desc: 'Asset packages, textures' },
                        { ext: '.engb', name: 'Engine Format', desc: 'Conversation/scripts' },
                        { ext: '.chrb', name: 'Character Format', desc: 'Character definitions' },
                        { ext: '.navb', name: 'Navigation Format', desc: 'Pathfinding/navmesh' },
                        { ext: '.boyb', name: 'Boy Format', desc: 'Buoy/waypoint data' },
                        { ext: '.bnx', name: 'Binary Text', desc: 'Config/options key=value' },
                        { ext: '.igb', name: 'Image Format', desc: 'HUD/texture images' },
                        { ext: '.zsm', name: 'Sound ZSM', desc: 'Sound metadata/index' },
                        { ext: '.zss', name: 'Sound ZSS', desc: 'Sound data streams' },
                        { ext: '.zam', name: 'Automap Format', desc: 'Minimap/automap data' }
                    ]
                }, { headers: corsHeaders });
            }
            
            // Serve editor UI
            if (path === "/" || path === "/editor" || path === "/alchemy_editor.html") {
                const htmlPath = join(XMLB_DIR, "alchemy_editor.html");
                if (existsSync(htmlPath)) {
                    return new Response(readFileSync(htmlPath), { 
                        headers: { ...corsHeaders, "Content-Type": "text/html" } 
                    });
                }
                return new Response("Editor UI not found", { status: 404 });
            }
            
            return Response.json({ error: "Not found", path }, { status: 404, headers: corsHeaders });
            
        } catch (e: any) {
            console.error("Server error:", e);
            return Response.json({ error: e.message || String(e) }, { status: 500, headers: corsHeaders });
        }
    },
});

console.log(`Athanor server running at http://localhost:${server.port}`);
console.log(`Game directory: ${GAME_DIR}`);
console.log(`Editor UI: http://localhost:${server.port}/editor`);