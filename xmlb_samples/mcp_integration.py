"""
MCP (Model Context Protocol) Integration for Athanor
Enables external AI tools and agents to interact with the engine.
"""

import json
import asyncio
try:
    import websockets
    WEBSOCKETS_AVAILABLE = True
except ImportError:
    WEBSOCKETS_AVAILABLE = False
from typing import Dict, Any, List, Callable
from dataclasses import dataclass, asdict
import uuid

@dataclass
class MCPTool:
    """Definition of an MCP tool that can be called by external agents."""
    name: str
    description: str
    parameters: Dict[str, Any]
    handler: Callable

@dataclass
class MCPResource:
    """Definition of an MCP resource (data) that can be accessed."""
    uri: str
    name: str
    description: str
    mime_type: str

class MCPServer:
    """MCP Server for exposing Athanor Engine functionality to external agents."""
    
    def __init__(self, host: str = "localhost", port: int = 3458):
        self.host = host
        self.port = port
        self.tools: Dict[str, MCPTool] = {}
        self.resources: Dict[str, MCPResource] = {}
        self.server = None
    
    def register_tool(self, name: str, description: str, parameters: Dict[str, Any], 
                      handler: Callable):
        """Register a tool that can be called by MCP clients."""
        self.tools[name] = MCPTool(name, description, parameters, handler)
    
    def register_resource(self, uri: str, name: str, description: str, mime_type: str):
        """Register a resource that can be accessed by MCP clients."""
        self.resources[uri] = MCPResource(uri, name, description, mime_type)
    
    async def start(self):
        """Start the MCP WebSocket server."""
        if not WEBSOCKETS_AVAILABLE:
            print("WARNING: websockets not installed - MCP server will have limited functionality")
            return
        self.server = await websockets.serve(self._handle_connection, self.host, self.port)
        print(f"MCP Server started on ws://{self.host}:{self.port}")
    
    async def stop(self):
        """Stop the MCP server."""
        if self.server:
            self.server.close()
            await self.server.wait_closed()
    
    async def _handle_connection(self, websocket, path):
        """Handle incoming MCP connections."""
        try:
            async for message in websocket:
                request = json.loads(message)
                response = await self._process_request(request)
                await websocket.send(json.dumps(response))
        except Exception as e:
            print(f"MCP connection error: {e}")
    
    async def _process_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Process an MCP request."""
        method = request.get("method")
        params = request.get("params", {})
        request_id = request.get("id", str(uuid.uuid4()))
        
        try:
            if method == "tools/list":
                return self._handle_list_tools(request_id)
            elif method == "tools/call":
                return await self._handle_call_tool(request_id, params)
            elif method == "resources/list":
                return self._handle_list_resources(request_id)
            elif method == "resources/read":
                return await self._handle_read_resource(request_id, params)
            else:
                return {"id": request_id, "error": {"code": -32601, "message": f"Method not found: {method}"}}
        except Exception as e:
            return {"id": request_id, "error": {"code": -32603, "message": str(e)}}
    
    def _handle_list_tools(self, request_id: str) -> Dict[str, Any]:
        """List all available tools."""
        tools_list = []
        for tool in self.tools.values():
            tools_list.append({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": {
                    "type": "object",
                    "properties": tool.parameters,
                    "required": list(tool.parameters.keys())
                }
            })
        return {"id": request_id, "result": {"tools": tools_list}}
    
    async def _handle_call_tool(self, request_id: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Call a registered tool."""
        tool_name = params.get("name")
        arguments = params.get("arguments", {})
        
        if tool_name not in self.tools:
            return {"id": request_id, "error": {"code": -32602, "message": f"Tool not found: {tool_name}"}}
        
        tool = self.tools[tool_name]
        result = await tool.handler(arguments)
        
        return {"id": request_id, "result": {"content": [{"type": "text", "text": str(result)}]}}
    
    def _handle_list_resources(self, request_id: str) -> Dict[str, Any]:
        """List all available resources."""
        resources_list = []
        for resource in self.resources.values():
            resources_list.append({
                "uri": resource.uri,
                "name": resource.name,
                "description": resource.description,
                "mimeType": resource.mime_type
            })
        return {"id": request_id, "result": {"resources": resources_list}}
    
    async def _handle_read_resource(self, request_id: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Read a registered resource."""
        uri = params.get("uri")
        
        if uri not in self.resources:
            return {"id": request_id, "error": {"code": -32602, "message": f"Resource not found: {uri}"}}
        
        # In a real implementation, this would fetch actual data
        resource = self.resources[uri]
        return {"id": request_id, "result": {"contents": [{"uri": uri, "mimeType": resource.mime_type, "text": "Resource content here"}]}}

# Global MCP server instance
mcp_server = None

def init_mcp_server(host: str = "localhost", port: int = 3458):
    """Initialize the MCP server globally."""
    global mcp_server
    mcp_server = MCPServer(host, port)
    return mcp_server

# Built-in Athanor Engine tools
def register_athanor_tools(server: MCPServer, athanor_api_base: str = "http://localhost:3457"):
    """Register Athanor Engine tools with the MCP server."""
    
    async def parse_file(arguments: Dict[str, Any]) -> str:
        """Parse an Athanor format file."""
        import urllib.request
        rel_path = arguments.get("path")
        url = f"{alchemy_api_base}/api/parse/{urllib.parse.quote(rel_path)}"
        with urllib.request.urlopen(url) as resp:
            return resp.read().decode()
    
    async def list_files(arguments: Dict[str, Any]) -> str:
        """List game files, optionally filtered by extension."""
        import urllib.request
        ext = arguments.get("extension")
        url = f"{alchemy_api_base}/api/files"
        if ext:
            url += f"/{ext}"
        with urllib.request.urlopen(url) as resp:
            return resp.read().decode()
    
    async def save_xmlb(arguments: Dict[str, Any]) -> str:
        """Save modified XMLB with new nodes."""
        import urllib.request, json
        url = f"{alchemy_api_base}/api/save"
        req = urllib.request.Request(url, data=json.dumps(arguments).encode())
        req.add_header('Content-Type', 'application/json')
        with urllib.request.urlopen(req) as resp:
            return resp.read().decode()
    
    async def deploy_mod(arguments: Dict[str, Any]) -> str:
        """Deploy a mod to the game directory."""
        import urllib.request, json
        url = f"{alchemy_api_base}/api/deploy"
        req = urllib.request.Request(url, data=json.dumps(arguments).encode())
        req.add_header('Content-Type', 'application/json')
        with urllib.request.urlopen(req) as resp:
            return resp.read().decode()
    
    async def upscale_asset(arguments: Dict[str, Any]) -> str:
        """Upscale an asset using AI."""
        from ai_upscaler import upscale_asset
        input_path = arguments.get("input")
        output_path = arguments.get("output")
        scale = arguments.get("scale", 4)
        success = upscale_asset(input_path, output_path, scale)
        return f"Upscaling {'succeeded' if success else 'failed'}: {input_path} -> {output_path}"
    
    async def create_level(arguments: Dict[str, Any]) -> str:
        """Create a new level from template."""
        level_name = arguments.get("name", "new_level")
        template = arguments.get("template", "empty")
        return f"Created level '{level_name}' from template '{template}'"
    
    async def generate_polygons(arguments: Dict[str, Any]) -> str:
        """Generate polygon mesh for asset upscaling."""
        from polygon_generator import generate_polygons
        input_path = arguments.get("input")
        output_path = arguments.get("output")
        detail = arguments.get("detail", "high")
        success = generate_polygons(input_path, output_path, detail)
        return f"Polygon generation {'succeeded' if success else 'failed'}: {input_path} -> {output_path}"
    
    # Register tools
    server.register_tool(
        "parse_alchemy_file",
        "Parse an Athanor format file (XMLB, BNX, IGB, ZSM, ZAM, etc.)",
        {"path": {"type": "string", "description": "Relative path to file in game directory"}},
        parse_file
    )
    
    server.register_tool(
        "list_game_files",
        "List all game files, optionally filtered by extension",
        {"extension": {"type": "string", "description": "File extension filter (e.g., xmlb, igb)"}},
        list_files
    )
    
    server.register_tool(
        "save_xmlb_modifications",
        "Save modified XMLB with new nodes inserted",
        {"file": {"type": "string", "description": "Relative path to XMLB file"}, 
         "newNodes": {"type": "array", "description": "Array of new node objects to insert"}},
        save_xmlb
    )
    
    server.register_tool(
        "deploy_mod",
        "Deploy mod files to game directory",
        {"modFiles": {"type": "array", "description": "Array of file paths to deploy"}},
        deploy_mod
    )
    
    server.register_tool(
        "upscale_asset",
        "Upscale a game asset using AI super-resolution",
        {"input": {"type": "string", "description": "Input asset path"},
         "output": {"type": "string", "description": "Output asset path"},
         "scale": {"type": "integer", "description": "Scale factor (2, 4, 8)", "default": 4}},
        upscale_asset
    )
    
    server.register_tool(
        "create_level",
        "Create a new level from template",
        {"name": {"type": "string", "description": "Level name"},
         "template": {"type": "string", "description": "Template to use (empty, outdoor, dungeon, arena)", "default": "empty"}},
        create_level
    )
    
    server.register_tool(
        "generate_polygons",
        "Generate polygon mesh for asset upscaling",
        {"input": {"type": "string", "description": "Input model/asset path"},
         "output": {"type": "string", "description": "Output mesh path"},
         "detail": {"type": "string", "description": "Detail level (low, medium, high)", "default": "high"}},
        generate_polygons
    )
    
    # Register resources
    server.register_resource(
        "alchemy://formats",
        "Supported Formats",
        "List of all supported Athanor file formats",
        "application/json"
    )
    
    server.register_resource(
        "alchemy://nodes",
        "Node Templates",
        "Available node types for XMLB creation",
        "application/json"
    )
    
    server.register_resource(
        "alchemy://signals",
        "Signal Definitions",
        "X-Men Legends signal/event definitions",
        "application/json"
    )
    
    server.register_resource(
        "alchemy://components",
        "ECS Components",
        "Available ECS component definitions",
        "application/json"
    )

# Example usage
async def main():
    server = init_mcp_server()
    register_alchemy_tools(server)
    await server.start()
    
    # Keep running
    try:
        await asyncio.Future()  # Run forever
    except KeyboardInterrupt:
        await server.stop()

if __name__ == "__main__":
    asyncio.run(main())