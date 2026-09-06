with open('D:/re-lab-share/xmen2_mod/xmlb_samples/alchemy_server.ts') as f:
    content = f.read()

# Find save endpoint
idx = content.find('"/api/save"')
print(content[idx:idx+1500])