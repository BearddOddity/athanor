#!/usr/bin/env python3
"""XMLB format core parser and serializer for X-Men Legends II UI menus."""

import struct
import os

MAGIC = 0x000011B1
HEADER_SIZE = 24
NODE_SIZE = 32
MAX_STRING_OFFSET = 0x1000000


class XMLBNode:
    """Represents a single widget node in an XMLB file."""
    def __init__(self, index, file_offset, type_str, name_str, props):
        self.index = index
        self.file_offset = file_offset
        self.type = type_str
        self.name = name_str
        self.props = props  # List of (key, value) tuples

    def __repr__(self):
        return f"Node({self.index}: {self.type} '{self.name}')"


class XMLBFile:
    """Represents a parsed XMLB file."""

    def __init__(self):
        self.data = bytearray()
        self.path = ""
        self.magic = 0
        self.version = 0
        self.strtab_off = 0
        self.unk1 = 0
        self.str_count = 0
        self.unk2 = 0
        self.strings = {}  # offset -> string
        self.strings_rev = {}  # string -> offset
        self.nodes = []
        self.modified = False

    @classmethod
    def load(cls, path):
        """Load and parse an XMLB file."""
        with open(path, 'rb') as f:
            data = bytearray(f.read())

        xmlb = cls()
        xmlb.path = path
        xmlb.data = data

        # Parse header
        xmlb.magic = struct.unpack_from('<I', data, 0)[0]
        xmlb.version = struct.unpack_from('<I', data, 4)[0]
        xmlb.strtab_off = struct.unpack_from('<I', data, 8)[0]
        xmlb.unk1 = struct.unpack_from('<I', data, 12)[0]
        xmlb.str_count = struct.unpack_from('<I', data, 16)[0]
        xmlb.unk2 = struct.unpack_from('<I', data, 20)[0]

        # Validate magic
        if xmlb.magic != MAGIC:
            raise ValueError(f"Invalid magic: 0x{xmlb.magic:08X} (expected 0x{MAGIC:08X})")

        # Build string table
        xmlb._build_string_table()

        # Parse nodes
        xmlb._parse_nodes()

        return xmlb

    def _build_string_table(self):
        """Parse the string table into a map."""
        pos = self.strtab_off
        for i in range(self.str_count):
            if pos >= len(self.data):
                break
            if self.data[pos] == 0:
                pos += 1
                continue
            end = pos
            while end < len(self.data) and self.data[end] != 0:
                end += 1
            s = bytes(self.data[pos:end]).decode('latin-1', errors='replace')
            self.strings[pos] = s
            self.strings_rev[s] = pos
            pos = end + 1

    def _parse_nodes(self):
        """Parse the node array."""
        for n in range(self.str_count):
            base = 0x18 + n * NODE_SIZE
            if base + NODE_SIZE > len(self.data):
                break

            offsets = []
            for i in range(8):
                off = struct.unpack_from('<I', self.data, base + i * 4)[0]
                offsets.append(off)

            # Offsets 0 and 1 are type and name
            type_off = offsets[0]
            name_off = offsets[1]

            type_str = self.strings.get(type_off)
            if type_off == 0xFFFFFFFF:
                type_str = None
            elif type_str is None:
                type_str = f"<0x{type_off:04X}>"

            name_str = self.strings.get(name_off)
            if name_off == 0xFFFFFFFF:
                name_str = None
            elif name_str is None:
                name_str = f"<0x{name_off:04X}>"

            # Offsets 2-7 are 3 property pairs (key, value)
            props = []
            for i in range(3):
                key_off = offsets[2 + i * 2]
                val = offsets[3 + i * 2]

                if key_off == 0xFFFFFFFF:
                    continue

                key = self.strings.get(key_off)
                if key is None:
                    key = f"<0x{key_off:04X}>"

                # Check if value is a string offset or raw uint32
                if val == 0xFFFFFFFF:
                    val_str = None
                    val_raw = None
                elif val in self.strings:
                    val_str = self.strings[val]
                    val_raw = None
                else:
                    val_str = str(val)
                    val_raw = val

                props.append((key, val_str, val_raw))

            node = XMLBNode(n, base, type_str, name_str, props)
            self.nodes.append(node)

    def get_string_offset(self, s):
        """Get the file offset for a string, or None if not found."""
        return self.strings_rev.get(s)

    def add_string(self, s):
        """Add a string to the string table. Returns the new offset."""
        if s in self.strings_rev:
            return self.strings_rev[s]

        # Find end of string table
        end = self.strtab_off
        while end < len(self.data) and self.data[end] != 0:
            end += 1
        while end < len(self.data) and self.data[end] == 0:
            end += 1

        # Append the new string
        new_off = end
        self.data.extend(s.encode('latin-1'))
        self.data.append(0)

        # Update string tables
        self.strings[new_off] = s
        self.strings_rev[s] = new_off
        self.str_count += 1

        # Update header
        struct.pack_into('<I', self.data, 16, self.str_count)

        self.modified = True
        return new_off

    def get_property_value(self, node_idx, prop_key):
        """Get a property value by key."""
        if node_idx >= len(self.nodes):
            return None
        for key, val_str, val_raw in self.nodes[node_idx].props:
            if key == prop_key:
                return val_str if val_raw is None else val_raw
        return None

    def set_property_value(self, node_idx, prop_idx, value):
        """Set a property value at the given node and property index."""
        if node_idx >= len(self.nodes):
            return
        if prop_idx >= len(self.nodes[node_idx].props):
            return

        key, old_val_str, old_val_raw = self.nodes[node_idx].props[prop_idx]

        # Determine new value type
        if isinstance(value, str):
            # String value - need to get/add string offset
            off = self.get_string_offset(value)
            if off is None:
                off = self.add_string(value)
            new_val = off
            new_val_str = value
            new_val_raw = None
        else:
            # Raw numeric value
            new_val = int(value)
            new_val_str = str(new_val)
            new_val_raw = new_val

        # Update node
        self.nodes[node_idx].props[prop_idx] = (key, new_val_str, new_val_raw)

        # Update binary data
        prop_off = self.nodes[node_idx].file_offset + 16 + prop_idx * 8 + 4
        struct.pack_into('<I', self.data, prop_off, new_val)

        self.modified = True

    def save(self, path=None):
        """Save the XMLB file."""
        if path is None:
            path = self.path

        with open(path, 'wb') as f:
            f.write(self.data)

        self.path = path
        self.modified = False

    def export_text(self):
        """Export to readable text format."""
        lines = []
        lines.append(f"=== {self.path} ===")
        lines.append(f"Size: {len(self.data)} bytes")
        lines.append(f"Magic: 0x{self.magic:08X}")
        lines.append(f"Version: {self.version}")
        lines.append(f"String table: 0x{self.strtab_off:04X}")
        lines.append(f"String count: {self.str_count}")
        lines.append("")
        lines.append(f"Strings ({len(self.strings)}):")
        for off in sorted(self.strings.keys()):
            lines.append(f"  0x{off:04X}: {self.strings[off]!r}")
        lines.append("")
        lines.append(f"Nodes ({len(self.nodes)}):")
        for node in self.nodes:
            lines.append(f"  Node {node.index:3d}: type={node.type!s:<20s} name={node.name!s:<20s}")
            for key, val_str, val_raw in node.props:
                if val_raw is not None:
                    lines.append(f"    {key} = {val_raw}")
                else:
                    lines.append(f"    {key} = {val_str!r}")
        return "\n".join(lines)


def load_xmlb(path):
    """Convenience function to load an XMLB file."""
    return XMLBFile.load(path)


if __name__ == '__main__':
    import sys
    if len(sys.argv) < 2:
        print("Usage: xmlb_core.py <file.xmlb>")
        sys.exit(1)

    xmlb = load_xmlb(sys.argv[1])
    print(xmlb.export_text())
