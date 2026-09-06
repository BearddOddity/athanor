import urllib.request, json

def add_graphics_settings_to_options():
    # Parse current options.XMLB
    resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
    data = json.loads(resp.read().decode())
    
    print('Original: %d nodes' % len(data['nodes']))
    print('First node: %s / %s' % (data['nodes'][0]['type'], data['nodes'][0]['name']))
    
    # Find a good insertion point - after the Controls section label (index 59)
    # The Controls section starts around item 2072-2360
    
    # Graphics settings to add - these will appear after current settings
    # Using 16:9 scaled coordinates (multiplied by 0.9375 from 4:3)
    
    # Find the last numeric item value
    max_item = 0
    for node in data['nodes']:
        for prop in node['props']:
            if prop['key'] == 'item':
                try:
                    v = int(prop['value'])
                    if v > max_item:
                        max_item = v
                except:
                    pass
    
    print('Max item position: %d' % max_item)
    
    # New graphics settings nodes - insert before node 103 (nofocus)
    # Item positions: 3660, 3700, 3740, 3780, 3820 (40px spacing)
    new_nodes = [
        {
            "index": len(data['nodes']),
            "type": "OPTIONS_MENU",
            "name": "graphics_settings_label",
            "props": [
                {"key": "mark", "value": 3600, "isRaw": True},
                {"key": "item", "value": 3600, "isRaw": True},
                {"key": "style", "value": "STYLE_TITLE_MED", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 1,
            "type": "OPTIONS_MENU",
            "name": "graphics_res_label",
            "props": [
                {"key": "mark", "value": 3640, "isRaw": True},
                {"key": "item", "value": 3640, "isRaw": True},
                {"key": "text", "value": "Resolution", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 2,
            "type": "OPTIONS_MENU",
            "name": "graphics_res",
            "props": [
                {"key": "mark", "value": 3680, "isRaw": True},
                {"key": "item", "value": 3680, "isRaw": True},
                {"key": "usecmd", "value": "setgraphics res", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 3,
            "type": "OPTIONS_MENU",
            "name": "graphics_fsaa_label",
            "props": [
                {"key": "mark", "value": 3720, "isRaw": True},
                {"key": "item", "value": 3720, "isRaw": True},
                {"key": "text", "value": "FSAA", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 4,
            "type": "OPTIONS_MENU",
            "name": "graphics_fsaa",
            "props": [
                {"key": "mark", "value": 3760, "isRaw": True},
                {"key": "item", "value": 3760, "isRaw": True},
                {"key": "usecmd", "value": "setgraphics fsaa", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 5,
            "type": "OPTIONS_MENU",
            "name": "graphics_shadow_label",
            "props": [
                {"key": "mark", "value": 3800, "isRaw": True},
                {"key": "item", "value": 3800, "isRaw": True},
                {"key": "text", "value": "Shadow Quality", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 6,
            "type": "OPTIONS_MENU",
            "name": "graphics_shadow",
            "props": [
                {"key": "mark", "value": 3840, "isRaw": True},
                {"key": "item", "value": 3840, "isRaw": True},
                {"key": "usecmd", "value": "setgraphics shadow", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 7,
            "type": "OPTIONS_MENU",
            "name": "graphics_texture_label",
            "props": [
                {"key": "mark", "value": 3880, "isRaw": True},
                {"key": "item", "value": 3880, "isRaw": True},
                {"key": "text", "value": "Texture Quality", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 8,
            "type": "OPTIONS_MENU",
            "name": "graphics_texture",
            "props": [
                {"key": "mark", "value": 3920, "isRaw": True},
                {"key": "item", "value": 3920, "isRaw": True},
                {"key": "usecmd", "value": "setgraphics texture", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 9,
            "type": "OPTIONS_MENU",
            "name": "graphics_viewdist_label",
            "props": [
                {"key": "mark", "value": 3960, "isRaw": True},
                {"key": "item", "value": 3960, "isRaw": True},
                {"key": "text", "value": "View Distance", "isRaw": False}
            ]
        },
        {
            "index": len(data['nodes']) + 10,
            "type": "OPTIONS_MENU",
            "name": "graphics_viewdist",
            "props": [
                {"key": "mark", "value": 4000, "isRaw": True},
                {"key": "item", "value": 4000, "isRaw": True},
                {"key": "usecmd", "value": "setgraphics viewdist", "isRaw": False}
            ]
        }
    ]
    
    # Combine nodes
    combined = data['nodes'] + new_nodes
    print('Combined: %d nodes' % len(combined))
    
    # Save
    body = json.dumps({'relPath': 'UI/menus/options.XMLB', 'nodes': combined}).encode()
    req = urllib.request.Request(
        'http://localhost:3457/api/save',
        data=body,
        headers={'Content-Type': 'application/json'},
        method='POST'
    )
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read().decode())
    print('Saved: nodes=%d, size=%d' % (result['nodes'], result['size']))
    
    # Verify
    resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
    verify = json.loads(resp.read().decode())
    print('Verify: nodes=%d, first type=%s' % (len(verify['nodes']), verify['nodes'][0]['type']))
    
    # Show new nodes
    print()
    print('New graphics settings nodes:')
    for node in verify['nodes'][-11:]:
        print('  [%2d] %s / %s' % (node['index'], node['type'], node['name']))

if __name__ == '__main__':
    add_graphics_settings_to_options()