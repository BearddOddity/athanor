# Athanor Engine - Feature Architecture

## Overview

**Athanor Engine** is a Rust-based game modification platform combining binary analysis, 3D editing, and AI-assisted workflows.

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
| **Cross-Title Analysis** | Compare binaries across games | Compiler | Todo |
| **Batch Decompilation** | Decompile multiple binaries | Ghidra/Radare2 | Todo |
| **AI Training Data** | Generate training data from analysis | Batch Decomp | Todo |

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
- [x] Binary Parser (19 formats)
- [x] HTTP API Server (Axum)
- [x] Tauri Desktop GUI
- [x] Headless Server Mode
- [x] Asset Editor V2 (Godot-style panels)
- [x] Level Editor V3 (WebGL viewport)
- [x] Anchorpoint Integration
- [x] Godot .gitignore

## In Progress
- [ ] **JavaScript Execution in Tauri Webview** (blocking editor interactivity)

## Planned (Phase 1)
- [ ] WebGL Renderer improvements
- [ ] Transform Gizmos
- [ ] Undo/Redo System
- [ ] Save/Load System
- [ ] Ghidra MCP Server

## Planned (Phase 2)
- [ ] ECS Core
- [ ] Signal/Event System
- [ ] PBR Materials
- [ ] Physics Integration
- [ ] Animation State Machine

## Planned (Phase 3)
- [ ] Content Pipeline
- [ ] Blender Integration
- [ ] CMake Integration
- [ ] Plugin Architecture
- [ ] AI Upscaling

## Planned (Phase 4)
- [ ] Full Ghidra/Radare2 Integration
- [ ] Cross-Binary Analysis
- [ ] AI Training Data Generation
- [ ] Mod Generator

---

# XII. FILE STRUCTURE

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

---

**Last Updated**: 2026-09-06  
**Version**: 0.1.0
