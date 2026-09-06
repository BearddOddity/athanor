import urllib.request, json

# Parse original
resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
orig = json.loads(resp.read().decode())
print('Original: nodes=%d' % len(orig['nodes']))
print('First node type:', orig['nodes'][0]['type'] if orig['nodes'] else 'None')
print()

# Add graphics settings at the end
# We'll insert new nodes for graphics options
# Looking at the format: each node has index, type, name, props
# Graphics settings typically have: type=OPTIONS_MENU, with mark/item coordinates

# New graphics settings nodes to add
new_graphics_nodes = [
    {
        "index": 112,  # after the original 112 nodes
        "type": "OPTIONS_MENU",
        "name": "graphics_settings",
        "props": [
            {"key": "mark", "value": 100, "isRaw": True},
            {"key": "item", "value": 500, "isRaw": True},
            {"key": "resolution", "value": "16:9", "isRaw": False}
        ]
    },
    {
        "index": 113,
        "type": "OPTIONS_MENU",
        "name": "resolution",
        "props": [
            {"key": "mark", "value": 140, "isRaw": True},
            {"key": "item", "value": 540, "isRaw": True},
            {"key": "text", "value": "Resolution", "isRaw": False}
        ]
    },
    {
        "index": 114,
        "type": "OPTIONS_MENU",
        "name": "fsaa",
        "props": [
            {"key": "mark", "value": 180, "isRaw": True},
            {"key": "item", "value": 580, "isRaw": True},
            {"key": "text", "value": "FSAA", "isRaw": False}
        ]
    },
    {
        "index": 115,
        "type": "OPTIONS_MENU",
        "name": "shadow_quality",
        "props": [
            {"key": "mark", "value": 220, "isRaw": True},
            {"key": "item", "value": 620, "isRaw": True},
            {"key": "text", "value": "Shadow Quality", "isRaw": False}
        ]
    },
    {
        "index": 116,
        "type": "OPTIONS_MENU",
        "name": "texture_quality",
        "props": [
            {"key": "mark", "value": 260, "isRaw": True},
            {"key": "item", "value": 660, "isRaw": True},
            {"key": "text", "value": "Texture Quality", "isRaw": False}
        ]
    },
    {
        "index": 117,
        "type": "OPTIONS_MENU",
        "name": "view_distance",
        "props": [
            {"key": "mark", "value": 300, "isRaw": True},
            {"key": "item", "value": 700, "isRaw": True},
            {"key": "text", "value": "View Distance", "isRaw": False}
        ]
    }
]

# Combine original + new nodes
combined_nodes = orig['nodes'] + new_graphics_nodes
print('Combined: nodes=%d' % len(combined_nodes))

# Save
body = json.dumps({'relPath': 'UI/menus/options.XMLB', 'nodes': combined_nodes}).encode()
req = urllib.request.Request('http://localhost:3457/api/save', data=body, headers={'Content-Type': 'application/json'}, method='POST')
resp = urllib.request.urlopen(req)
result = json.loads(resp.read().decode())
print('Save: OK, nodes=%d, size=%d' % (result['nodes'], result['size']))

# Verify parse
resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
verify = json.loads(resp.read().decode())
print('Verify: nodes=%d' % len(verify['nodes']))
if verify['nodes']:
    print('Last node:', verify['nodes'][-1])
# Check some original nodes still work
print('First node type:', verify['nodes'][0]['type'] if verify['nodes'] else 'None')