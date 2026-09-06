#!/usr/bin/env python3
"""Parse and edit X-Men Legends II - XMLB binary UI menu format.

Format discovered:
  Header (24 bytes):
    - 4 bytes: magic (0x000011B1)
    - 4 bytes: version (1)
    - 4 bytes: string_table_offset (file offset where strings begin)
    - 4 bytes: unknown (0xFFFFFFFF)
    - 4 bytes: string_count
    - 4 bytes: unknown2

  Node array: starts at 0x18, each node is 32 bytes (8 uint32 string offsets)
    - offset[0]: widget type string offset (e.g., "item", "animtext", "mark")
    - offset[1]: widget name string offset (e.g., "fullscreen", "label_options")
    - offsets[2..7]: 3 property pairs (key_offset, value)

    Property values are either:
      - String offsets (>= string_table_offset, points into string table)
      - Raw uint32 values (coordinates, flags, counts, etc.)
      - 0xFFFFFFFF (null / not set)

  String table: null-terminated ASCII strings at string_table_offset
"""

import struct
import os
import copy

MAGIC = 0x000011B1
HEADER_SIZE = 24
NODE_SIZE = 32
NODE_COUNT_DWORD = 16  # offset of str_count in header


def read_xmlb(path):
    with open(path, 'rb') as f:
        return bytearray(f.read())


def write_xmlb(path, data):
    with open(path, 'wb') as f:
        f.write(data)


def parse_header(data):
    """Parse the 24-byte header."""
    magic = struct.unpack_from('<I', data, 0)[0]
    version = struct.unpack_from('<I', data, 4)[0]
    strtab_off = struct.unpack_from('<I', data, 8)[0]
    unk1 = struct.unpack_from('<I', data, 12)[0]
    str_count = struct.unpack_from('<I', data, 16)[0]
    unk2 = struct.unpack_from('<I', data, 20)[0]
    return {
        'magic': magic,
        'version': version,
        'strtab_off': strtab_off,
        'unk1': unk1,
        'str_count': str_count,
        'unk2': unk2,
    }


def build_string_table(data, strtab_off, str_count):
    """Build offset->string map from the string table."""
    strings = {}
    pos = strtab_off
    for i in range(str_count):
        if pos >= len(data):
            break
        if data[pos] == 0:
            pos += 1
            continue
        end = pos
        while end < len(data) and data[end] != 0:
            end += 1
        s = bytes(data[pos:end]).decode('latin-1', errors='replace')
        strings[pos] = s
        pos = end + 1
    return strings


def is_string_offset(val, strtab_off):
    """Check if a uint32 value is a string table offset."""
    return val >= strtab_off and val < len(data)


def parse_nodes(data, strtab_off, str_count, strings):
    """Parse all nodes from the node array (starts at 0x18)."""
    nodes = []
    for n in range(str_count):
        base = 0x18 + n * NODE_SIZE
        if base + NODE_SIZE > len(data):
            break

        offsets = []
        for i in range(8):
            off = struct.unpack_from('<I', data, base + i * 4)[0]
            offsets.append(off)

        # Offsets 0 and 1 are type and name
        type_off = offsets[0]
        name_off = offsets[1]

        node = {
            'index': n,
            'file_offset': base,
            'type': strings.get(type_off, '<null>' if type_off == 0xFFFFFFFF else None),
            'type_off': type_off,
            'name': strings.get(name_off, '<null>' if name_off == 0xFFFFFFFF else None),
            'name_off': name_off,
            'props': {},
        }

        # Offsets 2-7 are 3 property pairs (key, value)
        for i in range(3):
            key_off = offsets[2 + i * 2]
            val = offsets[3 + i * 2]

            if key_off == 0xFFFFFFFF:
                continue

            key = strings.get(key_off, '<0x%04X>' % key_off) if key_off in strings else None
            if key is None and key_off == 0:
                key = '<empty>'

            # Check if value is a string offset or raw uint32
            if val == 0xFFFFFFFF:
                val_str = '<null>'
                val_raw = None
            elif val in strings:
                val_str = strings[val]
                val_raw = None
            else:
                val_str = str(val)
                val_raw = val

            node['props'][key] = {
                'string': val_str,
                'raw': val_raw,
            }

        nodes.append(node)

    return nodes


def node_to_bytes(node_data, strings_rev):
    """Serialize a node back to bytes."""
    result = bytearray()
    for offset in node_data:
        result.extend(struct.pack('<I', offset))
    return result


def get_string_offset(strings_rev, s):
    """Get the file offset for a string, adding it if needed."""
    if s in strings_rev:
        return strings_rev[s]
    return None


def xmlb_to_strings(data):
    """Extract all strings from the string table."""
    hdr = parse_header(data)
    strings = build_string_table(data, hdr['strtab_off'], hdr['str_count'])
    return strings, hdr


def print_xmlb(path):
    data = read_xmlb(path)
    hdr = parse_header(data)
    strings = build_string_table(data, hdr['strtab_off'], hdr['str_count'])
    nodes = parse_nodes(data, hdr['strtab_off'], hdr['str_count'], strings)

    print(f"File: {path}")
    print(f"Size: {len(data)} bytes")
    print(f"Magic: 0x{hdr['magic']:08X}")
    print(f"Version: {hdr['version']}")
    print(f"String table: 0x{hdr['strtab_off']:04X}")
    print(f"String count: {hdr['str_count']}")
    print(f"Unknown1: 0x{hdr['unk1']:08X}")
    print(f"Unknown2: 0x{hdr['unk2']:08X}")
    print()

    print(f"Strings ({len(strings)}):")
    for off in sorted(strings.keys()):
        print(f"  0x{off:04X}: {strings[off]!r}")
    print()

    print(f"Nodes ({len(nodes)}):")
    for node in nodes:
        print(f"  Node {node['index']:3d}: type={node['type'] or '?':<20s} name={node['name'] or '?':<20s}")
        for k, v in node['props'].items():
            print(f"    {k} = {v['string']}")


if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print("Usage: xmlb_parser.py [--dump <file>] [--scale <file> <factor>]")
        sys.exit(1)

    if sys.argv[1] == '--dump':
        print_xmlb(sys.argv[2])
    elif sys.argv[1] == '--scale':
        path = sys.argv[2]
        factor = float(sys.argv[3])
        data = read_xmlb(path)
        hdr = parse_header(data)
        strings = build_string_table(data, hdr['strtab_off'], hdr['str_count'])
        nodes = parse_nodes(data, hdr['strtab_off'], hdr['str_count'], strings)
        print("Scaling %d nodes by factor %.2f" % (len(nodes), factor))
        print("Not fully implemented yet - need to identify coordinate fields")
    else:
        print("Unknown command")
