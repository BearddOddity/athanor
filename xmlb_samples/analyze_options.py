import urllib.request, json

resp = urllib.request.urlopen('http://localhost:3457/api/parse/UI/menus/options.XMLB')
data = json.loads(resp.read().decode())
print('Options.XMLB:')
print('  Size:', data['size'])
print('  Nodes:', len(data['nodes']))
print('  String count:', data['strCount'])
print()
print('First 30 nodes:')
for n in data['nodes'][:30]:
    print('  [%2d] %s / %s' % (n['index'], n['type'], n['name']))
    for p in n['props'][:2]:
        print('        %s = %s' % (p['key'], p['value']))