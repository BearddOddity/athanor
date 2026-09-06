#!/usr/bin/env python3
"""Parse and edit X-Men Legends II XMLB UI menu files.

The XMLB format is a binary widget tree used by the Athanor engine.
Structure:
  Header (20 bytes):
    - 4 bytes: magic (0x000011B1)
    - 4 bytes: version (1)
    - 4 bytes: string table offset
    - 4 bytes: unknown (0xFFFFFFFF)
    - 4 bytes: string count
  String table: null-terminated ASCII strings
  Node array: array of variable-length nodes

Each node appears to be:
  - 4 bytes: name offset (into string table)
  - 4 bytes: x position
  - 4 bytes: y position
  - 4 bytes: width
  - 4 bytes: height
  - 4 bytes: type/flags
  - 4 bytes: property count
  - N * 8 bytes: property pairs (key_offset, value)
"""

import struct
import sys
import os

MAGIC = 0x000011B1

def read_xmlb(path):
    with open(path, 'rb') as f:
        return bytearray(f.read())

def get_string(data, offset):
    """Read null-terminated string at offset."""
    end = data.find(b'\x00', offset)
    if end == -1:
        return ""
    return data[offset:end].decode('ascii', errors='replace')

def parse_header(data):
    """Parse XMLB header."""
    magic = struct.unpack_from('<I', data, 0)[0]
    version = struct.unpack_from('<I', data, 4)[0]
    strtab_off = struct.unpack_from('<I', data, 8)[0]
    unknown = struct.unpack_from('<I', data, 12)[0]
    str_count = struct.unpack_from('<I', data, 16)[0]
    return {
        'magic': magic,
        'version': version,
        'strtab_off': strtab_off,
        'unknown': unknown,
        'str_count': str_count
    }

def parse_string_table(data, offset, count):
    """Parse string table."""
    strings = {}
    pos = offset
    for i in range(count):
        s = get_string(data, pos)
        strings[pos] = s
        pos += len(s) + 1
    return strings

def parse_nodes(data, start_offset, strings):
    """Parse node array."""
    nodes = []
    pos = start_offset
    
    while pos < len(data):
        # Check if we've reached the end
        if pos + 4 > len(data):
            break
        
        # Read first dword - could be name offset or end marker
        first = struct.unpack_from('<I', data, pos)[0]
        
        # If first is 0xFFFFFFFF, we might be at the end
        if first == 0xFFFFFFFF:
            # Check if this is a node with -1 parent or end marker
            if pos + 8 <= len(data):
                second = struct.unpack_from('<I', data, pos+4)[0]
                if second == 0xFFFFFFFF:
                    break  # Two -1s in a row = end
        
        # Try to parse as node
        name_off = first
        if name_off not in strings:
            # Not a valid string offset, might be end of nodes
            break
        
        name = strings[name_off]
        
        # Read node properties
        x = struct.unpack_from('<I', data, pos+4)[0]
        y = struct.unpack_from('<I', data, pos+8)[0]
        w = struct.unpack_from('<I', data, pos+12)[0]
        h = struct.unpack_from('<I', data, pos+16)[0]
        type_flags = struct.unpack_from('<I', data, pos+20)[0]
        prop_count = struct.unpack_from('<I', data, pos+24)[0]
        
        node = {
            'name': name,
            'name_off': name_off,
            'x': x,
            'y': y,
            'w': w,
            'h': h,
            'type_flags': type_flags,
            'prop_count': prop_count,
            'props': {},
            'offset': pos
        }
        
        pos += 28  # Skip header
        
        # Read properties
        for i in range(prop_count):
            if pos + 8 > len(data):
                break
            key_off = struct.unpack_from('<I', data, pos)[0]
            val = struct.unpack_from('<I', data, pos+4)[0]
            
            key = strings.get(key_off, f"0x{key_off:08X}")
            if val in strings:
                node['props'][key] = strings[val]
            else:
                node['props'][key] = val
            
            pos += 8
        
        nodes.append(node)
        
        # Safety check
        if len(nodes) > 1000:
            break
    
    return nodes

def main():
    if len(sys.argv) < 2:
        print("Usage: parse_xmlb.py <file.xmlb>")
        sys.exit(1)
    
    path = sys.argv[1]
    data = read_xmlb(path)
    
    print(f"File: {path}")
    print(f"Size: {len(data)} bytes")
    print()
    
    hdr = parse_header(data)
    print(f"Header:")
    print(f"  Magic: 0x{hdr['magic']:08X}")
    print(f"  Version: {hdr['version']}")
    print(f"  String table: 0x{hdr['strtab_off']:08X}")
    print(f"  Unknown: 0x{hdr['unknown']:08X}")
    print(f"  String count: {hdr['str_count']}")
    print()
    
    strings = parse_string_table(data, hdr['strtab_off'], hdr['str_count'])
    print(f"String table: {len(strings)} strings")
    for off, s in sorted(strings.items()):
        print(f"  0x{off:04X}: '{s}'")
    print()
    
    # Find node array start (after string table)
    strtab_end = max(strings.keys()) + len(strings[max(strings.keys())]) + 1
    print(f"String table ends at: 0x{strtab_end:04X}")
    print()
    
    nodes = parse_nodes(data, strtab_end, strings)
    print(f"Nodes found: {len(nodes)}")
    for i, node in enumerate(nodes):
        print(f"  Node {i}: '{node['name']}' at ({node['x']},{node['y']}) {node['w']}x{node['h']} type={node['type_flags']}")
        for k, v in node['props'].items():
            print(f"    {k} = {v}")

if __name__ == '__main__':
    main()