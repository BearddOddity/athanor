# Athanor Engine - Session Summary

**Date**: 2026-09-06  
**Repository**: https://github.com/BearddOddity/athanor  
**Location**: `D:\My apps\Athanor`

---

## Session Overview

This session established the foundation for the Athanor Engine - a Rust-based game modification platform specifically designed for X-Men Legends II: Rise of Apocalypse and sibling titles, with the ultimate goal of enabling console game decompilation and PC porting.

---

## What Was Done

### 1. Initial Setup
- Copied Athanor repo from `D:\re-lab-share\xmen2_mod\athanor-core` to `D:\My apps\Athanor`
- Set up Godot .gitignore patterns for Anchorpoint integration
- Built release binary (9.6 MB)

### 2. Architecture Refactor
- Refactored from single binary to Tauri GUI + CLI architecture
- Added `--gui` flag for windowed mode
- Kept headless mode for AI integration
- Moved source from `src/` to `src-tauri/src/` (Tauri convention)

### 3. Tauri 2 Integration
- Added Tauri 2 with `tray-icon` and `devtools` features
- Created `tauri.conf.json` with window configuration
- Generated app icon using PowerShell/System.Drawing
- Configured CSP for webview

### 4. Editor Implementation
- **V2 Editor** (`alchemy_editor_v2.html`): Godot-style 3-panel layout
  - Scene tree browser
  - AST tree view  
  - Hex view
  - Property inspector
- **V3 Level Editor** (`alchemy_editor_v3.html`): WebGL 3D viewport
  - Object hierarchy
  - Transform tools
  - Camera controls

### 5. Documentation
- Created comprehensive `FEATURES.md` with:
  - Full system architecture (9 subsystems)
  - ECS (Bevy-inspired) component system
  - Signal/Event (Godot-inspired) system
  - Plugin architecture
  - Console decompilation targets
  - PC recompilation workflow
  - Feature conflict resolution
- Updated `README.md` with current architecture

### 6. Feature Scope Definition
Based on analysis of OGRE, Godot, Bevy, sbox, and Stride:
- Defined 9 core subsystems
- Documented 100+ features
- Created dependency matrix
- Resolved potential conflicts
- Added console porting priority matrix

---

## Current State

### Working
| Feature | Status |
|---------|--------|
| HTTP API Server | Running on port 3459 |
| Binary Parsing | 19 formats supported |
| Editor HTML Serving | Both V2 and V3 |
| Headless Mode | `athanor.exe` works |
| GUI Mode | Window opens, UI renders |
| Git Repository | Pushed to GitHub |

### Broken/Not Working
| Feature | Issue |
|---------|-------|
| **Tauri WebView JS** | JavaScript not executing - buttons don't work |
| System Tray | Close-to-hide not fully wired |

### Known Issues
1. **JavaScript in Tauri WebView** - This is the blocking issue. The UI loads visually but JavaScript click handlers aren't firing. Likely a CSP or webview configuration issue.

---

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/main.rs` | Entry point, CLI args, server setup |
| `src-tauri/src/lib.rs` | Core library exports |
| `src-tauri/src/parser.rs` | Binary format parsers |
| `src-tauri/src/compiler.rs` | Compilation engine |
| `src-tauri/tauri.conf.json` | Tauri window config |
| `xmlb_samples/*.html` | Editor interfaces |
| `FEATURES.md` | Full feature documentation |
| `README.md` | Quick reference |

---

## Console Decompilation Targets

### Priority Games
1. **X-Men Legends II: Rise of Apocalypse** (PS2, Xbox, GameCube, PC)
2. **Marvel: Ultimate Alliance** (PS2, Xbox, Wii, PC)
3. **Marvel: Ultimate Alliance 2** (PS3, Xbox 360, Wii, PC)
4. **X-Men Legends (Xbox)** → PC port target

### Porting Workflow
```
Extract → Analyze → Decompile → Convert → Recompile → Enhance → Distribute
```

---

## Next Steps (Priority Order)

### 1. Fix Tauri WebView JavaScript (BLOCKING)
- Debug CSP configuration
- Check webview permissions
- Test with simpler JavaScript

### 2. Wire Editor to Backend
- Connect UI buttons to HTTP API calls
- Implement file browser functionality
- Add parse/disassemble/compile UI flow

### 3. System Tray Implementation
- Add tray icon with menu
- Implement close-to-hide behavior
- Add "Show Window" / "Quit" options

### 4. Console Binary Analysis
- Set up Ghidra MCP server connection
- Begin X-Men Legends II executable analysis
- Document format structures

### 5. Texture Extraction
- Implement IGB format parser fully
- Add texture export capability
- Build texture replacement pipeline

---

## Commands Reference

```bash
# Build
cargo build --release --manifest-path "D:\My apps\Athanor\Cargo.toml"

# Run headless
.\target\release\athanor.exe

# Run GUI
.\target\release\athanor.exe --gui

# Help
.\target\release\athanor.exe --help

# API test
curl http://127.0.0.1:3459/api/health
```

---

## Git Commands

```bash
# Commit
git -C "D:\My apps\Athanor" add -A
git -C "D:\My apps\Athanor" commit -m "message"
git -C "D:\My apps\Athanor" push origin master
```

---

## Questions to Answer in Next Session

1. **Why isn't JavaScript executing in Tauri webview?**
   - CSP issue? Permission issue? WebView2 config?

2. **What is the simplest test case?**
   - Create minimal HTML with one button to verify JS works

3. **Should we switch to different approach?**
   - Use embedded HTML with inline JavaScript
   - Serve from different endpoint
   - Use Tauri built-in HTML serving

---

## Useful Links

- [Tauri 2 Docs](https://tauri.app/)
- [Tauri WebView Config](https://tauri.app/reference/config/)
- [Ghidra MCP](https://github.com/re-lab-tools/ghidra-mcp)
- [Anchorpoint](https://www.anchorpoint.app/)

---

**End of Session Summary**
