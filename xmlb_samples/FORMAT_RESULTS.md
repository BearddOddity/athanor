# X-Men Legends II - XMLB Format Analysis Results

## File Structure (options.XMLB)
- File size: 7,856 bytes
- Magic: 0x000011B1
- Version: 1
- String table at: 0x1410
- String count: 112
- Node data starts at: 0x70 (after 20-byte header)

## Node Structure (hypothesis)
Each node appears to be 28 bytes + properties:
- 4 bytes: name offset (into string table)
- 4 bytes: x position
- 4 bytes: y position
- 4 bytes: width
- 4 bytes: height
- 4 bytes: type/flags
- 4 bytes: property count
- N * 8 bytes: property pairs (key_offset, value)

## String Table (112 strings)
Key strings found:
- 'MENU' (root type)
- 'animonclose', 'animonopen', 'fadein', 'fadeout' (animation props)
- 'desctext1' = '$MENU_BACK Back'
- 'desctext2' = '$MENU_ACCEPT Change'
- 'fullscreen' = 'false'
- 'igb' = 'x2m_options' (texture reference)
- 'type' = 'OPTIONS_MENU'
- 'updownonly' = 'true'
- 'animtext' = 'name' (child node type)
- 'mark' (node type)
- 'alpha' = '0', 'time' = '0.429', '1' (animation params)
- 'help' = '0.857' (animation)
- 'anim' = 'end', 'start', 'open', 'close' (animation states)
- 'item' (node type)
- 'model' = 'ui/models/m_track' (3D model reference)
- 'track01' = 'ui/models/m_team_helpbar'
- 'helpbar' = 'MENU_ITEM_MODEL'
- 'animtext_scene' (composite animation)
- 'style' = 'STYLE_TITLE_MED'
- 'title_bracket_top'
- 'title' (node type)
- 'animate' (boolean)
- 'label_options' = 'Options'
- 'label_controls' = 'Controls'
- 'text' = 'Options' (display text)
- 'textalignx' = 'TEXT_ALIGN_LEFT'
- 'leftcmd' = 'setdecrement sfxvolume'
- 'rightcmd' = 'setincrement sfxvolume'
- 'startactive' (boolean)
- 'STYLE_MENU_WHITE_MED'
- 'Effects Volume'
- 'up', 'down' (navigation)
- 'onfocus' (event handler)
- 'focus' (state)

## Widget Types Identified
1. **MENU** - Root container
2. **animtext** - Text with animation
3. **mark** - Marker/separator
4. **item** - Menu item
5. **title** - Title text
6. **animtext_scene** - Animated text scene

## Node Array (0x70 to 0x1410)
~300 nodes, each with:
- Position (x, y as 4-byte values)
- Size (w, h as 4-byte values)
- Type/flags (1-11 based on node type)
- Property count (2-5 typically)
- Properties: key-value pairs referencing string table

## Key Insight
The XMLB is NOT a binary tree. It's a **flat array of widget nodes** with properties.
Each node has:
- A name (e.g., "animtext", "mark", "item")
- Position and size
- Type/flags
- Properties that reference the string table

The "parent/child" relationship is likely maintained through the properties (e.g., "name" property points to parent).

## Next Steps
1. Write a proper parser that reads the node array
2. Identify which nodes are the Resolution/FSAA widgets
3. Modify coordinates to scale to 16:9
4. Add new widget nodes for additional settings
5. Test in-game
