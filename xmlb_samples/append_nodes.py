import struct

def append_graphics_settings(input_path, output_path, new_nodes):
    """Append new nodes to XMLB file without rebuilding string table"""
    with open(input_path, 'rb') as f:
        data = bytearray(f.read())
    
    # Parse header
    magic = struct.unpack('<I', data[0:4])[0]
    version = struct.unpack('<I', data[4:8])[0]
    strtab_off = struct.unpack('<I', data[8:12])[0]
    unk1 = struct.unpack('<I', data[12:16])[0]
    str_count = struct.unpack('<I', data[16:20])[0]
    unk2 = struct.unpack('<I', data[20:24])[0]
    
    print('Original: %d nodes, strtab at 0x%s, size=%d' % (str_count, hex(strtab_off), len(data)))
    
    # Verify str_count matches actual nodes
    node_size = 32
    nodes_start = 24
    nodes_end = nodes_start + str_count * node_size
    print('Nodes at: 0x%s - 0x%s' % (hex(nodes_start), hex(nodes_end)))
    
    # New nodes will be inserted at nodes_end
    new_node_count = len(new_nodes)
    new_str_count = str_count + new_node_count
    
    # We need to add the new nodes BEFORE the string table
    # The string table is at strtab_off (5136), nodes end at 3608
    # So there's a gap between 3608 and 5136 = 1528 bytes
    # We can insert nodes in this gap!
    
    gap_size = strtab_off - nodes_end
    print('Gap before string table: %d bytes' % gap_size)
    print('Can fit %d new nodes (%d bytes)' % (gap_size // node_size, (gap_size // node_size) * node_size))
    
    if gap_size >= new_node_count * node_size:
        print('Enough space in gap!')
        # Insert new nodes at nodes_end
        insert_pos = nodes_end
        
        # Shift string table and everything after it
        shift = new_node_count * node_size
        new_strtab_off = strtab_off + shift
        
        # Extend data
        data.extend(b'\x00' * shift)
        
        # Move string table forward
        old_strtab = data[strtab_off:strtab_off + len(data) - shift - strtab_off]
        # Actually easier: shift everything from strtab_off to end
        for i in range(len(data) - shift - 1, strtab_off - 1, -1):
            data[i + shift] = data[i]
        
        # Write new nodes at insert_pos
        pos = insert_pos
        for node in new_nodes:
            # Type offset (use 0xFFFFFFFF for now - need actual string offset)
            struct.pack_into('<I', data, pos, 0xFFFFFFFF); pos += 4
            # Name offset
            struct.pack_into('<I', data, pos, 0xFFFFFFFF); pos += 4
            # Props (3 pairs of key,value)
            for prop in node['props'][:3]:
                key_off = prop.get('key_offset', 0xFFFFFFFF)
                val = prop.get('value_offset', 0xFFFFFFFF)
                struct.pack_into('<I', data, pos, key_off); pos += 4
                struct.pack_into('<I', data, pos, val); pos += 4
            # Pad to 32 bytes if needed
            if pos < nodes_end + (new_node_count * node_size):
                pos += (nodes_end + (new_node_count * node_size)) - pos
        
        # Update header
        struct.pack_into('<I', data, 8, new_strtab_off)
        struct.pack_into('<I', data, 16, new_str_count)
        
        print('New strtab_off: 0x%s' % hex(new_strtab_off))
        print('New str_count: %d' % new_str_count)
        print('New file size: %d' % len(data))
        
        with open(output_path, 'wb') as f:
            f.write(data)
        print('Saved to %s' % output_path)
    else:
        print('Not enough space in gap!')

# Define new graphics settings nodes with proper string offsets
# For now, we'll use existing string offsets or add strings
new_nodes = [
    {
        'props': [
            {'key_offset': 0xFFFFFFFF, 'value_offset': 0xFFFFFFFF},
            {'key_offset': 0xFFFFFFFF, 'value_offset': 0xFFFFFFFF},
            {'key_offset': 0xFFFFFFFF, 'value_offset': 0xFFFFFFFF},
        ]
    }
]

# Test with one node first
append_graphics_settings(
    'D:/re-lab-share/xmen2_mod/xmlb_samples/options.XMLB',
    'D:/re-lab-share/xmen2_mod/xmlb_samples/options_test.XMLB',
    new_nodes
)