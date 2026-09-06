import urllib.request, json

# Parse original
resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
orig = json.loads(resp.read().decode())
print('Original: nodes=%d, first type=%s' % (len(orig['nodes']), orig['nodes'][0]['type']))

# Save unchanged
body = json.dumps({'relPath': 'UI/menus/options.XMLB', 'nodes': orig['nodes']}).encode()
req = urllib.request.Request('http://localhost:3457/api/save', data=body, headers={'Content-Type': 'application/json'}, method='POST')
resp = urllib.request.urlopen(req)
result = json.loads(resp.read().decode())
print('Save: OK, nodes=%d, size=%d' % (result['nodes'], result['size']))

# Verify parse
resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
verify = json.loads(resp.read().decode())
print('Verify: nodes=%d, first type=%s' % (len(verify['nodes']), verify['nodes'][0]['type']))

# Compare first 5 nodes
allMatch = True
for i in range(min(5, len(verify['nodes']), len(orig['nodes']))):
    v = verify['nodes'][i]
    o = orig['nodes'][i]
    if v['type'] != o['type'] or v['name'] != o['name']:
        print('  Node %d MISMATCH: %s/%s vs %s/%s' % (i, v['type'], v['name'], o['type'], o['name']))
        allMatch = False

if allMatch:
    print('First 5 nodes match!')
else:
    print('MISMATCHES FOUND!')