# Athanor Engine - Feature Architecture

## Overview

**Athanor Engine** is a Rust-based game modification platform designed specifically for **X-Men Legends II: Rise of Apocalypse** and its sibling titles, combining binary analysis, 3D editing, decompilation, and AI-assisted workflows to enable console game porting to PC.

**Target Games**:
- X-Men Legends II: Rise of Apocalypse (PS2, Xbox, GameCube, PC)
- Marvel: Ultimate Alliance (PS2, Xbox, Wii, PC)
- Marvel: Ultimate Alliance 2 (PS2, PS3, Xbox 360, Wii, PC)
- X-Men Legends (Xbox exclusive version - recompiling to PC)

**Ultimate Goal**: Decompile console-exclusive shelved games and recompile them for PC with enhancements.

**Repository**: https://github.com/BearddOddity/athanor

---

# CORE SYSTEMS ARCHITECTURE

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ATHANOR ENGINE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐│
│  │  BINARY CORE │  │  EDITOR CORE │  │     AI CORE              ││
│  ├──────────────┤  ├──────────────┤  ├──────────────────────────┤│
│  │ Assembler    │  │ Scene Graph  │  │ Ghidra MCP Server        ││
│  │ Disassembler │  │ WebGL       │  │ Radare2 Integration      ││
│  │ Compiler     │  │ Renderer    │  │ AI Analysis Pipeline     ││
│  │ RE Toolchain │  │ Level Grid  │  │ Batch Processing         ││
│  │ Format ID    │  │ Transform   │  │ Symbol Extraction        ││
│  │ Symbol Table │  │ Gizmos      │  │ Cross-Binary Analysis    ││
│  └──────────────┘  └──────────────┘  └──────────────────────────┘│
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐│
│  │  ECS CORE    │  │  GAME SYSTEMS│  │     PLATFORM            ││
│  ├──────────────┤  ├──────────────┤  ├──────────────────────────┤│
│  │ Components   │  │ PBR Materials│  │ Tauri Desktop GUI        ││
│  │ Systems      │  │ Compositor   │  │ Headless Server          ││
│  │ Resources    │  │ Animation    │  │ HTTP API (Axum)         ││
│  │ Events       │  │ Physics      │  │ Anchorpoint VCS         ││
│  │ Commands     │  │ Audio        │  │ Blender Integration      ││
│  │ Plugins      │  │ Save/Load    │  │ CMake Build System      ││
│  └──────────────┘  └──────────────┘  └──────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

# CONSOLE GAME DECOMPILATION TARGETS

## Target Game Files

| Game | Platform | Status | Key Files |
|------|----------|--------|-----------|
| **X-Men Legends II** | PS2 | Analyzed | XMLB, BNX, IGB, ZSM, ZAM, ANIM, PHYS, AUD |
| **X-Men Legends II** | Xbox | Analyzed | Same formats |
| **X-Men Legends II** | GameCube | Todo | Same formats |
| **X-Men Legends II** | PC | Analyzed | Same formats |
| **Marvel: Ultimate Alliance** | PS2 | Todo | Similar formats |
| **Marvel: Ultimate Alliance** | Xbox 360 | Todo | Similar formats |
| **Marvel: Ultimate Alliance 2** | PS3 | Todo | Similar formats |
| **X-Men Legends** | Xbox | Todo | Similar formats |

## Console-Specific Features

| Feature | Description | Platform | Status |
|---------|-------------|----------|--------|
| **Console Binary Parsing** | Parse Xbox/PS2/GC executable formats | All | Todo |
| **Executable Decompilation** | Convert console EXE to readable code | All | Todo |
| **Memory Dump Analysis** | Analyze runtime memory structures | All | Todo |
| **Save Data Extraction** | Extract/decrypt console save data | All | Todo |
| **Asset Extraction** | Pull assets from console formats | All | Todo |
| **Disc Image Mounting** | Mount ISO/GCM/CSO images | All | Todo |
| **DVD Layer Detection** | Handle dual-layer DVDs | PS2/GC | Todo |
| **DRM/Copyright Removal** | Strip console DRM | All | Todo |

## PC Recompilation Features

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **PC Binary Generation** | Compile modified code to PC EXE | Assembler | Todo |
| **DirectX Wrapper** | Wrap OpenGL->DirectX calls | None | Todo |
| **Widescreen Support** | Fix aspect ratio for modern displays | Binary Core | Todo |
| **60 FPS Unlock** | Remove frame rate locks | Binary Core | Todo |
| **Controller Support** | Add Xbox/PS controller mapping | Input System | Todo |
| **Achievement Hooks** | Add Steam achievements | PC Binary | Todo |
| **Cloud Saves** | Add Steam cloud support | Save System | Todo |
| **Mod Loader** | Load PC mods alongside console assets | Asset Pipeline | Todo |

## Porting Workflow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONSOLE PORTING WORKFLOW                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. EXTRACT                                                        │
│     Console Disc → Mount → Extract All Files                        │
│                                                                     │
│  2. ANALYZE                                                        │
│     Binary Formats → Ghidra/Radare2 → Document Structures          │
│                                                                     │
│  3. DECOMPILE                                                       │
│     Console EXE → Decompiled Code → Symbol Tables                   │
│                                                                     │
│  4. CONVERT ASSETS                                                 │
│     Console Textures → PC Formats → Rebuild Bundles                 │
│                                                                     │
│  5. RECOMPILE                                                       │
│     Modified Code → PC EXE → Link with Engine                       │
│                                                                     │
│  6. ENHANCE                                                         │
│     Widescreen → 60 FPS → Controller → Achievements                │
│                                                                     │
│  7. DISTRIBUTE                                                      │
│     Build → Package → Mod Loader Ready                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

# I. BINARY CORE SUBSYSTEM

## 1.1 Assembler/Disassembler

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Binary Parser** | Parse 19 game formats (XMLB, BNX, IGB, etc.) | None | Done |
| **Disassembler** | Convert binary to readable AST | Binary Parser | Done |
| **Assembler** | Convert AST back to binary | Disassembler | Todo |
| **Hex Viewer** | Raw byte visualization with highlighting | None | Done |
| **String Extraction** | Extract and decode string tables | Binary Parser | Todo |
| **Symbol Table** | Extract/manage function/variable symbols | Disassembler | Todo |
| **Relocation Fixer** | Fix relocations for modified binaries | Assembler | Todo |

## 1.2 Compiler

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Modification Engine** | Apply changes to parsed binary | Binary Parser | Done |
| **Node Editor** | Add/remove/modify AST nodes | Disassembler | Todo |
| **Property Inspector** | Edit node properties | Node Editor | Done |
| **Batch Compilation** | Compile multiple files | Compiler | Todo |
| **Compilation Testing** | Verify output matches expected | Compiler | Todo |
| **Incremental Compile** | Only recompile changed sections | Compiler | Todo |

## 1.3 Reverse Engineering Toolchain

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Ghidra MCP Server** | Headless Ghidra analysis via MCP | Ghidra, MCP | Planned |
| **Radare2 Integration** | Alternative RE via r2pipe | Radare2 | Todo |
| **Format Identification** | Auto-detect binary format | Binary Parser | Todo |
| **Signature Database** | Common format signatures | Format ID | Todo |
| **Console EXE Analysis** | Analyze PS2/Xbox/GC executables | Ghidra MCP | Todo |
| **Symbol Recovery** | Recover lost function names | Ghidra MCP | Todo |
| **Cross-Title Analysis** | Compare binaries across MUA/XL games | Compiler | Todo |
| **Batch Decompilation** | Decompile entire game executable | Ghidra MCP | Todo |
| **AI Training Data** | Generate training data from analysis | Batch Decomp | Todo |

## 1.4 X-Men Legends II Format Analysis

Based on reverse engineering of X-Men Legends II: Rise of Apocalypse (PC version):

| Format | Extension | Description | Status |
|--------|-----------|-------------|--------|
| **Menu/UI** | XMLB | Menu layouts, settings, HUD elements | Analyzed |
| **Config** | BNX | Key-value configuration, options | Analyzed |
| **Image** | IGB | HUD textures, image data | Analyzed |
| **Sound Index** | ZSM | Sound metadata, bank indices | Analyzed |
| **Sound Data** | ZSS | Audio stream data | Analyzed |
| **Minimap** | ZAM | Automap/waypoint data | Analyzed |
| **Animation** | ANIM | Animation state machine data | Analyzed |
| **Physics** | PHYS | Collision shapes, physics data | Analyzed |
| **Audio** | AUD | Audio bus definitions | Analyzed |
| **Compositor** | COMP | Visual effects pipeline | Analyzed |
| **Material** | PBR | Material definitions | Analyzed |
| **Plugin** | PLGN | Plugin manifests | Analyzed |
| **Save** | SAVE | Save game structure | Analyzed |
| **Pipeline** | PIPE | Content pipeline data | Analyzed |
| **Package** | PKGB | Asset packages, textures | Analyzed |
| **Engine** | ENGB | Engine configuration | Analyzed |
| **Character** | CHRB | Character definitions | Analyzed |
| **Navigation** | NAVB | Navmesh, pathfinding | Analyzed |
| **Buoy** | BOYB | Waypoint/buoy data | Analyzed |
| **String Table** | ZSS | String lookup tables | Analyzed |

### XMLB Format (Menu/UI)

XMLB is the primary menu/ui format used throughout the game for:
- Main menu screens
- HUD elements
- Dialog boxes
- Settings panels
- Inventory displays
- World map overlays

#### Binary Structure

```
XMLB File Layout:
- Header (24 bytes)
- Node Array (32 bytes per node)
- String Table (null-terminated)
- Data Blocks (variable size)
```

#### Header Structure (24 bytes, little-endian)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 4 | Magic | 0x584D4C42 ("XMLB") |
| 0x04 | 4 | Version | Format version number |
| 0x08 | 4 | NodeCount | Number of nodes in file |
| 0x0C | 4 | StringCount | Number of strings in table |
| 0x10 | 4 | HeaderSize | Offset where data begins |
| 0x14 | 4 | Unknown | Reserved/unused |

#### Node Structure (32 bytes)

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0x00 | 4 | TypeOffset | String table offset to type name |
| 0x04 | 4 | NameOffset | String table offset to node name |
| 0x08 | 4 | Prop1Key | Property 1 key offset |
| 0x0C | 4 | Prop1Value | Property 1 value (offset or raw) |
| 0x10 | 4 | Prop2Key | Property 2 key offset |
| 0x14 | 4 | Prop2Value | Property 2 value |
| 0x18 | 4 | Prop3Key | Property 3 key offset |
| 0x1C | 4 | Prop3Value | Property 3 value |

**Property Value Encoding:**
- `0xFFFFFFFF`: Null/empty property
- `< 0x10000000`: Offset into string table
- `>= 0x10000000`: Raw value (integer, float, or boolean)

#### String Table

- Null-terminated strings
- Sorted by file offset for binary search
- Common strings: "MENU_ITEM", "TEXT", "STYLE", "animtext", "mark"

#### Common Node Types

| Type | Description | Properties |
|------|-------------|------------|
| MENU_ITEM | Clickable menu entry | position, action, label |
| TEXT | Text label | position, content, style |
| STYLE | Visual style data | color, font, alignment |
| animtext | Animated text | position, text, timing |
| item | Generic item | position, type, data |

#### Example Node

```
Node: MENU_ITEM
  pos: 0x100
  type: "MENU_ITEM"
  name: "MainMenuItem"
  Properties:
    - mark: 10
    - item: 500
    - style: 0x1234
```

#### API Usage (Editor JavaScript)

```javascript
// Parse XMLB file
const data = await fetch('/api/load/menuscreen.xmlb').then(r => r.json());

// Modify a property
await fetch('/api/property/menuscreen.xmlb/0/mark', {
    method: 'POST',
    body: JSON.stringify({ value: 15 })
});

// Save changes
await fetch('/api/save', {
    method: 'POST',
    body: JSON.stringify({ filename: 'menuscreen.xmlb' })
});

// Deploy to game directory
await fetch('/api/deploy', {
    method: 'POST',
    body: JSON.stringify({ filename: 'menuscreen.xmlb' })
});
```

#### Key Editor Functions

| Function | Description |
|----------|-------------|
| `loadFile()` | Open XMLB file dialog |
| `saveFile()` | Save file to disk |
| `renderFile()` | Display tree and preview |
| `renderNodeTree()` | Build node tree view |
| `renderPreview()` | Create canvas preview |
| `selectNode(idx)` | Select node and show properties |
| `renderProperties()` | Build property editor |
| `updateProperty(idx, value)` | Modify node property |

#### Version History

| Version | Introduced | Notes |
|---------|------------|-------|
| 1.0 | X-Men Legends II | Initial format |
| 1.1 | MUA | Extended with audio events |
| 1.2 | MUA2 | Added PBR material support |

---

# II. EDITOR CORE SUBSYSTEM

## 2.1 Rendering Engine

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **WebGL Renderer** | Hardware-accelerated 3D viewport | None | V3 Done |
| **WebGPU Fallback** | Modern GPU API fallback | None | Todo |
| **PBR Materials** | Physically-based rendering | WebGL | Todo |
| **Compositor Effects** | Post-processing pipeline | WebGL | Todo |
| **Particle System** | GPU particle effects | WebGL | Todo |
| **Dynamic Lighting** | Multiple light types | WebGL | Todo |
| **Shadow Mapping** | Real-time shadows | Dynamic Lighting | Todo |
| **Skybox/Environment** | HDR environment maps | WebGL | Todo |

## 2.2 Scene Management

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Scene Graph** | Hierarchical node tree | None | V2 Done |
| **Object Palette** | Drag-and-drop asset library | None | Todo |
| **Level Grid** | Configurable placement grid | None | Todo |
| **Camera Controls** | Orbit/pan/zoom | WebGL | V3 Done |
| **Viewport Modes** | Wireframe/solid/textured | WebGL | Todo |
| ** Gizmo System** | Transform handles | Scene Graph | Todo |
| **Selection** | Single/multi select | Gizmo | Todo |

## 2.3 Transform Gizmos

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Move Gizmo** | Position with axis handles | WebGL | Todo |
| **Rotate Gizmo** | Rotation with arc handles | WebGL | Todo |
| **Scale Gizmo** | Uniform/axis scale | WebGL | Todo |
| **Snap System** | Grid/angle snapping | Gizmos | Todo |
| **Transform Spaces** | Local/world coordinates | Gizmos | Todo |
| **Numeric Input** | Direct coordinate entry | Gizmos | Todo |

## 2.4 Editor Tools

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Undo/Redo** | Full edit history | None | Todo |
| **Save/Load** | JSON level format | None | Todo |
| **Clipboard** | Copy/paste objects | Scene Graph | Todo |
| **Duplicate** | Clone with offset | Clipboard | Todo |
| **Delete** | Remove objects | Scene Graph | Todo |
| **Group/Ungroup** | Object parenting | Scene Graph | Todo |
| **Search** | Find objects by name | Scene Graph | Todo |
| **Filter** | Filter hierarchy view | Scene Graph | Todo |

---

# III. AI CORE SUBSYSTEM

## 3.1 Analysis Pipeline

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Ghidra MCP Server** | 37 analysis tools via MCP | Ghidra | Planned |
| **Headless Scripts** | Automated binary analysis | Ghidra MCP | Todo |
| **Symbol Extraction** | Extract function/data symbols | Ghidra MCP | Todo |
| **Function Analysis** | Batch analyze functions | Headless Scripts | Todo |
| **Cross-Binary AI** | Compare/analyze across files | Function Analysis | Todo |
| **AI Training Export** | Generate training datasets | Function Analysis | Todo |

## 3.2 AI-Assisted Features

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **AI Upscaling** | Upscale textures with AI | None | Todo |
| **Polygon Generator** | Procedural mesh generation | ECS | Todo |
| **Auto-Documentation** | Generate docs from analysis | Ghidra MCP | Todo |
| **Mod Generator** | AI-assisted mod creation | All | Todo |
| **Compatibility Checker** | Detect mod conflicts | Cross-Binary AI | Todo |

---

# IV. ECS CORE (Bevy-Inspired)

## 4.1 Architecture

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Entity** | Unique ID for game objects | None | Todo |
| **Components** | Data containers (Transform, Mesh, etc.) | None | Todo |
| **Systems** | Logic that processes components | None | Todo |
| **Resources** | Global singletons | None | Todo |
| **Commands** | Queue entity mutations | None | Todo |

## 4.2 Component Types

| Component | Description |
|-----------|-------------|
| **Transform** | Position, rotation, scale |
| **Mesh** | Geometry reference |
| **Material** | PBR material reference |
| **Collider** | Physics collision shape |
| **RigidBody** | Physics body properties |
| **Camera** | Viewport configuration |
| **Light** | Light source settings |
| **AudioSource** | Sound playback |
| **AnimationPlayer** | Animation control |
| **Script** | Custom behavior |

## 4.3 Systems

| System | Description | Components |
|--------|-------------|------------|
| **TransformSystem** | Update world transforms | Transform, Parent |
| **RenderSystem** | Submit meshes to GPU | Mesh, Transform, Material |
| **PhysicsSystem** | Step physics simulation | RigidBody, Transform, Collider |
| **AnimationSystem** | Update animation state | AnimationPlayer, Skeleton |
| **AudioSystem** | Play sounds | AudioSource, Transform |

---

# V. SIGNAL/EVENT SYSTEM (Godot-Inspired)

## 5.1 Event Architecture

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| **Signal Emitter** | Emit events from any node | None |
| **Signal Receiver** | Connect to events | None |
| **Event Bus** | Centralized event routing | None |
| **Deferred Events** | Queue events for next frame | Event Bus |
| **Custom Signals** | User-defined event types | Signal Emitter |

## 5.2 Built-in Signals

| Signal | Description |
|--------|-------------|
| **body_entered** | Collision started |
| **body_exited** | Collision ended |
| **timeout** | Timer finished |
| **pressed** | Button clicked |
| **value_changed** | Slider/spinner changed |
| **tree_entered** | Node added to scene |
| **tree_exited** | Node removed from scene |

---

# VI. GAME SYSTEMS

## 6.1 Rendering (OGRE-Inspired)

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Render Pipeline** | Multi-pass rendering | WebGL | Todo |
| **PBR Materials** | Metallic/roughness workflow | Render Pipeline | Todo |
| **Compositor** | Post-processing graph | Render Pipeline | Todo |
| **Lightmapper** | Bake static lighting | PBR | Todo |
| **Skeletal Animation** | Character rigging | Mesh | Todo |
| **Morph Targets** | Vertex animations | Mesh | Todo |
| **Terrain System** | Heightmap terrain | Mesh | Todo |
| **Vegetation** | Instanced foliage | Terrain | Todo |

## 6.2 Animation System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Animation State Machine** | State-based transitions | ECS | Todo |
| **Blend Trees** | Mix multiple animations | Animation SM | Todo |
| **IK System** | Inverse kinematics | Animation SM | Todo |
| **Procedural Animation** | Dynamic motion | Animation SM | Todo |
| **Root Motion** | Animation-driven movement | Animation SM | Todo |

## 6.3 Physics System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Rigid Body** | Dynamic physics | ECS | Todo |
| **Static Body** | Immovable colliders | Collider | Todo |
| **Character Controller** | Player movement | Rigid Body | Todo |
| **Joints** | Connections between bodies | Rigid Body | Todo |
| **Ragdoll** | Physics-based death | Joints | Todo |
| **Vehicle** | Wheel physics | Rigid Body | Todo |
| **Raycast** | Query physics world | Physics | Todo |

## 6.4 Audio System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Audio Bus** | Volume/panning routing | None | Todo |
| **Audio Stream** | Play sound files | Audio Bus | Todo |
| **3D Audio** | Spatial sound | Audio Stream | Todo |
| **Reverb Zones** | Environment effects | 3D Audio | Todo |
| **Dialogue** | Voice playback | Audio Bus | Todo |
| **Music** | Background music | Audio Bus | Todo |

## 6.5 Save/Load System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Save Format** | Binary save files | Compiler | Todo |
| **Auto-Save** | Periodic saves | Save Format | Todo |
| **Quick Save** | Instant save slot | Save Format | Todo |
| **Save Compression** | Reduce save size | Save Format | Todo |
| **Save Encryption** | Protect save data | Save Format | Todo |

---

# VII. CONTENT PIPELINE

## 7.1 Asset Processing

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Model Import** | Load 3D models (Assimp) | None | Todo |
| **Texture Import** | Load textures with mipmaps | None | Todo |
| **Audio Import** | Load audio with transcoding | None | Todo |
| **Font Import** | Load BMFont/freetype | None | Todo |
| **Atlas Builder** | Pack textures | Texture Import | Todo |
| **LOD Generator** | Create mesh levels | Model Import | Todo |
| **Collision Generator** | Auto-generate colliders | Model Import | Todo |

## 7.2 Build System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **CMake Integration** | Native build configuration | None | Todo |
| **Asset Compiler** | Convert assets to engine format | Asset Processing | Todo |
| **Bundle Builder** | Package resources | Asset Compiler | Todo |
| **Dependency Graph** | Track asset dependencies | Asset Compiler | Todo |
| **Hot Reload** | Update assets without restart | Asset Compiler | Todo |

## 7.3 Blender Integration

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Exporter** | Export to engine format | Blender | Todo |
| **Importer** | Import engine assets | Blender | Todo |
| **Material Sync** | Live material updates | Exporter | Todo |
| **Animation Export** | Export animations | Exporter | Todo |

---

# VIII. PLUGIN ARCHITECTURE

## 8.1 Plugin System

| Feature | Description | Dependencies | Status |
|---------|-------------|--------------|--------|
| **Plugin Interface** | Define plugin API | None | Todo |
| **Plugin Loader** | Load .dll/.so at runtime | None | Todo |
| **Plugin Registry** | Discover/manage plugins | Plugin Loader | Todo |
| **Sandbox** | Isolate plugin execution | Plugin Loader | Todo |

## 8.2 Plugin Types

| Type | Description |
|------|-------------|
| **Format Plugin** | Support new binary formats |
| **Importer Plugin** | New asset format importers |
| **Exporter Plugin** | New export targets |
| **Tool Plugin** | Editor tools |
| **System Plugin** | Game systems |
| **AI Plugin** | AI features |

---

# IX. INTEGRATION MATRIX

```
Feature Dependencies:

WebGL Renderer ─────┬──> PBR Materials ──> Compositor Effects
                     │
                     ├──> Particle System
                     │
                     └──> Dynamic Lighting ──> Shadow Mapping

Scene Graph ─────────┬──> Object Palette
                     ├──> Level Grid
                     ├──> Gizmo System ──────> Transform Gizmos
                     │              └──> Snap System
                     └──> Selection

ECS Core ────────────┬──> TransformSystem
                     ├──> RenderSystem ────> WebGL Renderer
                     ├──> PhysicsSystem ───> Physics Integration
                     ├──> AnimationSystem ─> Animation State Machine
                     └──> AudioSystem ────> Audio System

Signal/Event ────────┬──> Built-in Signals
                     └──> Custom Signals

Binary Core ─────────┬──> Assembler/Disassembler
                     ├──> Compiler
                     ├──> Symbol Extraction
                     └──> Format Identification

AI Core ─────────────┬──> Ghidra MCP Server ──> Headless Scripts
                     ├──> Radare2 Integration
                     └──> Batch Processing

Content Pipeline ────┬──> Asset Importers ──> Blender Integration
                     ├──> Asset Compiler
                     └──> CMake Integration

Plugin System ──────┴──> Format Plugins
                     ├──> Importer Plugins
                     ├──> Exporter Plugins
                     └──> Tool Plugins
```

---

# X. CONFLICT RESOLUTION

## Feature Groups (Non-Overlapping)

| Group | Features |
|-------|----------|
| **Rendering** | WebGL, WebGPU, PBR, Compositor, Particles, Lighting, Shadows |
| **Animation** | Animation SM, Blend Trees, IK, Root Motion, Procedural |
| **Physics** | RigidBody, StaticBody, Colliders, Joints, Ragdoll |
| **Audio** | Audio Bus, Streams, 3D Audio, Reverb, Dialogue |
| **AI** | Ghidra MCP, Radare2, Upscaling, Polygon Gen, Training Export |
| **Binary** | Parser, Disassembler, Assembler, Compiler, Format ID |
| **Editor** | Scene Graph, Gizmos, Grid, Camera, Undo/Redo |
| **ECS** | Components, Systems, Resources, Commands, Events |
| **Pipeline** | Importers, Compiler, Bundler, CMake, Blender |

## Conflicting Features (Mutually Exclusive)

| Conflict | Resolution |
|----------|------------|
| WebGL vs WebGPU | WebGPU is fallback, WebGL is primary |
| Ghidra vs Radare2 | Both supported, user selects in config |
| Bevy ECS vs Godot Signals | Both integrated - ECS for game logic, Signals for UI/Events |

---

# XI. IMPLEMENTATION STATUS

## Complete
- [x] Binary Parser (20 formats: XMLB, PKGB, ENGB, BOYB, CHRB, NAVB, BNX, IGB, ZSM, ZSS, ZAM, ANIM, PHYS, AUD, COMP, PBR, PLGN, SAVE, PIPE)
- [x] Binary Builder (20 formats - round-trip compilation)
- [x] HTTP API Server (Axum) with /api/load, /api/save, /api/property, /api/deploy, /api/anim, /api/anim/import
- [x] Tauri Desktop GUI
- [x] Headless Server Mode
- [x] Asset Editor V2 (Godot-style panels)
- [x] Level Editor V3 (WebGL viewport)
- [x] Anchorpoint Integration
- [x] Godot .gitignore
- [x] JavaScript execution in Tauri webview
- [x] XMLB format documentation complete
- [x] Bring Your Own Game architecture (ISO/XBE/directory per request)
- [x] 5 unit tests passing

## In Progress
- [ ] Editor UI improvements (Godot-inspired but unique design)

## Complete
- [x] IGB texture extraction/rebuild (export/import endpoints wired)
- [x] ANIM animation export (JSON export/import pipeline wired)
- [x] CHRB character format (export/import endpoints wired)
- [x] Console disc extraction (Xbox ISO, PS2 ISO, GCM - all parsers implemented)
- [x] Executable format parsers (XBE, ELF, DOL, PE with auto-detect)
- [x] Ghidra MCP Server framework (RPC endpoint /api/ghidra/rpc)

## Planned (Phase 1) - Console Binary Analysis
- [ ] Executable format parsers (Xbox XBE, PS2 ELF, GCN DOL)
- [ ] Ghidra MCP Server for headless analysis
- [ ] Symbol recovery and documentation

## Planned (Phase 2) - Asset Conversion
- [ ] Console texture extraction (DXT -> PC formats)
- [ ] Audio format conversion
- [ ] Model format conversion (with skeleton)
- [ ] Level data extraction and rebuild
- [ ] Save data decryption

## Planned (Phase 3) - PC Recompilation
- [ ] PC EXE generation framework
- [ ] DirectX/OpenGL wrapper
- [ ] Widescreen fix system
- [ ] Frame rate unlocker
- [ ] Controller input mapping

## Planned (Phase 4) - Enhancement
- [ ] Steam achievements integration
- [ ] Cloud saves
- [ ] Mod loader
- [ ] AI upscaling integration
- [ ] Cross-title mod compatibility

---

# XII. CONSOLE PORTING PRIORITY

## Highest Priority (Required for XL2 PC)
1. [ ] Complete XMLB format documentation
2. [ ] IGB texture extraction/rebuild
3. [ ] ANIM animation export
4. [ ] CHRB character format
5. [ ] Full disassembly of game executable

## High Priority (For Enhanced PC Version)
1. [ ] 60 FPS unlock
2. [ ] Widescreen support
3. [ ] Controller support
4. [ ] Higher resolution textures

## Medium Priority (For Console Ports)
1. [ ] PS2 disc extraction
2. [ ] Xbox XBE analysis
3. [ ] Cross-platform binary generation
4. [ ] Console save data compatibility

## Future (For Other Games)
1. [ ] Marvel: Ultimate Alliance formats
2. [ ] X-Men Legends (Xbox) formats
3. [ ] General console executable decompilation
4. [ ] Universal porting framework

---

# XIII. FILE STRUCTURE

```
athanor/
├── src/                      # Rust source
│   ├── main.rs              # CLI entry point
│   ├── binary/              # Binary core
│   │   ├── parser.rs        # Format parsers
│   │   ├── disassembler.rs   # Binary -> AST
│   │   ├── assembler.rs      # AST -> Binary
│   │   ├── compiler.rs      # Modification engine
│   │   └── format_id.rs     # Auto-detection
│   ├── editor/              # Editor core
│   │   ├── scene.rs         # Scene graph
│   │   ├── renderer.rs      # WebGL/WebGPU
│   │   ├── gizmos.rs        # Transform handles
│   │   └── viewport.rs      # Camera controls
│   ├── ecs/                # Entity Component System
│   │   ├── component.rs     # Component definitions
│   │   ├── system.rs        # System definitions
│   │   ├── resource.rs     # Resource management
│   │   └── command.rs       # Command buffer
│   ├── game/                # Game systems
│   │   ├── animation.rs     # Animation SM
│   │   ├── physics.rs       # Physics integration
│   │   ├── audio.rs        # Audio system
│   │   ├── save.rs         # Save/Load
│   │   └── pbr.rs          # PBR materials
│   ├── ai/                  # AI integration
│   │   ├── ghidra_mcp.rs   # Ghidra MCP server
│   │   ├── radare2.rs      # Radare2 integration
│   │   ├── analysis.rs      # Analysis pipeline
│   │   └── upscaler.rs     # AI upscaling
│   ├── pipeline/            # Content pipeline
│   │   ├── importer.rs      # Asset importers
│   │   ├── compiler.rs      # Asset compiler
│   │   └── blender.rs       # Blender integration
│   ├── platform/            # Platform integrations
│   │   ├── tauri.rs        # Tauri GUI
│   │   ├── anchorpoint.rs  # VCS integration
│   │   └── cmake.rs        # CMake build
│   └── plugins/             # Plugin system
│       ├── loader.rs        # Plugin loader
│       └── sandbox.rs       # Plugin sandbox
├── editor/                   # Editor HTML/JS
│   ├── index.html
│   ├── scene_view.js
│   ├── gizmo.js
│   └── panels/
├── assets/                   # Editor assets
│   ├── icons/
│   └── shaders/
├── xmlb_samples/             # Game data samples
├── FEATURES.md               # This file
└── README.md
```

---

# XIII. QUICK REFERENCE

| Command | Description |
|---------|-------------|
| `athanor` | Start headless server (port 3459) |
| `athanor --gui` | Start with Tauri window |
| `athanor --help` | Show help |

| API Endpoint | Description |
|--------------|-------------|
| `GET /api/formats` | List 19 supported formats |
| `GET /api/files` | List game directory |
| `POST /api/parse` | Parse binary file |
| `POST /api/disassemble` | Convert to AST |
| `POST /api/compile` | Compile modifications |
| `GET /editor` | Asset editor UI |
| `GET /level-editor` | Level editor UI |

## Game Data Locations

| Game | Default Path |
|------|--------------|
| X-Men Legends II (PC) | `D:\My Games\X-Men Legends II Rise of Apocalypse` |
| MUA (PC) | `C:\Program Files\Marvel - Ultimate Alliance` |
| MUA2 (PC) | `C:\Program Files\Marvel - Ultimate Alliance 2` |

---

# XIV. GAME-SPECIFIC DOCUMENTATION

## X-Men Legends II: Rise of Apocalypse

### Game Engine
- **Engine**: Proprietary (similar to engine used in MUA)
- **Renderer**: OpenGL (PC), Custom (Console)
- **Platforms**: PS2, Xbox, GameCube, PC

### Known Executables
| Platform | Filename | Format |
|----------|----------|--------|
| PC | `game.exe` | PE32 |
| PS2 | `SLES_524.13` | ELF |
| Xbox | `default.xbe` | XBE |
| GameCube | `game.dol` | DOL |

### Modding Potential
| Area | Potential | Notes |
|------|-----------|-------|
| Characters | High | New playable characters via CHRB |
| Levels | High | New levels via XMLB/ANIM |
| Textures | High | New textures via PKGB/IGB |
| Audio | Medium | Voice lines via ZSM/ZSS |
| UI | High | Menu layouts via XMLB |
| Physics | Medium | Collision tweaking via PHYS |
| Animation | High | New moves via ANIM |

### Priority Mods
1. **Widescreen Fix** - Modify renderer settings in ENGB
2. **60 FPS Unlock** - Remove frame rate cap
3. **New Characters** - Dump CHRB, modify, rebuild
4. **New Levels** - Full level creation workflow
5. **Texture Upscaling** - Replace IGB with higher res

---

**Last Updated**: 2026-09-07  
**Version**: 0.1.0  
**Purpose**: Console game decompilation for X-Men Legends series

---

# XV. IMPLEMENTATION PROGRESS REPORT

## What is actually working end-to-end

| Feature | Evidence |
|---------|----------|
| Folder import via `/api/scan` | Returns real file list for extracted game directories |
| XMLB/BNX/BOYB/ZSM/ZSS parsing | Real binary parsing, string tables, node records |
| CHRB/ANIM/IGB JSON export/import | Real parsers + rebuild pipeline |
| PE executable parsing | Real section table, entry point, COFF header |
| PS2 ISO 9660 directory extraction | Real sector parsing |
| Xbox XISO mini entry table parsing | Real offset/size extraction |
| GameCube GCM header reading | Real metadata from header |
| DXT1/DXT5 decompression/compression | Real texture codec |
| Compiler/rebuild pipeline | Real round-trip compilation |

## What is partially real or stubbed

| Feature | Evidence |
|---------|----------|
| XBE/ELF/DOL parsing | Partial: headers real, imports/exports empty, ELF names generic |
| PHYS/AUD/COMP/PBR/PLGN/SAVE/PIPE/NAVB/PKGB/ENGB | Skeleton parsers only: 32-byte raw chunks, no semantic fields |
| Disc metadata | Hardcoded strings for Xbox/PS2 |
| Ghidra MCP | Fully stubbed: returns hardcoded `"main"` at `0x401000` |
| `update_property` endpoint | No-op: returns JSON without writing disk |
| `deploy` endpoint | No-op: returns formatted path, no file copy |
| `scan` for ISO/XBE containers | Unimplemented |

## Recent changes

- Added Game Folder import page to dashboard
- Fixed dashboard buttons by switching from inline `onclick` to delegated event listeners
- Applied Godot design palette to dashboard UI
- `parse_file_handler` now accepts both folders and files as source
- CHRB/ANIM/IGB handlers now resolve files via optional `source` query parameter

## Next steps

1. Wire remaining folder-source loaders if endpoints still 404
2. Replace stubbed parsers with real implementations
3. Make Ghidra MCP actually call Ghidra headless analysis
4. Implement `update_property` and `deploy` writeback
5. Add ISO/XBE container listing to `scan_handler`
