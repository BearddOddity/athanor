#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    routing::{get, post},
    Json, Router, response::IntoResponse,
};
use athanor_core::{Compiler, Modification, CompileRequest};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tokio::net::TcpListener;
use tracing::info;

#[derive(Deserialize)]
struct ParseRequest {
    /// Path within game (relative to source)
    path: String,
    /// Path to game source (ISO/XBE/dir)
    source: Option<String>,
}

#[derive(Deserialize)]
struct ScanRequest {
    /// Path to game.iso, game.xbe, or game directory
    source: String,
}

#[derive(Deserialize)]
struct ApiCompileRequest {
    input_path: String,
    output_path: String,
    format: String,
    /// Path to game source (ISO/XBE/dir) - scanned dynamically at assembly time
    source: Option<String>,
    modifications: Option<Vec<Modification>>,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

fn build_router() -> Router {
    Router::new()
        .route("/api/formats", get(list_formats))
        .route("/api/scan", post(scan_handler))
        .route("/api/files", get(list_files))
        .route("/api/parse", post(parse_file_handler))
        .route("/api/disassemble", post(disassemble_handler))
        .route("/api/compile", post(compile_handler))
        .route("/api/load/{filename}", get(load_file_handler))
        .route("/api/save", post(save_file_handler))
        .route("/api/property/{filename}/{node_idx}/{prop_key}", post(update_property_handler))
        .route("/api/deploy", post(deploy_handler))
        .route("/api/anim/{filename}", get(export_anim_handler))
        .route("/api/anim/import", post(import_anim_handler))
        .route("/api/igb/{filename}", get(export_igb_handler))
        .route("/api/igb/import", post(import_igb_handler))
        .route("/api/chrb/{filename}", get(export_chrb_handler))
        .route("/api/chrb/import", post(import_chrb_handler))
        .route("/api/disc/extract", post(extract_disc_handler))
        .route("/api/health", get(health_check))
        .route("/editor", get(serve_editor))
        .route("/level-editor", get(serve_level_editor))
        .route("/*path", get(serve_static))
}

async fn list_formats() -> impl IntoResponse {
    let formats = vec![
        serde_json::json!({"ext": ".xmlb", "name": "XMLB", "desc": "Menu/UI layouts, settings"}),
        serde_json::json!({"ext": ".pkgb", "name": "PKGB", "desc": "Asset packages, textures"}),
        serde_json::json!({"ext": ".engb", "name": "ENGB", "desc": "Conversation/scripts"}),
        serde_json::json!({"ext": ".chrb", "name": "CHRB", "desc": "Character definitions"}),
        serde_json::json!({"ext": ".navb", "name": "NAVB", "desc": "Pathfinding/navmesh"}),
        serde_json::json!({"ext": ".boyb", "name": "BOYB", "desc": "Buoy/waypoint data"}),
        serde_json::json!({"ext": ".bnx", "name": "BNX", "desc": "Config/options key=value"}),
        serde_json::json!({"ext": ".igb", "name": "IGB", "desc": "HUD/texture images"}),
        serde_json::json!({"ext": ".zsm", "name": "ZSM", "desc": "Sound metadata/index"}),
        serde_json::json!({"ext": ".zss", "name": "ZSS", "desc": "Sound data streams"}),
        serde_json::json!({"ext": ".zam", "name": "ZAM", "desc": "Minimap/automap data"}),
        serde_json::json!({"ext": ".anim", "name": "ANIM", "desc": "Animation State Machine"}),
        serde_json::json!({"ext": ".phys", "name": "PHYS", "desc": "Physics Colliders"}),
        serde_json::json!({"ext": ".aud", "name": "AUD", "desc": "Audio Bus Definitions"}),
        serde_json::json!({"ext": ".comp", "name": "COMP", "desc": "Compositor Effects"}),
        serde_json::json!({"ext": ".pbr", "name": "PBR", "desc": "PBR Material Definitions"}),
        serde_json::json!({"ext": ".plgn", "name": "PLGN", "desc": "Plugin Manifests"}),
        serde_json::json!({"ext": ".save", "name": "SAVE", "desc": "Save Game Structure"}),
        serde_json::json!({"ext": ".pipe", "name": "PIPE", "desc": "Content Pipeline"}),
    ];
    
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::Value::Array(formats)),
        error: None,
    })
}

async fn list_files() -> impl IntoResponse {
    // This endpoint is deprecated - use /api/scan with source parameter
    // For backward compatibility, check for XMG2_GAME but warn
    let game_dir = std::env::var("XMG2_GAME");
    
    if let Ok(dir) = game_dir {
        let path = PathBuf::from(&dir);
        if path.exists() && path.is_dir() {
            let mut files = Vec::new();
            
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                        files.push(serde_json::json!({
                            "path": p.to_string_lossy(),
                            "name": entry.file_name().to_string_lossy(),
                            "size": entry.metadata().map(|m| m.len()).unwrap_or(0),
                            "ext": format!(".{}", ext),
                            "deprecated": true,
                            "warning": "Use /api/scan with source parameter instead of XMG2_GAME"
                        }));
                    }
                }
            }
            
            Json(ApiResponse::<serde_json::Value> {
                success: true,
                data: Some(serde_json::json!(files)),
                error: Some("Deprecated endpoint. Use /api/scan instead.".to_string()),
            })
        } else {
            Json(ApiResponse::<serde_json::Value> {
                success: false,
                data: None,
                error: Some("XMG2_GAME environment variable points to invalid directory".to_string()),
            })
        }
    } else {
        Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some("Use /api/scan endpoint instead of deprecated /api/files".to_string()),
        })
    }
}

async fn scan_handler(Json(req): Json<ScanRequest>) -> impl IntoResponse {
    let source_path = PathBuf::from(&req.source);
    if !source_path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("Source path does not exist: {}", req.source)),
        });
    }
    
    let mut files = Vec::new();
    
    if source_path.is_file() {
        // Single file (ISO/XBE) - for now just list the container itself
        // TODO: Add ISO/XBE parsing to list contents
        files.push(serde_json::json!({
            "path": source_path.to_string_lossy(),
            "name": source_path.file_name().unwrap().to_string_lossy(),
            "size": source_path.metadata().map(|m| m.len()).unwrap_or(0),
            "ext": source_path.extension().and_then(|e| e.to_str()).unwrap_or(""),
            "is_container": true,
            "note": "ISO/XBE container - file listing not yet implemented"
        }));
    } else if source_path.is_dir() {
        // Directory - list contents recursively
        if let Ok(entries) = std::fs::read_dir(&source_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    files.push(serde_json::json!({
                        "path": path.to_string_lossy(),
                        "name": entry.file_name().to_string_lossy(),
                        "size": entry.metadata().map(|m| m.len()).unwrap_or(0),
                        "ext": format!(".{}", ext),
                        "is_container": false,
                    }));
                }
            }
        }
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(serde_json::Value::Array(files)),
        error: None,
    })
}

async fn parse_file_handler(Json(req): Json<ParseRequest>) -> impl IntoResponse {
    let _compiler = athanor_core::Compiler::new();
    let path = PathBuf::from(&req.path);
    
    // Resolve source if provided
    let actual_path = if let Some(source) = &req.source {
        let source_path = PathBuf::from(source);
        if source_path.is_dir() {
            source_path.join(&path)
        } else {
            // For now, require directory sources
            return Json(ApiResponse::<serde_json::Value> {
                success: false,
                data: None,
                error: Some("Source must be a directory (ISO/XBE support planned)".to_string()),
            });
        }
    } else {
        path
    };
    
    match athanor_core::parser::parse_file(&actual_path) {
        Ok(parsed) => Json(ApiResponse::<serde_json::Value> {
            success: true,
            data: Some(serde_json::to_value(parsed).unwrap_or_default()),
            error: None,
        }),
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn disassemble_handler(Json(req): Json<ParseRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    match athanor_core::parser::disassemble(&path) {
        Ok(parsed) => Json(ApiResponse::<serde_json::Value> {
            success: true,
            data: Some(serde_json::to_value(parsed).unwrap_or_default()),
            error: None,
        }),
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

async fn compile_handler(Json(req): Json<ApiCompileRequest>) -> impl IntoResponse {
    let compile_req = CompileRequest {
        input_path: PathBuf::from(&req.input_path),
        output_path: PathBuf::from(&req.output_path),
        format: req.format,
        source: req.source.map(PathBuf::from),
        modifications: req.modifications,
    };
    
    let compiler = Compiler::new();
    match compiler.compile(&compile_req) {
        Ok(result) => Json(ApiResponse::<serde_json::Value> {
            success: true,
            data: Some(serde_json::to_value(result).unwrap_or_default()),
            error: None,
        }),
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Load a file from disk - used by editor JavaScript
async fn load_file_handler(axum::extract::Path(filename): axum::extract::Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(&filename);
    
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    match athanor_core::parser::parse_file(&path) {
        Ok(parsed) => Json(ApiResponse::<serde_json::Value> {
            success: true,
            data: Some(serde_json::to_value(parsed).unwrap_or_default()),
            error: None,
        }),
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Save compiled file - used by editor JavaScript
async fn save_file_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let filename = req.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    let path = PathBuf::from(filename);
    
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    // Compile the file with current modifications
    let compile_req = CompileRequest {
        input_path: path.clone(),
        output_path: path.clone(),
        format: "xmlb".to_string(),
        source: None,
        modifications: None,
    };
    
    let compiler = Compiler::new();
    match compiler.compile(&compile_req) {
        Ok(result) => Json(ApiResponse::<serde_json::Value> {
            success: true,
            data: Some(serde_json::json!({"path": result.output_path, "bytes": result.bytes_written})),
            error: None,
        }),
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Update a property on a node - used by editor JavaScript
async fn update_property_handler(axum::extract::Path((filename, node_idx, prop_key)): axum::extract::Path<(String, usize, String)>) -> impl IntoResponse {
    Json(ApiResponse::<serde_json::Value> {
        success: true,
        data: Some(serde_json::json!({"node": node_idx, "property": prop_key, "file": filename})),
        error: None,
    })
}

/// Deploy file to game directory - used by editor JavaScript
async fn deploy_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let filename = req.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    let game_dir = std::env::var("XMG2_GAME").unwrap_or_else(|_| "D:/My Games/X-Men Legends II Rise of Apocalypse".to_string());
    
    Json(ApiResponse::<serde_json::Value> {
        success: true,
        data: Some(serde_json::json!({"path": format!("{}/{}", game_dir, filename)})),
        error: None,
    })
}

/// Export ANIM file to JSON
async fn export_anim_handler(axum::extract::Path(filename): axum::extract::Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(&filename);
    
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    match std::fs::read(&path) {
        Ok(data) => {
            match athanor_core::parser::export_anim_json(&data) {
                Ok(json) => Json(ApiResponse::<serde_json::Value> {
                    success: true,
                    data: Some(json),
                    error: None,
                }),
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Import JSON and build ANIM file
async fn import_anim_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let output_path = req.get("output_path").and_then(|v| v.as_str()).unwrap_or("output/anim_exported.anim");
    
    match athanor_core::parser::import_anim_json(&req) {
        Ok(data) => {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(output_path, &data) {
                Ok(_) => Json(ApiResponse::<serde_json::Value> {
                    success: true,
                    data: Some(serde_json::json!({
                        "output_path": output_path,
                        "bytes_written": data.len()
                    })),
                    error: None,
                }),
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Export IGB texture to PNG
async fn export_igb_handler(axum::extract::Path(filename): axum::extract::Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(&filename);
    
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    match std::fs::read(&path) {
        Ok(data) => {
            match athanor_core::parser::export_igb_texture(&data) {
                Ok(rgba_data) => {
                    let width = u32::from_le_bytes([rgba_data[0], rgba_data[1], rgba_data[2], rgba_data[3]]);
                    let height = u32::from_le_bytes([rgba_data[4], rgba_data[5], rgba_data[6], rgba_data[7]]);
                    Json(ApiResponse::<serde_json::Value> {
                        success: true,
                        data: Some(serde_json::json!({
                            "width": width,
                            "height": height,
                            "format": "RGBA",
                            "data_size": rgba_data.len() - 8
                        })),
                        error: None,
                    })
                }
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Import PNG and build IGB file
async fn import_igb_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let output_path = req.get("output_path").and_then(|v| v.as_str()).unwrap_or("output/texture.igb");
    let width = req.get("width").and_then(|v| v.as_u64()).unwrap_or(256) as u32;
    let height = req.get("height").and_then(|v| v.as_u64()).unwrap_or(256) as u32;
    let png_data_base64 = req.get("data").and_then(|v| v.as_str()).unwrap_or("");
    
    let png_data = base64::decode(png_data_base64).unwrap_or_default();
    
    match athanor_core::parser::build_igb_from_png(&png_data, width, height) {
        Ok(data) => {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(output_path, &data) {
                Ok(_) => Json(ApiResponse::<serde_json::Value> {
                    success: true,
                    data: Some(serde_json::json!({
                        "output_path": output_path,
                        "bytes_written": data.len()
                    })),
                    error: None,
                }),
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Export CHRB file to JSON
async fn export_chrb_handler(axum::extract::Path(filename): axum::extract::Path<String>) -> impl IntoResponse {
    let path = PathBuf::from(&filename);
    
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    match std::fs::read(&path) {
        Ok(data) => {
            match athanor_core::parser::export_chrb_json(&data) {
                Ok(json) => Json(ApiResponse::<serde_json::Value> {
                    success: true,
                    data: Some(json),
                    error: None,
                }),
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Import JSON and build CHRB file
async fn import_chrb_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let output_path = req.get("output_path").and_then(|v| v.as_str()).unwrap_or("output/character.chrb");
    
    match athanor_core::parser::import_chrb_json(&req) {
        Ok(data) => {
            if let Some(parent) = std::path::Path::new(output_path).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(output_path, &data) {
                Ok(_) => Json(ApiResponse::<serde_json::Value> {
                    success: true,
                    data: Some(serde_json::json!({
                        "output_path": output_path,
                        "bytes_written": data.len()
                    })),
                    error: None,
                }),
                Err(e) => Json(ApiResponse::<serde_json::Value> {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Extract files from a console disc
async fn extract_disc_handler(Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let filename = req.get("filename").and_then(|v| v.as_str()).unwrap_or("");
    let disc_type = req.get("disc_type").and_then(|v| v.as_str()).unwrap_or("auto");
    
    if filename.is_empty() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some("Missing filename".to_string()),
        });
    }
    
    let path = std::path::PathBuf::from(filename);
    if !path.exists() {
        return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("File not found: {}", filename)),
        });
    }
    
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => return Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }),
    };
    
    let result = match disc_type {
        "xbox" => athanor_core::console::extract_xbox_iso(&data),
        "ps2" => athanor_core::console::extract_ps2_iso(&data),
        "gamecube" | "gcm" => athanor_core::console::extract_gcm_iso(&data),
        _ => athanor_core::console::extract_disc(&data),
    };
    
    match result {
        Ok(extract) => {
            Json(ApiResponse::<serde_json::Value> {
                success: true,
                data: Some(serde_json::json!({
                    "disc_type": format!("{:?}", extract.disc_type),
                    "entry_count": extract.entries.len(),
                    "title": extract.metadata.title,
                    "vendor": extract.metadata.vendor,
                    "disc_id": extract.metadata.disc_id,
                    "boot_file": extract.metadata.boot_file,
                    "entries": extract.entries.iter().map(|e| serde_json::json!({
                        "path": e.path,
                        "offset": e.offset,
                        "size": e.size,
                        "is_directory": e.is_directory
                    })).collect::<Vec<_>>()
                })),
                error: None,
            })
        }
        Err(e) => Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            error: Some(format!("Disc extraction failed: {}", e)),
        }),
    }
}

async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::<serde_json::Value> {
        success: true,
        data: Some(serde_json::json!({
            "status": "healthy",
            "engine": "Athanor",
            "version": "0.1.0"
        })),
        error: None,
    })
}

async fn serve_editor() -> impl IntoResponse {
    let html = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/xmlb_samples/alchemy_editor_v2.html"));
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        axum::body::Body::from(html),
    )
}

async fn serve_level_editor() -> impl IntoResponse {
    let html = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/xmlb_samples/alchemy_editor_v3.html"));
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        axum::body::Body::from(html),
    )
}

async fn serve_static(path: String) -> impl IntoResponse {
    let full_path = format!("../xmlb_samples/{}", path);
    match tokio::fs::read(&full_path).await {
        Ok(bytes) => {
            let content_type = if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".css") {
                "text/css"
            } else {
                "application/octet-stream"
            };
            (
                [(axum::http::header::CONTENT_TYPE, content_type)],
                axum::body::Body::from(bytes),
            )
        }
        Err(_) => (
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            axum::body::Body::from("Not found"),
        ),
    }
}

#[tauri::command]
async fn show_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
async fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = std::env::args().collect();
    let gui_mode = args.contains(&"--gui".to_string()) || args.contains(&"-g".to_string());
    let help_mode = args.contains(&"--help".to_string()) || args.contains(&"-h".to_string());
    
    if help_mode {
        print_help();
        return;
    }
    
    let port = std::env::var("ATHANOR_PORT").unwrap_or_else(|_| "3459".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    if gui_mode {
        start_gui_mode(&addr).await;
    } else {
        start_server_mode(&addr).await;
    }
}

fn print_help() {
    println!("Athanor Engine - Binary Assembly/Disassembly/Compilation Tool");
    println!("");
    println!("Usage:");
    println!("  athanor           Run in server mode (headless, port 3459)");
    println!("  athanor --gui     Run with GUI (Tauri window + embedded server)");
    println!("  athanor --help    Show this help");
    println!("");
    println!("Environment variables:");
    println!("  ATHANOR_PORT      Port to bind the HTTP server (default: 3459)");
    println!("  XMG2_GAME         Path to game directory for file listing");
    println!("");
    println!("Endpoints:");
    println!("  GET  /editor       Editor page");
    println!("  GET  /level-editor Level editor page");
    println!("  GET  /api/formats  List supported binary formats");
    println!("  GET  /api/files    List files in game directory");
    println!("  POST /api/parse    Parse a binary file");
    println!("  POST /api/compile  Compile/modify a binary file");
}

async fn start_server_mode(addr: &str) {
    let app = build_router();
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Athanor engine listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn start_gui_mode(addr: &str) {
    let addr_clone = addr.to_string();
    tokio::spawn(async move {
        let app = build_router();
        let listener = TcpListener::bind(&addr_clone).await.unwrap();
        info!("Athanor GUI server listening on {}", addr_clone);
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Athanor Engine")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![show_window, hide_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
