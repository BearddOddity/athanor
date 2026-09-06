# Athanor Engine

**Full-stack binary assembly, disassembly, and compilation engine for game formats**

Athanor (recreated from Alchemy Engine) is a Rust-based system for parsing, modifying, and recompiling game binary formats. Designed for the X-Men Legends II Rise of Apocalypse modding community with the goal of enabling console game decompilation and PC porting.

## Primary Goals
 
1. **Modding Support**: Modify X-Men Legends II and sibling titles (MUA 1, MUA 2)
2. **Console Decompilation**: Decompile console-exclusive shelved games
3. **PC Recompilation**: Recompile for modern PC with enhancements
4. **Bring Your Own Game**: Specify game ISO/XBE/directory per request, never cached

## Target Games

| Game | Status |
|------|--------|
| X-Men Legends II: Rise of Apocalypse | Primary target - formats analyzed |
| Marvel: Ultimate Alliance | Similar formats - future |
| Marvel: Ultimate Alliance 2 | Similar formats - future |
| X-Men Legends (Xbox) | Console-only, planned for PC port |

## Quick Start

### Build

```bash
cd D:\My apps\Athanor
cargo build --release
```

### Run

```bash
# Headless mode (for AI integration)
.\target\release\athanor.exe

# GUI mode (with Tauri window)
.\target\release\athanor.exe --gui

# Help
.\target\release\athanor.exe --help
```

### Web Editor

Open browser to `http://127.0.0.1:3459/editor` or use the Tauri GUI window.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         ATHANOR ENGINE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐ │
│  │  TAURI GUI     │  │  HTTP SERVER   │  │  BINARY CORE          │ │
│  │  Desktop App   │  │  Port 3459     │  │  20 Formats + Builder  │ │
│  │  (WebView)    │  │  (Axum)       │  │  Parser/Compiler      │ │
│  └────────────────┘  └────────────────┘  └────────────────────────┘ │
│                                                                      │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐ │
│  │  EDITOR UI     │  │  AI CORE      │  │  GAME DATA            │ │
│  │  V2 + V3      │  │  Ghidra MCP   │  │  xmlb_samples/        │ │
│  │  WebGL 3D     │  │  Analysis     │  │  X-Men Legends II     │ │
│  └────────────────┘  └────────────────┘  └────────────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Features

### Binary Format Support (20 formats, all with parsers + builders)

| Format | Extension | Description | Status |
|--------|-----------|-------------|--------|
| XMLB | Menu/UI | Menu layouts, settings | Analyzed |
| PKGB | Package | Game asset bundles | Analyzed |
| ENGB | Engine | Engine configuration | Analyzed |
| BOYB | Boy | Binary object data | Analyzed |
| CHRB | Character | Character definitions | Analyzed |
| NAVB | Nav | Navigation/pathfinding | Analyzed |
| BNX | Config | Key-value configuration | Analyzed |
| IGB | Image | HUD textures, image data | Analyzed |
| ZSM | Sound Index | Sound metadata, bank indices | Analyzed |
| ZSS | String Table | String lookup tables | Analyzed |
| ZAM | Minimap | Automap/waypoint data | Analyzed |
| ANIM | Animation | Animation state machine | Analyzed |
| PHYS | Physics | Collision shapes, physics data | Analyzed |
| AUD | Audio | Audio bus definitions | Analyzed |
| COMP | Composite | Composite data structures | Analyzed |
| PBR | Material | Material definitions | Analyzed |
| PLGN | Level | Level/zone data | Analyzed |
| SAVE | Save | Save game structure | Analyzed |
| PIPE | Pipe | Pipe/connection data | Analyzed |

### Editor Interfaces

- **Asset Editor (V2)**: Godot-style 3-panel layout
  - Scene tree browser
  - AST tree view
  - Hex view
  - Property inspector

- **Level Editor (V3)**: WebGL-based 3D viewport
  - Object hierarchy
  - Transform tools
  - Camera controls

### API Endpoints
 
 | Method | Endpoint | Description |
 |--------|----------|-------------|
 | GET | `/api/formats` | List 20 supported formats |
 | POST | `/api/scan` | Scan game directory at assembly time (Bring Your Own Game) |
 | POST | `/api/parse` | Parse binary file from source path |
 | POST | `/api/compile` | Compile with modifications (output to COPY only) |
 | GET | `/api/health` | Health check |
 | GET | `/editor` | Asset editor UI |
 | GET | `/level-editor` | Level editor UI |
 | GET | `/api/files` | **Deprecated** - use /api/scan |

### Integrations

- **Anchorpoint.app**: Git-based version control for binary assets
- **Ghidra MCP**: Headless binary analysis via 37-tool MCP server
- **Godot .gitignore**: Compatible with game engine version control

## File Structure

```
D:\My apps\Athanor\
├── src-tauri/           # Tauri + Axum application
│   ├── main.rs         # Entry point, CLI args, GUI/headless modes
│   ├── src/            # Core library
│   │   ├── lib.rs      # Library root
│   │   ├── parser.rs   # Binary format parsers
│   │   ├── compiler.rs  # Compilation engine
│   │   ├── formats.rs  # Format definitions
│   │   ├── builder.rs  # Binary builder
│   │   └── anchorpoint_integration.rs
│   ├── Cargo.toml
│   └── tauri.conf.json # Tauri configuration
├── xmlb_samples/       # Editor HTML + game data samples
│   ├── alchemy_editor_v2.html  # Asset editor
│   ├── alchemy_editor_v3.html  # Level editor
│   └── *.XMLB, *.BNX   # Sample game files
├── icons/              # App icons
├── build.rs            # Tauri build script
├── Cargo.toml          # Workspace manifest
├── tauri.conf.json    # Tauri config (root)
├── FEATURES.md         # Full feature documentation
└── README.md           # This file
```

## Environment Variables
 
 | Variable | Default | Description |
 |----------|---------|-------------|
 | `ATHANOR_PORT` | 3459 | HTTP server port |
 | `XMG2_GAME` | — | **Deprecated** - use `source` parameter in API requests |

## Console Porting Workflow

```
1. EXTRACT   - Console Disc → Mount → Extract All Files
2. ANALYZE   - Binary Formats → Ghidra/Radare2 → Document Structures
3. DECOMPILE - Console EXE → Decompiled Code → Symbol Tables
4. CONVERT   - Console Textures → PC Formats → Rebuild Bundles
5. RECOMPILE  - Modified Code → PC EXE → Link with Engine
6. ENHANCE   - Widescreen → 60 FPS → Controller → Achievements
7. DISTRIBUTE - Build → Package → Mod Loader Ready
```

## Status
 
 ### All Issues Resolved ✅
 
 - [x] All 20 format parsers implemented (XMLB, PKGB, ENGB, BOYB, CHRB, NAVB, BNX, IGB, ZSM, ZSS, ZAM, ANIM, PHYS, AUD, COMP, PBR, PLGN, SAVE, PIPE)
 - [x] Binary builders for all 20 formats (round-trip compilation)
 - [x] Bring Your Own Game architecture (ISO/XBE/directory per request)
 - [x] HTTP API endpoints with source parameter support
 - [x] 5 unit tests passing (format detection, XMLB round-trip, XL2 BNX parsing)
 - [x] Clean build with no warnings
 
 ### Known Working
 
 - [x] HTTP API server (Axum)
 - [x] Binary parsing (20 formats)
 - [x] Binary builders (round-trip compilation)
 - [x] Editor HTML serving
 - [x] Headless mode
 - [x] GUI mode (window opens, UI renders)
 - [x] System tray (click to show, close-to-hide)
 - [x] JavaScript execution in Tauri webview
 - [x] Tauri 2 capabilities configured
 - [x] IPC commands (show_window, hide_window)
- [x] /api/scan endpoint (Bring Your Own Game)

## Development
 
 ### Building
 
 ```bash
 # Debug build
 cargo build
 
 # Release build
 cargo build --release
 ```
 
 ### Testing
 
 ```bash
 # Run all tests
 cargo test
 
 # Check without building
 cargo check
 ```
 
 ### Bring Your Own Game Workflow
 
 ```bash
 # Compile a file from a game directory
 POST /api/compile
 {
   "source": "D:/My Games/X-Men Legends II",
   "input_path": "menu/menuscreen.xmlb",
   "modifications": [...],
   "output_path": "output/menuscreen.xmlb"
 }
 ```
 
 ## Contributing
 
 See FEATURES.md for full feature roadmap and system architecture.

## License

MIT

## Links

- [Repository](https://github.com/BearddOddity/athanor)
- [Features](FEATURES.md)
