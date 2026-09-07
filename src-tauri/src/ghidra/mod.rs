//! Ghidra MCP Server
//!
//! Model Context Protocol server for headless Ghidra analysis.
//! Provides RPC-style endpoints for binary analysis, symbol recovery,
//! and function detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RPC request types for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: String,
    pub params: Option<McpParams>,
}

/// MCP method parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpParams {
    #[serde(rename = "binary_path")]
    pub binary_path: Option<String>,
    #[serde(rename = "function_name")]
    pub function_name: Option<String>,
    #[serde(rename = "address")]
    pub address: Option<String>,
    #[serde(rename = "analysis_options")]
    pub analysis_options: Option<HashMap<String, String>>,
    #[serde(rename = "export_format")]
    pub export_format: Option<String>,
}

/// RPC response types for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub result: Option<McpResult>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResult {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub symbols: Option<Vec<SymbolInfo>>,
    pub functions: Option<Vec<FunctionInfo>>,
    pub strings: Option<Vec<StringInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub address: u64,
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub entry_point: u64,
    #[serde(rename = "type")]
    pub func_type: String,
    pub signature: String,
    pub calling_convention: String,
    pub local_variables: Vec<LocalVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVariable {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringInfo {
    pub value: String,
    pub address: u64,
    pub length: usize,
    pub encoding: String,
}

/// Ghidra MCP Server implementation
pub struct GhidraMcpServer {
    pub analysis_results: HashMap<String, AnalysisSession>,
}

pub struct AnalysisSession {
    pub binary_path: String,
    pub program: Option<ProgramInfo>,
    pub symbols: Vec<SymbolInfo>,
    pub functions: Vec<FunctionInfo>,
    pub strings: Vec<StringInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub arch: String,
    pub endian: String,
    pub entry_point: u64,
    pub base_address: u64,
    pub image_size: u64,
}

impl GhidraMcpServer {
    pub fn new() -> Self {
        Self {
            analysis_results: HashMap::new(),
        }
    }

    /// Handle MCP request and return response
    pub fn handle_request(&mut self, req: McpRequest) -> McpResponse {
        let id = req.id;
        
        match req.method.as_str() {
            "initialize" => self.initialize(id),
            "analyze" => self.analyze(req.params, id),
            "get_symbols" => self.get_symbols(req.params, id),
            "get_functions" => self.get_functions(req.params, id),
            "get_strings" => self.get_strings(req.params, id),
            "decompile" => self.decompile(req.params, id),
            "find_references" => self.find_references(req.params, id),
            "get_address_info" => self.get_address_info(req.params, id),
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", req.method),
                }),
            },
        }
    }

    fn success_response(&self, id: Option<u64>, result: McpResult) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error_response(&self, id: Option<u64>, code: i32, message: String) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError { code, message }),
        }
    }

    fn initialize(&self, id: Option<u64>) -> McpResponse {
        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "server_name": "Ghidra MCP Server",
                "version": "1.0.0",
                "capabilities": {
                    "analyze": true,
                    "symbols": true,
                    "functions": true,
                    "decompile": true,
                    "references": true,
                },
                "supported_archs": ["x86", "x86_64", "ppc", "ppc64", "arm", "arm64", "mips"],
            })),
            symbols: None,
            functions: None,
            strings: None,
        })
    }

    fn analyze(&mut self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let binary_path = match params.binary_path {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing binary_path".to_string()),
        };

        // Check if binary exists
        if !std::path::Path::new(&binary_path).exists() {
            return self.error_response(id, -32603, format!("File not found: {}", binary_path));
        }

        // Create analysis session (placeholder - real implementation would call Ghidra)
        let session = AnalysisSession {
            binary_path: binary_path.clone(),
            program: Some(ProgramInfo {
                arch: "x86".to_string(),
                endian: "little".to_string(),
                entry_point: 0x401000,
                base_address: 0x400000,
                image_size: 0x10000,
            }),
            symbols: Vec::new(),
            functions: Vec::new(),
            strings: Vec::new(),
        };

        self.analysis_results.insert(binary_path.clone(), session);

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "status": "analyzed",
                "binary_path": binary_path,
                "program": self.analysis_results.get(&binary_path).unwrap().program,
            })),
            symbols: None,
            functions: None,
            strings: None,
        })
    }

    fn get_symbols(&mut self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let binary_path = match params.binary_path {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing binary_path".to_string()),
        };

        // Return placeholder symbols (real implementation would query Ghidra)
        let symbols = vec![
            SymbolInfo {
                name: "main".to_string(),
                address: 0x401000,
                symbol_type: "function".to_string(),
                namespace: "global".to_string(),
            },
            SymbolInfo {
                name: "_start".to_string(),
                address: 0x401000,
                symbol_type: "function".to_string(),
                namespace: "global".to_string(),
            },
        ];

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "symbol_count": symbols.len(),
            })),
            symbols: Some(symbols),
            functions: None,
            strings: None,
        })
    }

    fn get_functions(&mut self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let _binary_path = match params.binary_path {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing binary_path".to_string()),
        };

        // Return placeholder functions
        let functions = vec![
            FunctionInfo {
                name: "main".to_string(),
                entry_point: 0x401000,
                func_type: "function".to_string(),
                signature: "int main(int argc, char** argv)".to_string(),
                calling_convention: "cdecl".to_string(),
                local_variables: vec![
                    LocalVariable {
                        name: "argc".to_string(),
                        var_type: "int".to_string(),
                        offset: 8,
                    },
                    LocalVariable {
                        name: "argv".to_string(),
                        var_type: "char**".to_string(),
                        offset: 12,
                    },
                ],
            },
        ];

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "function_count": functions.len(),
            })),
            symbols: None,
            functions: Some(functions),
            strings: None,
        })
    }

    fn get_strings(&mut self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let _binary_path = match params.binary_path {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing binary_path".to_string()),
        };

        // Return placeholder strings
        let strings = vec![
            StringInfo {
                value: "Hello, World!".to_string(),
                address: 0x404000,
                length: 13,
                encoding: "ASCII".to_string(),
            },
        ];

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "string_count": strings.len(),
            })),
            symbols: None,
            functions: None,
            strings: Some(strings),
        })
    }

    fn decompile(&self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let function_name = params.function_name.unwrap_or_else(|| "main".to_string());

        // Return placeholder decompiled code
        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "function": function_name,
                "decompiled_code": format!(
                    "int {}(int argc, char **argv) {{\n  printf(\"Hello, World!\\n\");\n  return 0;\n}}",
                    function_name
                ),
            })),
            symbols: None,
            functions: None,
            strings: None,
        })
    }

    fn find_references(&self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let address = params.address.unwrap_or_else(|| "0x401000".to_string());

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "address": address,
                "references": [
                    {"from": "0x401050", "type": "CALL"},
                    {"from": "0x401100", "type": "JUMP"},
                ],
            })),
            symbols: None,
            functions: None,
            strings: None,
        })
    }

    fn get_address_info(&self, params: Option<McpParams>, id: Option<u64>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => return self.error_response(id, -32602, "Missing params".to_string()),
        };

        let address = params.address.unwrap_or_else(|| "0x401000".to_string());

        self.success_response(id, McpResult {
            success: true,
            data: Some(serde_json::json!({
                "address": address,
                "label": "main",
                "type": "function",
                "namespace": "global",
                "bytes": [0x55, 0x48, 0x89, 0xe5],
            })),
            symbols: None,
            functions: None,
            strings: None,
        })
    }
}

impl Default for GhidraMcpServer {
    fn default() -> Self {
        Self::new()
    }
}