//! Athanor Core - Binary format assembly, disassembly, and compilation engine
//! 
//! Full-stack architecture: Rust backend + Bun frontend

use axum::{
    routing::{get, post},
    Json, Router, response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::info;

use athanor_core::{Compiler, CompileRequest as CoreCompileRequest, Modification, ModificationOp};

// API request/response types
#[derive(Deserialize)]
struct ParseRequest {
    path: String,
}

#[derive(Deserialize)]
struct ApiCompileRequest {
    input_path: String,
    output_path: String,
    format: String,
    modifications: Option<Vec<Modification>>,
}

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

/// Build the full Axum router
fn build_router() -> Router {
    Router::new()
        .route("/api/formats", get(list_formats))
        .route("/api/files", get(list_files))
        .route("/api/parse", post(parse_file_handler))
        .route("/api/disassemble", post(disassemble_handler))
        .route("/api/compile", post(compile_handler))
        .route("/api/health", get(health_check))
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
        serde_json::json!({"ext": ".bus", "name": "AUD", "desc": "Audio Bus Definitions"}),
        serde_json::json!({"ext": ".comp", "name": "COMP", "desc": "Compositor Effects"}),
        serde_json::json!({"ext": ".pbr", "name": "PBR", "desc": "PBR Material Definitions"}),
        serde_json::json!({"ext": ".plgn", "name": "PLGN", "desc": "Plugin Manifests"}),
        serde_json::json!({"ext": ".save", "name": "SAVE", "desc": "Save Game Structure"}),
        serde_json::json!({"ext": ".pipe", "name": "PIPE", "desc": "Content Pipeline"}),
    ];
    
    Json(ApiResponse {
        success: true,
        data: Some(formats),
        error: None,
    })
}

async fn list_files() -> impl IntoResponse {
    let game_dir = std::env::var("XMG2_GAME").unwrap_or_else(|_| "D:/My Games/X-Men Legends II Rise of Apocalypse".to_string());
    let mut files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                files.push(serde_json::json!({
                    "path": path.to_string_lossy(),
                    "name": entry.file_name().to_string_lossy(),
                    "size": entry.metadata().map(|m| m.len()).unwrap_or(0),
                    "ext": format!(".{}", ext),
                }));
            }
        }
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(files),
        error: None,
    })
}

async fn parse_file_handler(Json(req): Json<ParseRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    match athanor_core::parser::parse_file(&path) {
        Ok(parsed) => {
            Json(ApiResponse::<serde_json::Value> {
                success: true,
                data: Some(serde_json::to_value(parsed).unwrap_or_default()),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse::<serde_json::Value> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn disassemble_handler(Json(req): Json<ParseRequest>) -> impl IntoResponse {
    let path = PathBuf::from(&req.path);
    match athanor_core::parser::disassemble(&path) {
        Ok(parsed) => {
            Json(ApiResponse::<serde_json::Value> {
                success: true,
                data: Some(serde_json::to_value(parsed).unwrap_or_default()),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse::<serde_json::Value> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn compile_handler(Json(req): Json<ApiCompileRequest>) -> impl IntoResponse {
    let compile_req = CoreCompileRequest {
        input_path: PathBuf::from(&req.input_path),
        output_path: PathBuf::from(&req.output_path),
        format: req.format,
        modifications: req.modifications,
    };
    
    let compiler = Compiler::new();
    match compiler.compile(&compile_req) {
        Ok(result) => {
            Json(ApiResponse::<serde_json::Value> {
                success: true,
                data: Some(serde_json::to_value(result).unwrap_or_default()),
                error: None,
            })
        }
        Err(e) => {
            Json(ApiResponse::<serde_json::Value> {
                success: false,
                data: None,
                error: Some(e.to_string()),
            })
        }
    }
}

async fn health_check() -> impl IntoResponse {
    Json(ApiResponse::<serde_json::Value> {
        success: true,
        data: Some(serde_json::json!({"status": "healthy", "engine": "Athanor", "version": "0.1.0"})),
        error: None,
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let port = std::env::var("ATHANOR_PORT").unwrap_or_else(|_| "3459".to_string());
    let addr = format!("127.0.0.1:{}", port);
    
    let app = build_router();
    
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Athanor engine listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}