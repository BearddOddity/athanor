# Athanor

**Full-stack binary assembly, disassembly, and compilation engine for game formats**

Athanor (recreated from Alchemy Engine) is a high-performance Rust backend + Bun frontend system for parsing, modifying, and recompiling Xbox and PC game binary formats. Designed for the X-Men Legends II Rise of Apocalypse modding community.

## Features

- **19+ Binary Formats**: XMLB, PKGB, ENGB, CHRB, NAVB, BOYB, BNX, IGB, ZSM, ZSS, ZAM, ANIM, PHYS, AUD, COMP, PBR, PLGN, SAVE, PIPE
- **Full Assembly/Disassembly**: Parse binary → modify nodes → compile/repack
- **Axum HTTP Server**: REST API for file parsing, disassembly, and compilation
- **Bun Frontend**: Modern web editor at `/editor` endpoint
- **Ghidra MCP Integration**: Headless binary analysis via 37-tool Ghidra MCP server
- **Anchorpoint.app Integration**: Git-based version control for binary game assets
- **Cross-Title Analysis**: Function ID (FID) databases for identifying shared engine code
- **Batch AI Training**: Extract decompiled functions with context for ML model training

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Bun Server    │────▶│  Axum HTTP API   │────▶│  Rust Backend   │
│   Port 3457     │     │  Port 3459       │     │  athanor-core   │
└─────────────────┘     └──────────────────┘     └─────────────────┘
       │                        │                        │
       ▼                        ▼                        ▼
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  /editor UI     │     │  /api/parse      │     │  Ghidra MCP     │
│  /api/formats   │     │  /api/compile    │     │  Anchorpoint    │
│  /api/files     │     │  /api/disassemble│     │  Batch Training │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## Quick Start

### Build

```bash
cd athanor-core
cargo build --release
```

### Run Server

```bash
# Start Rust backend (port 3459)
cd athanor-core && ./target/release/athanor

# Start Bun server (port 3457)
cd ../xmlb_samples && bun run alchemy_server.ts
```

### Web Editor

Open `http://localhost:3457/editor` in your browser.

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/formats` | List supported formats |
| GET | `/api/files` | List game files |
| POST | `/api/parse` | Parse binary file |
| POST | `/api/disassemble` | Disassemble binary |
| POST | `/api/compile` | Compile with modifications |
| GET | `/api/health` | Health check |

## Supported Formats

| Extension | Name | Description |
|-----------|------|-------------|
| `.xmlb` | XMLB | Menu/UI layouts, settings |
| `.pkgb` | PKGB | Asset packages, textures |
| `.engb` | ENGB | Conversation/scripts |
| `.chrb` | CHRB | Character definitions |
| `.navb` | NAVB | Pathfinding/navmesh |
| `.boyb` | BOYB | Buoy/waypoint data |
| `.bnx` | BNX | Config/options key=value |
| `.igb` | IGB | HUD/texture images |
| `.zsm` | ZSM | Sound metadata/index |
| `.zss` | ZSS | Sound data streams |
| `.zam` | ZAM | Minimap/automap data |
| `.anim` | ANIM | Animation State Machine |
| `.phys` | PHYS | Physics Colliders |
| `.bus` | AUD | Audio Bus Definitions |
| `.comp` | COMP | Compositor Effects |
| `.pbr` | PBR | PBR Material Definitions |
| `.plgn` | PLGN | Plugin Manifests |
| `.save` | SAVE | Save Game Structure |
| `.pipe` | PIPE | Content Pipeline |

## Anchorpoint Integration

Athanor integrates with Anchorpoint.app for Git-based version control of binary game assets:

- **File Locking**: Prevent merge conflicts on binary files
- **Asset Metadata**: Tags, descriptions, review status
- **Review Workflow**: Submit for review, approval process
- **Git LFS**: Automatic large file storage configuration
- **Sparse Checkout**: Work with TB-sized repositories

## Ghidra MCP Integration

Connects to the re-lab-tools Ghidra MCP server for headless binary analysis:

- **37 Tools**: Full Ghidra MCP toolset
- **Xbox/XBE Analysis**: Specialized scripts for Xbox binaries
- **Symbol Database**: Apply XbSymbolDatabase signatures
- **Function ID**: Cross-title function matching
- **RTTI Analysis**: Walk MSVC RTTI for vtables
- **Batch Decompilation**: AI training data extraction

## License

MIT
