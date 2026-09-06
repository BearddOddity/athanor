import { Buffer } from 'buffer';

const MAGIC_XMLB = 0x000011B1;
const HEADER_SIZE = 24;
const NODE_SIZE = 32;

function buildXMLBLike(data: any): Uint8Array {
    const headerSize = 24;
    const nodeSize = 32;
    
    // Collect strings from nodes
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
    
    // Build string table
    const stringsToAdd: { offset: number; value: string }[] = [];
    let stringTableSize = 0;
    for (const str of allStrings) {
        stringsToAdd.push({ offset: stringTableSize, value: str });
        stringTableSize += Buffer.byteLength(str, 'iso-8859-1') + 1;
    }
    
    // Create string lookup
    const stringToOffset = new Map();
    let currentOffset = 0;
    for (const { value } of stringsToAdd) {
        stringToOffset.set(value, currentOffset);
        currentOffset += Buffer.byteLength(value, 'iso-8859-1') + 1;
    }
    
    const stringTableOffset = headerSize + (data.nodes.length * nodeSize);
    const totalSize = stringTableOffset + stringTableSize;
    
    console.log('nodes:', data.nodes.length);
    console.log('stringTableOffset:', stringTableOffset);
    console.log('stringTableSize:', stringTableSize);
    console.log('totalSize:', totalSize);
    console.log('strings:', allStrings.length);
    
    const buffer = Buffer.alloc(totalSize);
    let pos = 0;
    
    buffer.writeUInt32LE(MAGIC_XMLB, pos); pos += 4;
    buffer.writeUInt32LE(data.version || 1, pos); pos += 4;
    buffer.writeUInt32LE(stringTableOffset, pos); pos += 4;
    buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
    buffer.writeUInt32LE(allStrings.length, pos); pos += 4;
    buffer.writeUInt32LE(0, pos); pos += 4;
    
    for (const node of data.nodes) {
        const typeOffset = node.type ? stringToOffset.get(node.type) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        const nameOffset = node.name ? stringToOffset.get(node.name) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        
        buffer.writeUInt32LE(typeOffset, pos); pos += 4;
        buffer.writeUInt32LE(nameOffset, pos); pos += 4;
        
        for (let i = 0; i < 3; i++) {
            if (i < (node.props || []).length) {
                const prop = node.props[i];
                const keyOffset = prop.key ? stringToOffset.get(prop.key) ?? 0xFFFFFFFF : 0xFFFFFFFF;
                buffer.writeUInt32LE(keyOffset, pos); pos += 4;
                
                let valueOffset;
                if (prop.value === null) {
                    valueOffset = 0xFFFFFFFF;
                } else if (typeof prop.value === 'string') {
                    valueOffset = stringToOffset.get(prop.value) ?? 0xFFFFFFFF;
                } else {
                    valueOffset = prop.value;
                }
                buffer.writeUInt32LE(valueOffset, pos); pos += 4;
            } else {
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
            }
        }
    }
    
    pos = stringTableOffset;
    for (const { value } of stringsToAdd) {
        const strBuf = Buffer.from(value, 'iso-8859-1');
        buffer.write(strBuf, pos);
        pos += strBuf.length;
        buffer.writeUInt8(0, pos);
        pos += 1;
    }
    
    return buffer;
}

const data = {
    nodes: [
        { index: 0, type: 'OPTIONS_MENU', name: 'options', props: [{ key: 'mark', value: 200, isRaw: true }] },
        { index: 1, type: 'MENU_ITEM_MODEL', name: 'track01', props: [{ key: 'item', value: 552, isRaw: true }] }
    ]
};

try {
    const buf = buildXMLBLike(data);
    console.log('SUCCESS! Buffer size:', buf.length);
} catch (e: any) {
    console.error('ERROR:', e.message);
}