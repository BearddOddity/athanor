import struct
with open('/mnt/share/xmen2_mod/xmlb_samples/options.XMLB', 'rb') as f:
    data = f.read()

print(f"File size: {len(data)}")
print()

# String table ends at 0x1A28
# Let's look at data starting from 0x1A28
print("Data from 0x1A28 to 0x1B00:")
for off in range(0x1A28, min(0x1B00, len(data)), 4):
    val = struct.unpack_from('<I', data, off)[0]
    print(f"  0x{off:04X}: 0x{val:08X} ({val})")

print()
print("Data from 0x1B00 to 0x1C00:")
for off in range(0x1B00, min(0x1C00, len(data)), 4):
    val = struct.unpack_from('<I', data, off)[0]
    print(f"  0x{off:04X}: 0x{val:08X} ({val})")

print()
print("Data from 0x1C00 to 0x1D00:")
for off in range(0x1C00, min(0x1D00, len(data)), 4):
    val = struct.unpack_from('<I', data, off)[0]
    print(f"  0x{off:04X}: 0x{val:08X} ({val})")