import struct
with open('/mnt/share/xmen2_mod/xmlb_samples/options.XMLB', 'rb') as f:
    data = f.read()

print(f"File size: {len(data)} bytes")
print()
print("Header:")
print(f"  Magic: 0x{struct.unpack('<I', data[0:4])[0]:08X}")
print(f"  Version: {struct.unpack('<I', data[4:8])[0]}")
print(f"  String table offset: 0x{struct.unpack('<I', data[8:12])[0]:08X}")
print(f"  Unknown: 0x{struct.unpack('<I', data[12:16])[0]:08X}")
print(f"  String count: {struct.unpack('<I', data[16:20])[0]}")
print()

print("String table (starting at 0x18):")
for off in range(0x18, 0x70, 4):
    str_off = struct.unpack('<I', data[off:off+4])[0]
    if str_off == 0 or str_off >= len(data):
        print(f"  offset 0x{off:04X} -> 0x{str_off:08X} (invalid)")
        continue
    try:
        end = data.index(b'\x00', str_off)
        s = data[str_off:end].decode('ascii', errors='replace')
        print(f"  offset 0x{off:04X} -> 0x{str_off:04X}: '{s}'")
    except ValueError:
        print(f"  offset 0x{off:04X} -> 0x{str_off:04X} (no null terminator)")

print()
print("Data after string table (starting at 0x70):")
for off in range(0x70, min(0x200, len(data)), 4):
    val = struct.unpack('<I', data[off:off+4])[0]
    print(f"  0x{off:04X}: 0x{val:08X}")

print()
print("Data 0x200-0x400:")
for off in range(0x200, min(0x400, len(data)), 4):
    val = struct.unpack('<I', data[off:off+4])[0]
    print(f"  0x{off:04X}: 0x{val:08X}")

print()
print("Data 0x400-0x600:")
for off in range(0x400, min(0x600, len(data)), 4):
    val = struct.unpack('<I', data[off:off+4])[0]
    print(f"  0x{off:04X}: 0x{val:08X}")

print()
print("Data 0x600-0x800:")
for off in range(0x600, min(0x800, len(data)), 4):
    val = struct.unpack('<I', data[off:off+4])[0]
    print(f"  0x{off:04X}: 0x{val:08X}")
