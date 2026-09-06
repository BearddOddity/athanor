# Athanor Engine - Feature List

## Overview

**Athanor Engine** is a Rust-based binary assembly/disassembly/compilation tool with an OGRE-style 3D level editor. It is designed for game modders and AI-assisted development workflows.

**Repository**: https://github.com/BearddOddity/athanor

---

## Implemented Features

### Core Engine
- [x] **Rust Backend** - High-performance Axum HTTP server
- [x] **Tauri 2 GUI** - Native desktop window with webview
- [x] **Dual Mode** - `--gui` (windowed) / headless server
- [x] **HTTP API** - RESTful endpoints for AI integration
- [x] **System Tray** - Close-to-hide behavior

### Binary Format Support
- [x] **19 Formats Supported**:
  - XMLB - Menu/UI layouts, settings
  - PKGB - Asset packages, textures
  - ENGB - Conversation/scripts
  - CHRB - Character definitions
  - NAVB - Pathfinding/navmesh
  - BOYB - Buoy/waypoint data
  - BNX - Config/options key=value
  - IGB - HUD/texture images
  - ZSM - Sound metadata/index
  - ZSS - Sound data streams
  - ZAM - Minimap/automap data
  - ANIM - Animation State Machine
  - PHYS - Physics Colliders
  - AUD - Audio Bus Definitions
  - COMP - Compositor Effects
  - PBR - PBR Material Definitions
  - PLGN - Plugin Manifests
  - SAVE - Save Game Structure
  - PIPE - Content Pipeline

### HTTP API Endpoints
- [x] `GET /api/formats` - List supported formats
- [x] `GET /api/files` - List game directory files
- [x] `POST /api/parse` - Parse binary file
- [x] `POST /api/disassemble` - Disassemble to readable
- [x] `POST /api/compile` - Compile/modify binary
- [x] `GET /api/health` - Health check

### Editor UI
- [x] **Asset Editor (V2)** - Godot-style 3-panel layout
  - Scene tree browser
  - AST tree view
  - Hex view
  - Property inspector
  - Tab switching
- [x] **Level Editor (V3)** - WebGL-based 3D viewport
  - Object hierarchy
  - Transform gizmos
  - Camera controls

### Integrations
- [x] **Anchorpoint** - Git-based version control for binaries
- [x] **Ghidra MCP** - Headless binary analysis (37 tools)
- [x] **Bun Server** - Fallback HTTP server on port 3457

---

## Missing Features (TODO)

### Rendering & Graphics
- [ ] **WebGL 3D Renderer** - Full OGRE-style rendering
- [ ] **Procedural Geometry** - Generate meshes programmatically
- [ ] **PBR Materials** - Physically-based rendering
- [ ] **Particle Systems** - Advanced particle effects
- [ ] **Shadows & Lighting** - Dynamic shadows, multiple light types
- [ ] **Post-Processing** - Bloom, SSAO, DOF effects
- [ ] **Skeletal Animation** - Character rigging system
- [ ] **Terrain System** - Heightmap-based terrain
- [ ] **Occlusion Culling** - Frustum and portal culling
- [ ] **Level-of-Detail (LOD)** - Dynamic mesh simplification

### Editor Tools
- [ ] **Scene Graph Editor** - Visual node hierarchy
- [ ] **WYSIWYG Scene Builder** - Ogitor-style level editor
- [ ] **Transform Gizmos** - Move/rotate/scale handles
- [ ] **Snap & Grid System** - Precise object placement
- [ ] **Undo/Redo System** - Full edit history
- [ ] **Asset Importer** - Blender2OGRE-style exporters
- [ ] **Mesh Optimizer** - Automatic mesh optimization
- [ ] **Collision Editor** - Physics collider visualization
- [ ] **Material Editor** - Visual shader editor
- [ ] **Particle Editor** - Visual particle system designer

### Asset Pipeline
- [ ] **Model Import** - Assimp mesh loading
- [ ] **Texture Pipeline** - Automatic mipmap generation
- [ ] **Animation Import** - Skeletal animation support
- [ ] **Audio Pipeline** - Sound bank processing
- [ ] **Bundle System** - Package assets into containers
- [ ] **LOD Generator** - Automatic level-of-detail creation
- [ ] **Atlas Generator** - Texture atlas packing

### Game Systems
- [ ] **Physics Engine** - Rigid body, joints, ragdolls
- [ ] **Collision Detection** - Broadphase/narrowphase
- [ ] **Navigation Mesh** - Pathfinding integration
- [ ] **Sound System** - 3D audio, reverb zones
- [ ] **Input System** - Keyboard/mouse/gamepad
- [ ] **Scripting** - Visual or embedded scripting

### AI Integration
- [ ] **Ghidra MCP Server** - Full binary analysis
- [ ] **AI Command Interface** - Natural language to actions
- [ ] **Code Generation** - AI-assisted feature creation
- [ ] **Mod Analysis** - AI-powered mod compatibility
- [ ] **Auto-Documentation** - Generate docs from binary analysis

### Platform Support
- [ ] **Multi-platform Build** - Windows, Linux, macOS
- [ ] **Mobile Export** - Android, iOS
- [ ] **Web Export** - WebAssembly/WebGL
- [ ] **Console Export** - PS5, Xbox, Switch

### Version Control
- [ ] **Binary Diff** - Visual diff for binary files
- [ ] **Conflict Resolution** - Merge tool for binaries
- [ ] **Asset Locking** - Prevent concurrent edits
- [ ] **Branch Support** - Multiple mod versions
- [ ] **Commit History** - Full VCS for assets

### Documentation
- [ ] **Format Specification** - Document each binary format
- [ ] **API Documentation** - OpenAPI/Swagger docs
- [ ] **Tutorial System** - Interactive tutorials
- [ ] **Sample Projects** - Example mods and workflows
- [ ] **Video Documentation** - YouTube tutorial series

---

## Reference Engines

Inspired by these open-source projects:

### OGRE (OGRECave)
- High-performance 3D rendering backend
- Cross-platform (OpenGL, Vulkan, DirectX, Metal)
- PBR rendering pipeline
- Scene management system

### Godot Engine
- WYSIWYG scene editing
- Node-based architecture
- Visual scripting
- Multi-platform export
- Asset pipeline integration

### Bevy Engine
- Data-oriented architecture (ECS)
- Rust-native performance
- Modular plugin system
- Hot reloading

### sbox (Facepunch)
- Real-time asset editing
- Server-authoritative replication
- Rapid iteration workflow

### Stride 3D
- Modern rendering pipeline
- C# game scripting
- Visual materials editor
- Cross-platform deployment

---

## Roadmap

### Phase 1: Core (Current)
- [x] Binary parsing/compilation
- [x] Basic editor UI
- [x] HTTP API
- [ ] **TODO**: Fix JavaScript execution in Tauri

### Phase 2: 3D Editor
- [ ] WebGL rendering engine
- [ ] Scene graph editor
- [ ] Transform gizmos
- [ ] Asset browser

### Phase 3: Asset Pipeline
- [ ] Model import (Assimp)
- [ ] Texture processing
- [ ] Collision generation

### Phase 4: Game Systems
- [ ] Physics integration
- [ ] Navigation mesh
- [ ] Scripting system

### Phase 5: AI Integration
- [ ] Ghidra MCP server
- [ ] Natural language commands
- [ ] Auto-mod generation

---

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development guidelines.

## License

MIT License - See [LICENSE](./LICENSE)
