"""
Athanor - Binary Format Definitions
Creates actual binary files for each new feature format.
"""

import struct, os

# ============================================================
# Format Constants
# ============================================================

MAGIC_XMLB = 0x000011B1
MAGIC_ANIM = 0x0000414E  # 'AN' Animation State Machine
MAGIC_PHYS = 0x00005048   # 'PH' Physics
MAGIC_AUD  = 0x00004155   # 'AU' Audio
MAGIC_COMP = 0x0000434D   # 'CM' Compositor
MAGIC_PBR  = 0x00005042   # 'PB' PBR Material
MAGIC_PLGN = 0x00004C47   # 'PL' Plugin
MAGIC_SAVE = 0x00005347   # 'SG' Save
MAGIC_PIPE = 0x00005049   # 'PI' Pipeline

HEADER_SIZE = 24
NODE_SIZE = 32

# ============================================================
# Animation State Machine (.ANIM)
# ============================================================

def create_anim_format():
    """Create animation state machine binary format"""
    
    # Header
    data = bytearray()
    
    # Magic
    data += struct.pack('<I', MAGIC_ANIM)
    # Version
    data += struct.pack('<I', 1)
    # String table offset (will be calculated)
    strtab_off = len(data) + 20  # header + state count + transition count
    data += struct.pack('<I', strtab_off)
    # Reserved
    data += struct.pack('<I', 0xFFFFFFFF)
    
    states = [
        ('Idle', 'idle_clip', True, 1.0),
        ('Walk', 'walk_clip', True, 1.2),
        ('Run', 'run_clip', True, 1.5),
        ('Attack', 'attack_clip', False, 1.0),
        ('Hit', 'hit_reaction', False, 1.0),
        ('Death', 'death_clip', False, 0.8),
    ]
    
    transitions = [
        ('Idle', 'Walk', 'speed > 0.1'),
        ('Walk', 'Idle', 'speed < 0.1'),
        ('Idle', 'Run', 'speed > 2.0'),
        ('Run', 'Idle', 'speed < 1.0'),
        ('*', 'Attack', 'attack pressed'),
        ('*', 'Hit', 'health changed'),
        ('*', 'Death', 'health <= 0'),
    ]
    
    # State count
    data += struct.pack('<I', len(states))
    # Transition count
    data += struct.pack('<I', len(transitions))
    
    # Reserve space for states
    data += b'\xFF' * (len(states) * NODE_SIZE)
    
    # String table
    strings = []
    for state_name, clip_name, loop, speed in states:
        strings.extend([state_name, clip_name])
    for from_s, to_s, cond in transitions:
        strings.extend([from_s, to_s, cond])
    
    strtab_start = len(data)
    for s in strings:
        data.extend(s.encode('latin-1') + b'\x00')
    
    return bytes(data)

# ============================================================
# Physics Collider (.PHYS)
# ============================================================

def create_phys_format():
    """Create physics collider binary format"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_PHYS)
    data += struct.pack('<I', 1)  # version
    data += struct.pack('<I', 24)  # strtab offset
    data += struct.pack('<I', 0xFFFFFFFF)
    
    shapes = [
        {'type': 'box', 'size': [1.0, 2.0, 1.0], 'mass': 80.0, 'is_trigger': False},
        {'type': 'sphere', 'radius': 0.5, 'mass': 10.0, 'is_trigger': False},
        {'type': 'capsule', 'radius': 0.3, 'height': 1.8, 'mass': 75.0, 'is_trigger': False},
    ]
    
    data += struct.pack('<I', len(shapes))
    
    for shape in shapes:
        # Shape type
        type_map = {'box': 0, 'sphere': 1, 'capsule': 2, 'convex_hull': 3, 'triangle_mesh': 4}
        data += struct.pack('<I', type_map.get(shape['type'], 0))
        
        # Mass
        data += struct.pack('<f', shape['mass'])
        
        # Is trigger
        data += struct.pack('<I', 1 if shape['is_trigger'] else 0)
        
        # Shape params (padded to 32 bytes)
        if shape['type'] == 'box':
            data += struct.pack('<3f', *shape.get('size', [1,1,1]))
        elif shape['type'] == 'sphere':
            data += struct.pack('<f', shape.get('radius', 0.5))
        elif shape['type'] == 'capsule':
            data += struct.pack('<f', shape.get('radius', 0.3))
            data += struct.pack('<f', shape.get('height', 1.8))
        
        # Pad to 32 bytes
        while len(data) % 32 != 0:
            data += b'\x00'
    
    return bytes(data)

# ============================================================
# Audio Bus Definition (.AUD)
# ============================================================

def create_aud_format():
    """Create audio bus definition format"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_AUD)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    buses = [
        {'name': 'Master', 'volume': 1.0, 'effects': []},
        {'name': 'Music', 'volume': 0.8, 'parent': 'Master'},
        {'name': 'SFX', 'volume': 1.0, 'parent': 'Master'},
        {'name': 'Voice', 'volume': 1.0, 'parent': 'Master'},
        {'name': 'UI', 'volume': 0.7, 'parent': 'Master'},
        {'name': 'Ambient', 'volume': 0.6, 'parent': 'Master'},
        {'name': 'Combat', 'volume': 0.9, 'parent': 'SFX'},
    ]
    
    effects = [
        {'name': 'reverb', 'params': [0.5, 0.5, 2.0, 0.5]},
        {'name': 'lowpass', 'params': [2000.0, 1.0]},
        {'name': 'compressor', 'params': [-20.0, 4.0, 0.003, 0.25, 6.0]},
    ]
    
    data += struct.pack('<I', len(buses))
    data += struct.pack('<I', len(effects))
    
    for bus in buses:
        name_off = 0  # will be resolved
        data += struct.pack('<f', bus['volume'])
        parent_off = 0
        data += struct.pack('<I', 0xFFFFFFFF)  # parent
        data += struct.pack('<I', len(bus.get('effects', [])))
        data += b'\x00' * 16  # padding
    
    for effect in effects:
        data += struct.pack('<I', 0xFFFFFFFF)  # name offset
        data += struct.pack('<I', len(effect['params']))
        for p in effect['params']:
            data += struct.pack('<f', p)
        while len(data) % 32 != 0:
            data += b'\x00'
    
    return bytes(data)

# ============================================================
# Compositor Post-Process (.COMP)
# ============================================================

def create_comp_format():
    """Create compositor post-processing definition"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_COMP)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    effects = [
        {'name': 'fxaa', 'enabled': True, 'params': {'quality': 3}},
        {'name': 'bloom', 'enabled': True, 'params': {'threshold': 0.8, 'intensity': 1.0, 'kernelSize': 7}},
        {'name': 'vignette', 'enabled': True, 'params': {'intensity': 0.5}},
        {'name': 'tone_mapping', 'enabled': True, 'params': {'type': 1, 'exposure': 1.0}},  # 1 = ACES
    ]
    
    data += struct.pack('<I', len(effects))
    
    for effect in effects:
        data += struct.pack('<I', 0xFFFFFFFF)  # name offset
        data += struct.pack('<I', 1 if effect['enabled'] else 0)
        data += struct.pack('<I', len(effect['params']))
        for name, val in effect['params'].items():
            data += struct.pack('<I', 0xFFFFFFFF)  # param name
            if isinstance(val, float):
                data += struct.pack('<f', val)
            else:
                data += struct.pack('<I', int(val))
        while len(data) % 32 != 0:
            data += b'\x00'
    
    return bytes(data)

# ============================================================
# PBR Material Definition (.PBR)
# ============================================================

def create_pbr_format():
    """Create PBR material definition"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_PBR)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    materials = [
        {'name': 'metal_plate', 'baseColor': [0.8, 0.8, 0.8, 1.0], 'metallic': 1.0, 'roughness': 0.3, 'normalStrength': 1.0},
        {'name': 'concrete_wall', 'baseColor': [0.5, 0.5, 0.5, 1.0], 'metallic': 0.0, 'roughness': 0.9, 'normalStrength': 0.5},
        {'name': 'wood_crate', 'baseColor': [0.6, 0.4, 0.2, 1.0], 'metallic': 0.0, 'roughness': 0.7, 'normalStrength': 0.3},
        {'name': 'character_skin', 'baseColor': [0.9, 0.7, 0.6, 1.0], 'metallic': 0.0, 'roughness': 0.8, 'normalStrength': 0.2},
    ]
    
    data += struct.pack('<I', len(materials))
    
    for mat in materials:
        data += struct.pack('<I', 0xFFFFFFFF)  # name offset
        data += struct.pack('<4f', *mat['baseColor'])
        data += struct.pack('<f', mat['metallic'])
        data += struct.pack('<f', mat['roughness'])
        data += struct.pack('<f', mat['normalStrength'])
        data += b'\x00' * 8  # padding
    
    return bytes(data)

# ============================================================
# Plugin Manifest (.PLGN)
# ============================================================

def create_plgn_format():
    """Create plugin manifest format"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_PLGN)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    plugins = [
        {'name': 'WidescreenFix', 'version': '1.0', 'author': 'modder', 'entry': 'widescreen.dll'},
        {'name': 'HiResTextures', 'version': '1.2', 'author': 'modder', 'entry': 'hirez.dll'},
        {'name': 'DebugConsole', 'version': '0.5', 'author': 'modder', 'entry': 'console.dll'},
    ]
    
    data += struct.pack('<I', len(plugins))
    
    for plugin in plugins:
        data += struct.pack('<I', 0xFFFFFFFF)  # name
        data += struct.pack('<I', 0xFFFFFFFF)  # version
        data += struct.pack('<I', 0xFFFFFFFF)  # author
        data += struct.pack('<I', 0xFFFFFFFF)  # entry point
        data += struct.pack('<I', 1)  # enabled
        data += b'\x00' * 12
    
    return bytes(data)

# ============================================================
# Save Game Definition (.SAVE)
# ============================================================

def create_save_format():
    """Create save game definition format"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_SAVE)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    sections = [
        {'name': 'player', 'version': 3, 'compress': True},
        {'name': 'inventory', 'version': 2, 'compress': True},
        {'name': 'quests', 'version': 1, 'compress': True},
        {'name': 'world_state', 'version': 4, 'compress': False},
        {'name': 'settings', 'version': 1, 'compress': False},
        {'name': 'statistics', 'version': 1, 'compress': False},
    ]
    
    data += struct.pack('<I', len(sections))
    
    for section in sections:
        data += struct.pack('<I', 0xFFFFFFFF)  # name offset
        data += struct.pack('<I', section['version'])
        data += struct.pack('<I', 1 if section['compress'] else 0)
        data += struct.pack('<I', 0)  # size (filled at runtime)
        data += b'\x00' * 12
    
    return bytes(data)

# ============================================================
# Content Pipeline Definition (.PIPE)
# ============================================================

def create_pipe_format():
    """Create content pipeline definition"""
    
    data = bytearray()
    
    data += struct.pack('<I', MAGIC_PIPE)
    data += struct.pack('<I', 1)
    data += struct.pack('<I', 24)
    data += struct.pack('<I', 0xFFFFFFFF)
    
    importers = [
        {'ext': '.png', 'processor': 'TextureProcessor'},
        {'ext': '.fbx', 'processor': 'ModelProcessor'},
        {'ext': '.wav', 'processor': 'AudioProcessor'},
        {'ext': '.ttf', 'processor': 'FontProcessor'},
    ]
    
    data += struct.pack('<I', len(importers))
    
    for imp in importers:
        data += struct.pack('<I', 0xFFFFFFFF)  # extension
        data += struct.pack('<I', 0xFFFFFFFF)  # processor name
        data += b'\x00' * 24
    
    return bytes(data)

# ============================================================
# Create all new format files
# ============================================================

def create_all_formats():
    """Create all new binary format files"""
    
    output_dir = 'D:/re-lab-share/xmen2_mod/formats'
    os.makedirs(output_dir, exist_ok=True)
    
    formats = {
        'character.anim': create_anim_format(),
        'collision.phys': create_phys_format(),
        'audio.bus': create_aud_format(),
        'postprocess.comp': create_comp_format(),
        'materials.pbr': create_pbr_format(),
        'widescreenfix.plgn': create_plgn_format(),
        'savegame.save': create_save_format(),
        'content.pipe': create_pipe_format(),
    }
    
    for filename, data in formats.items():
        path = os.path.join(output_dir, filename)
        with open(path, 'wb') as f:
            f.write(data)
        print(f'Created: {path} ({len(data)} bytes)')
        
        # Parse and verify
        magic = struct.unpack('<I', data[0:4])[0]
        version = struct.unpack('<I', data[4:8])[0]
        strtab_off = struct.unpack('<I', data[8:12])[0]
        print(f'  Magic: 0x{magic:08X}, Version: {version}, StrTab: 0x{strtab_off:04X}')

if __name__ == '__main__':
    print('Athanor - Creating New Binary Formats')
    print('=' * 50)
    print()
    create_all_formats()
    print()
    print('All format files created successfully!')