import struct, os

def patch_xmlb(input_path, output_path, new_nodes):
    """Patch XMLB by inserting new nodes in the gap and appending strings."""
    with open(input_path, 'rb') as f:
        data = bytearray(f.read())
    
    # Parse header
    magic = struct.unpack('<I', data[0:4])[0]
    version = struct.unpack('<I', data[4:8])[0]
    strtab_off = struct.unpack('<I', data[8:12])[0]
    unk1 = struct.unpack('<I', data[12:16])[0]
    str_count = struct.unpack('<I', data[16:20])[0]
    unk2 = struct.unpack('<I', data[20:24])[0]
    
    print('Original: %d nodes, strtab=0x%x, size=%d' % (str_count, strtab_off, len(data)))
    
    NODE_SIZE = 32
    nodes_end = 24 + str_count * NODE_SIZE
    gap_start = nodes_end
    gap_end = strtab_off
    gap_size = gap_end - gap_start
    
    print('Nodes end: 0x%x, gap: %d bytes, can fit %d nodes' % (gap_start, gap_size, gap_size // NODE_SIZE))
    
    if gap_size < len(new_nodes) * NODE_SIZE:
        print('ERROR: Not enough space in gap')
        return None
    
    # Build string table
    strings = []
    string_offsets = {}
    pos = strtab_off
    while pos < len(data):
        if data[pos] == 0:
            pos += 1
            continue
        end = pos
        while end < len(data) and data[end] != 0:
            end += 1
        s = data[pos:end].decode('latin-1')
        if s:
            strings.append(s)
            string_offsets[s] = pos
        pos = end + 1
    
    # New strings from nodes - collect all keys and string values
    all_new_strings = set()
    for node in new_nodes:
        if node['type']: all_new_strings.add(node['type'])
        if node['name']: all_new_strings.add(node['name'])
        for prop in node.get('props', []):
            if prop.get('key'): all_new_strings.add(prop['key'])
            val = prop.get('value', 0xFFFFFFFF)
            if isinstance(val, str) and val != '0xFFFFFFFF':
                all_new_strings.add(val)
    
    # Add to string table
    for s in all_new_strings:
        if s not in string_offsets:
            string_offsets[s] = strtab_off + sum(len(x) + 1 for x in strings)
            strings.append(s)
    
    # New string table size
    new_strtab_size = sum(len(s) + 1 for s in strings)
    old_strtab_size = len(data) - strtab_off
    strtab_delta = new_strtab_size - old_strtab_size
    
    print('String table: old=%d, new=%d, delta=%d' % (old_strtab_size, new_strtab_size, strtab_delta))
    
    # Total new size
    new_node_count = len(new_nodes)
    new_str_count = str_count + new_node_count
    new_size = len(data) + strtab_delta
    new_strtab_off = strtab_off  # strtab position doesn't change, we just extend it
    
    # Extend data
    data.extend(b'\x00' * strtab_delta)
    
# Write new nodes in the gap
    node_pos = gap_start
    for idx, node in enumerate(new_nodes):
        # Type offset
        type_off = string_offsets.get(node['type'], 0xFFFFFFFF)
        struct.pack_into('<I', data, node_pos, type_off); node_pos += 4
        # Name offset
        name_off = string_offsets.get(node['name'], 0xFFFFFFFF)
        struct.pack_into('<I', data, node_pos, name_off); node_pos += 4
        # Props
        for prop in node['props'][:3]:
            key_off = string_offsets.get(prop['key'], 0xFFFFFFFF)
            val = prop.get('value', 0xFFFFFFFF)
            if isinstance(val, str):
                val = string_offsets.get(val, 0xFFFFFFFF)
            struct.pack_into('<I', data, node_pos, key_off); node_pos += 4
            struct.pack_into('<I', data, node_pos, val); node_pos += 4
        # Pad to NODE_SIZE
        node_end = gap_start + (idx + 1) * NODE_SIZE
        while node_pos < node_end:
            struct.pack_into('<I', data, node_pos, 0xFFFFFFFF); node_pos += 4
    
    # Rebuild string table
    strtab_pos = strtab_off
    for s in strings:
        data[strtab_pos:strtab_pos + len(s)] = s.encode('latin-1')
        strtab_pos += len(s)
        data[strtab_pos] = 0
        strtab_pos += 1
    
    # Update header
    struct.pack_into('<I', data, 16, new_str_count)
    
    print('New: %d nodes, strtab=0x%x, size=%d' % (new_str_count, new_strtab_off, len(data)))
    
    with open(output_path, 'wb') as f:
        f.write(data)
    print('Saved to %s' % output_path)
    return True

# Define new graphics settings nodes
new_nodes = [
    {
        'type': 'OPTIONS_MENU',
        'name': 'graphics',
        'props': [
            {'key': 'mark', 'value': 3600},
            {'key': 'item', 'value': 3600},
            {'key': 'style', 'value': 'STYLE_TITLE_MED'},
        ]
    },
    {
        'type': 'OPTIONS_MENU',
        'name': 'resolution',
        'props': [
            {'key': 'mark', 'value': 3640},
            {'key': 'item', 'value': 3640},
            {'key': 'text', 'value': 'Resolution'},
        ]
    },
    {
        'type': 'OPTIONS_MENU',
        'name': 'fsaa',
        'props': [
            {'key': 'mark', 'value': 3680},
            {'key': 'item', 'value': 3680},
            {'key': 'text', 'value': 'FSAA'},
        ]
    },
    {
        'type': 'OPTIONS_MENU',
        'name': 'shadow_quality',
        'props': [
            {'key': 'mark', 'value': 3720},
            {'key': 'item', 'value': 3720},
            {'key': 'text', 'value': 'Shadow Quality'},
        ]
    },
    {
        'type': 'OPTIONS_MENU',
        'name': 'texture_quality',
        'props': [
            {'key': 'mark', 'value': 3760},
            {'key': 'item', 'value': 3760},
            {'key': 'text', 'value': 'Texture Quality'},
        ]
    },
    {
        'type': 'OPTIONS_MENU',
        'name': 'view_distance',
        'props': [
            {'key': 'mark', 'value': 3800},
            {'key': 'item', 'value': 3800},
            {'key': 'text', 'value': 'View Distance'},
        ]
    },
]

result = patch_xmlb(
    'D:/re-lab-share/xmen2_mod/xmlb_samples/options.XMLB',
    'D:/re-lab-share/xmen2_mod/xmlb_samples/options_patched.XMLB',
    new_nodes
)

if result:
    print('SUCCESS!')
else:
    print('FAILED!')