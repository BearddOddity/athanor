import { Buffer } from 'buffer';

const MAGIC_XMLB = 0x000011B1;
const HEADER_SIZE = 24;
const NODE_SIZE = 32;

function buildXMLBLike(data: any): Uint8Array {
    const allStrings: string[] = [];
    for (const node of data.nodes) {
        if (node.type && typeof node.type === 'string' && !allStrings.includes(node.type)) {
            allStrings.push(node.type);
        }
        if (node.name && typeof node.name === 'string' && !allStrings.includes(node.name)) {
            allStrings.push(node.name);
        }
        for (const prop of node.props || []) {
            if (typeof prop.value === 'string' && !allStrings.includes(prop.value)) {
                allStrings.push(prop.value);
            }
        }
    }
    
    console.log('Strings collected:', allStrings.slice(0, 10));
    console.log('Total strings:', allStrings.length);
    
    const stringToOffset = new Map();
    let currentOffset = 0;
    for (const str of allStrings) {
        stringToOffset.set(str, currentOffset);
        currentOffset += Buffer.byteLength(str, 'latin1') + 1;
    }
    
    console.log('String offsets:');
    for (const [str, off] of stringToOffset) {
        console.log('  "%s" -> 0x%s' % [str, off.toString(16).padStart(4, '0')]);
        if (stringToOffset.size > 20) break;
    }
    
    const stringTableOffset = HEADER_SIZE + (data.nodes.length * NODE_SIZE);
    let stringTableSize = 0;
    for (const str of allStrings) {
        stringTableSize += Buffer.byteLength(str, 'latin1') + 1;
    }
    const totalSize = stringTableOffset + stringTableSize;
    
    console.log('stringTableOffset:', stringTableOffset);
    console.log('stringTableSize:', stringTableSize);
    console.log('totalSize:', totalSize);
    
    const buffer = Buffer.alloc(totalSize);
    let pos = 0;
    
    buffer.writeUInt32LE(MAGIC_XMLB, pos); pos += 4;
    buffer.writeUInt32LE(data.version || 1, pos); pos += 4;
    buffer.writeUInt32LE(stringTableOffset, pos); pos += 4;
    buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
    buffer.writeUInt32LE(allStrings.length, pos); pos += 4;
    buffer.writeUInt32LE(0, pos); pos += 4;
    
    for (const node of data.nodes) {
        const nodeStart = pos;
        
        const typeOffset = node.type ? stringToOffset.get(node.type) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        const nameOffset = node.name ? stringToOffset.get(node.name) ?? 0xFFFFFFFF : 0xFFFFFFFF;
        
        buffer.writeUInt32LE(typeOffset, pos); pos += 4;
        buffer.writeUInt32LE(nameOffset, pos); pos += 4;
        
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
                    valueOffset = prop.value;
                }
                buffer.writeUInt32LE(valueOffset, pos); pos += 4;
            } else {
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
                buffer.writeUInt32LE(0xFFFFFFFF, pos); pos += 4;
            }
        }
        
        const nodeEnd = pos;
        if (nodeEnd - nodeStart < NODE_SIZE) {
            pos = nodeStart + NODE_SIZE;
        }
    }
    
    pos = stringTableOffset;
    for (const str of allStrings) {
        buffer.write(str, pos, 'latin1');
        pos += Buffer.byteLength(str, 'latin1');
        buffer.writeUInt8(0, pos);
        pos += 1;
    }
    
    return buffer;
}

const data = {
    version: 1,
    nodes: [
        { index: 0, type: 'animonclose', name: 'true', props: [
            { key: 'animonopen', value: 'true' },
            { key: 'desctext1', value: '$MENU_BACK Back' },
            { key: 'desctext2', value: '$MENU_ACCEPT Change' }
        ]},
        { index: 1, type: 'fadein', name: 'false', props: [
            { key: 'fadeout', value: 'true' },
            { key: 'fullscreen', value: 'false' },
            { key: 'igb', value: 'x2m_options' }
        ]}
    ]
};

const buf = buildXMLBLike(data);
console.log('SUCCESS! Buffer size:', buf.length);
console.log('First 24 bytes (header):');
for (let i = 0; i < 24; i += 4) {
    console.log('  0x%s: 0x%s' % [i.toString(16).padStart(2, '0'), buf.readUInt32LE(i).toString(16).padStart(8, '0')]);
}
console.log('Node 0 (bytes 24-55):');
for (let i = 24; i < 56; i += 4) {
    const off = buf.readUInt32LE(i);
    console.log('  0x%s: 0x%s -> %s' % [i.toString(16).padStart(2, '0'), off.toString(16).padStart(8, '0'), off === 0xFFFFFFFF ? 'NULL' : buf.toString('latin1', stringTableOffset(off))]);
}

function stringTableOffset(off: number): number {
    const strtabOff = buf.readUInt32LE(8);
    return strtabOff + off;
}