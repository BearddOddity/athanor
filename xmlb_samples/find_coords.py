import urllib.request, json

# Parse original
resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
orig = json.loads(resp.read().decode())

print('Finding coordinate-like properties...')
print('Nodes with mark/item/time/alpha properties:')

coords_found = []
for i, node in enumerate(orig['nodes']):
    mark_val = None
    item_val = None
    time_val = None
    alpha_val = None
    
    for prop in node['props']:
        if prop['key'] == 'mark':
            mark_val = prop['value']
        elif prop['key'] == 'item':
            item_val = prop['value']
        elif prop['key'] == 'time':
            time_val = prop['value']
        elif prop['key'] == 'alpha':
            alpha_val = prop['value']
    
    if mark_val is not None or item_val is not None:
        coords_found.append({
            'index': i,
            'type': node['type'],
            'name': node['name'],
            'mark': mark_val,
            'item': item_val,
            'time': time_val,
            'alpha': alpha_val
        })
        
print('Total nodes with mark/item:', len(coords_found))

# Look for sequential items that might be menu entries
print()
print('Sequential item values (potential Y coordinates):')
prev_item = None
sequential_count = 0
for c in coords_found:
    if c['item'] is not None:
        if prev_item is not None and c['item'] == prev_item + 40:
            sequential_count += 1
            if sequential_count <= 5:  # Show first 5
                print('  [%2d] %s: item=%d (prev+40)' % (c['index'], c['type'] or '???', c['item']))
        elif prev_item is not None:
            if sequential_count > 0:
                print('  ... (%d sequential items found)' % sequential_count)
                sequential_count = 0
        prev_item = c['item']

print()
print('Sample mark values:')
for c in coords_found[:10]:
    if c['mark'] is not None and c['mark'] >= 100:  # Likely Y coordinate
        print('  [%2d] %s: mark=%d' % (c['index'], c['type'] or '???', c['mark']))