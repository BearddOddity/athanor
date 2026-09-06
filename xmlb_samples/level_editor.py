"""
Level Editor for Athanor
Provides a framework for creating and managing game levels.
"""

import json
import asyncio
from typing import Dict, List, Optional, Callable
from dataclasses import dataclass, asdict
from abc import ABC, abstractmethod
import tempfile
import os

@dataclass
class LevelTemplate:
    """Definition of a level template."""
    name: str
    width: int
    height: int
    tileset: str
    difficulty: str = "normal"
    
    @property
    def tile_grid(self) -> List[List[str]]:
        """Generate a tile grid for the level."""
        return [[" " for _ in range(self.width)] for _ in range(self.height)]

class LevelEditor:
    """Main level editor class."""
    
    def __init__(self, game_dir: str = "D:/re-lab-share/xmen2_mod/xmlb_samples"):
        self.game_dir = game_dir
        self.levels: Dict[str, Dict] = {}
        self.current_level: Optional[Dict] = None
    
    def create_level(self, name: str, width: int, height: int, tileset: str) -> str:
        """Create a new level from a template."""
        template = LevelTemplate(name=name, width=width, height=height, tileset=tileset)
        
        # Create level file
        level_path = os.path.join(self.game_dir, f"level_{name}.xmlb")
        
        # Generate initial level content
        content = {
            "name": name,
            "dimensions": {"width": width, "height": height},
            "tileset": tileset,
            "difficulty": "normal",
            "tiles": template.tile_grid,
            "objects": []
        }
        
        with open(level_path, 'w') as f:
            json.dump(content, f, indent=2)
        
        self.levels[name] = content
        self.current_level = content
        
        print(f"Created level: {name} ({width}x{height})")
        return level_path
    
    def load_level(self, name: str) -> Optional[Dict]:
        """Load a saved level."""
        if name in self.levels:
            return self.levels[name]
        return None
    
    def save_level(self, name: str, path: str) -> bool:
        """Save a level to disk."""
        if name not in self.levels:
            print(f"Level {name} not found")
            return False
        
        with open(path, 'w') as f:
            json.dump(self.levels[name], f, indent=2)
        print(f"Saved level: {name} to {path}")
        return True
    
    def delete_level(self, name: str) -> bool:
        """Delete a level."""
        if name in self.levels:
            del self.levels[name]
            print(f"Deleted level: {name}")
            return True
        return False
    
    def list_levels(self) -> List[str]:
        """List all saved levels."""
        return list(self.levels.keys())
    
    async def add_object(self, name: str, x: int, y: int, type_str: str, 
                        properties: Optional[Dict] = None) -> bool:
        """Add an object to the current level."""
        if not self.current_level:
            print("No active level. Create one first.")
            return False
        
        obj = {
            "name": name,
            "x": x,
            "y": y,
            "type": type_str,
            "properties": properties or {}
        }
        
        self.current_level.setdefault("objects", []).append(obj)
        print(f"Added object: {name} at ({x}, {y})")
        return True
    
    async def remove_object(self, name: str) -> bool:
        """Remove an object from the current level."""
        if not self.current_level:
            print("No active level")
            return False
        
        self.current_level["objects"].remove(next((o for o in self.current_level["objects"] if o["name"] == name), None))
        print(f"Removed object: {name}")
        return True
    
    async def get_objects(self) -> List[Dict]:
        """Get all objects in the current level."""
        return self.current_level.get("objects", [])
    
    def export_level(self, name: str, format: str = "xmlb") -> bool:
        """Export a level to a file."""
        if not self.current_level:
            print("No active level")
            return False
        
        if format == "xmlb":
            path = os.path.join(self.game_dir, f"exported/{name}.xmlb")
            with open(path, 'w') as f:
                json.dump(self.current_level, f, indent=2)
            print(f"Exported level to {path}")
            return True
        else:
            print(f"Unsupported format: {format}")
            return False

# Global level editor instance
level_editor = None

def init_level_editor(game_dir: str = "D:/re-lab-share/xmen2_mod/xmlb_samples"):
    """Initialize the level editor globally."""
    global level_editor
    level_editor = LevelEditor(game_dir)
    return level_editor
    return level_editor

def create_level(name: str, width: int, height: int, tileset: str) -> str:
    """Create a new level from a template."""
    level_editor = init_level_editor()
    return level_editor.create_level(name, width, height, tileset)

def load_level(name: str) -> Optional[Dict]:
    """Load a saved level."""
    level_editor = init_level_editor()
    return level_editor.load_level(name)

def save_level(name: str, path: str) -> bool:
    """Save a level to disk."""
    level_editor = init_level_editor()
    return level_editor.save_level(name, path)

def delete_level(name: str) -> bool:
    """Delete a level."""
    level_editor = init_level_editor()
    return level_editor.delete_level(name)

def list_levels() -> List[str]:
    """List all saved levels."""
    level_editor = init_level_editor()
    return level_editor.list_levels()

# Example usage
if __name__ == "__main__":
    # Initialize
    init_level_editor()
    
    # Create a new level
    level_path = create_level("forest_01", 800, 600, "forest")
    print(f"Created level: {level_path}")
    
    # Add some objects
    level_editor.add_object("tree_1", 100, 200, "tree", {"height": 10, "color": "green"})
    level_editor.add_object("rock_1", 400, 300, "rock", {"size": "large"})
    
    # List objects
    print("Objects in forest_01:", level_editor.get_objects())
    
    # Export
    level_editor.export_level("forest_01", "exports/forest_01.xmlb")
    
    # Delete
    # delete_level("forest_01")
