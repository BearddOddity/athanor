"""
Polygon Generator for Athanor
Procedurally generates polygon meshes for upscaling old assets
while preserving the original art style.
"""

import json
import math
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass

@dataclass
class PolygonVertex:
    """A vertex in the polygon mesh."""
    x: float
    y: float
    z: float = 0.0
    u: float = 0.0
    v: float = 0.0
    normal_x: float = 0.0
    normal_y: float = 0.0
    normal_z: float = 1.0

@dataclass
class PolygonFace:
    """A face/triangle in the polygon mesh."""
    vertex_indices: Tuple[int, int, int]
    material_id: int = 0

@dataclass
class PolygonMesh:
    """A complete polygon mesh."""
    vertices: List[PolygonVertex]
    faces: List[PolygonFace]
    name: str = "mesh"

class PolygonGenerator:
    """Generate polygon meshes for asset upscaling."""
    
    def __init__(self, detail_level: str = "high"):
        """
        Initialize the polygon generator.
        
        Args:
            detail_level: Level of detail (low, medium, high, ultra)
        """
        self.detail_level = detail_level
        self.detail_settings = {
            "low": {"subdivisions": 1, "smoothing": 0.0, "vertices_per_face": 3},
            "medium": {"subdivisions": 2, "smoothing": 0.5, "vertices_per_face": 4},
            "high": {"subdivisions": 4, "smoothing": 0.8, "vertices_per_face": 6},
            "ultra": {"subdivisions": 8, "smoothing": 1.0, "vertices_per_face": 8}
        }
    
    def generate_from_image(self, image_path: str) -> PolygonMesh:
        """
        Generate a polygon mesh from an image.
        
        Args:
            image_path: Path to input image
            
        Returns:
            Generated polygon mesh
        """
        try:
            from PIL import Image
            img = Image.open(image_path)
            width, height = img.size
            
            # Generate a simple grid mesh
            return self._generate_grid_mesh(width, height, 32)
        except Exception as e:
            print(f"  Error loading image: {e}")
            return self._generate_default_mesh()
    
    def generate_from_primitive(self, primitive_type: str, size: Tuple[float, float, float]) -> PolygonMesh:
        """
        Generate a mesh from a primitive shape.
        
        Args:
            primitive_type: Type of primitive (box, sphere, cylinder, cone)
            size: Size of the primitive (width, height, depth)
            
        Returns:
            Generated polygon mesh
        """
        if primitive_type == "box":
            return self._generate_box(size)
        elif primitive_type == "sphere":
            return self._generate_sphere(size[0], size[1])
        elif primitive_type == "cylinder":
            return self._generate_cylinder(size[0], size[1])
        elif primitive_type == "cone":
            return self._generate_cone(size[0], size[1])
        else:
            raise ValueError(f"Unknown primitive type: {primitive_type}")
    
    def _generate_grid_mesh(self, width: int, height: int, grid_size: int) -> PolygonMesh:
        """Generate a simple grid mesh."""
        vertices = []
        faces = []
        
        x_step = width / grid_size
        y_step = height / grid_size
        
        for y in range(grid_size + 1):
            for x in range(grid_size + 1):
                px = x * x_step
                py = y * y_step
                u = x / grid_size
                v = y / grid_size
                vertices.append(PolygonVertex(px, py, 0.0, u, v))
        
        for y in range(grid_size):
            for x in range(grid_size):
                i = y * (grid_size + 1) + x
                faces.append(PolygonFace((i, i + 1, i + grid_size + 1)))
                faces.append(PolygonFace((i + 1, i + grid_size + 2, i + grid_size + 1)))
        
        return PolygonMesh(vertices, faces, "grid_mesh")
    
    def _generate_default_mesh(self) -> PolygonMesh:
        """Generate a default mesh."""
        vertices = [
            PolygonVertex(0, 0, 0, 0.0, 0.0),
            PolygonVertex(100, 0, 0, 1.0, 0.0),
            PolygonVertex(100, 100, 0, 1.0, 1.0),
            PolygonVertex(0, 100, 0, 0.0, 1.0),
        ]
        faces = [
            PolygonFace((0, 1, 2)),
            PolygonFace((0, 2, 3)),
        ]
        return PolygonMesh(vertices, faces, "default_mesh")
    
    def _generate_box(self, size: Tuple[float, float, float]) -> PolygonMesh:
        """Generate a box mesh."""
        w, h, d = size
        vertices = [
            PolygonVertex(-w/2, -h/2, -d/2, 0.0, 0.0),
            PolygonVertex(w/2, -h/2, -d/2, 1.0, 0.0),
            PolygonVertex(w/2, h/2, -d/2, 1.0, 1.0),
            PolygonVertex(-w/2, h/2, -d/2, 0.0, 1.0),
            PolygonVertex(-w/2, -h/2, d/2, 0.0, 0.0),
            PolygonVertex(w/2, -h/2, d/2, 1.0, 0.0),
            PolygonVertex(w/2, h/2, d/2, 1.0, 1.0),
            PolygonVertex(-w/2, h/2, d/2, 0.0, 1.0),
        ]
        
        faces = [
            PolygonFace((0, 1, 2)), PolygonFace((0, 2, 3)),
            PolygonFace((4, 5, 6)), PolygonFace((4, 6, 7)),
            PolygonFace((0, 4, 7)), PolygonFace((0, 7, 3)),
            PolygonFace((1, 5, 6)), PolygonFace((1, 6, 2)),
            PolygonFace((3, 2, 6)), PolygonFace((3, 6, 7)),
            PolygonFace((0, 1, 5)), PolygonFace((0, 5, 4)),
        ]
        
        return PolygonMesh(vertices, faces, "box")
    
    def _generate_sphere(self, radius: float, segments: float) -> PolygonMesh:
        """Generate a sphere mesh."""
        vertices = []
        faces = []
        
        stack_count = int(segments)
        slice_count = int(segments * 2)
        
        for i in range(stack_count + 1):
            phi = math.pi * i / stack_count
            for j in range(slice_count + 1):
                theta = 2 * math.pi * j / slice_count
                
                x = radius * math.sin(phi) * math.cos(theta)
                y = radius * math.cos(phi)
                z = radius * math.sin(phi) * math.sin(theta)
                
                u = j / slice_count
                v = i / stack_count
                
                vertices.append(PolygonVertex(x, y, z, u, v, x/radius, y/radius, z/radius))
        
        for i in range(stack_count):
            for j in range(slice_count):
                first = i * (slice_count + 1) + j
                second = first + slice_count + 1
                
                if i == 0:
                    faces.append(PolygonFace((first, second, second + 1)))
                elif i == stack_count - 1:
                    faces.append(PolygonFace((first, first + 1, second)))
                else:
                    faces.append(PolygonFace((first, first + 1, second + 1)))
                    faces.append(PolygonFace((first, second + 1, second)))
        
        return PolygonMesh(vertices, faces, "sphere")
    
    def _generate_cylinder(self, radius: float, height: float) -> PolygonMesh:
        """Generate a cylinder mesh."""
        vertices = []
        faces = []
        
        segments = 32
        
        for i in range(segments + 1):
            theta = 2 * math.pi * i / segments
            
            x = radius * math.cos(theta)
            z = radius * math.sin(theta)
            vertices.append(PolygonVertex(x, -height/2, z, i/segments, 0.0))
            vertices.append(PolygonVertex(x, height/2, z, i/segments, 1.0))
        
        for i in range(segments):
            bottom1 = i * 2
            bottom2 = (i + 1) * 2
            top1 = bottom1 + 1
            top2 = bottom2 + 1
            
            faces.append(PolygonFace((bottom1, bottom2, top2)))
            faces.append(PolygonFace((bottom1, top2, top1)))
        
        return PolygonMesh(vertices, faces, "cylinder")
    
    def _generate_cone(self, radius: float, height: float) -> PolygonMesh:
        """Generate a cone mesh."""
        vertices = []
        faces = []
        
        segments = 32
        
        vertices.append(PolygonVertex(0, height/2, 0, 0.5, 0.0))
        
        for i in range(segments + 1):
            theta = 2 * math.pi * i / segments
            x = radius * math.cos(theta)
            z = radius * math.sin(theta)
            vertices.append(PolygonVertex(x, -height/2, z, i/segments, 1.0))
        
        for i in range(segments):
            apex = 0
            bottom1 = i + 1
            bottom2 = (i + 1) % segments + 1
            
            faces.append(PolygonFace((apex, bottom2, bottom1)))
        
        return PolygonMesh(vertices, faces, "cone")
    
    def export_mesh(self, mesh: PolygonMesh, output_path: str) -> bool:
        """Export mesh to JSON format."""
        try:
            mesh_data = {
                "name": mesh.name,
                "vertices": [
                    {"x": v.x, "y": v.y, "z": v.z, "u": v.u, "v": v.v,
                     "nx": v.normal_x, "ny": v.normal_y, "nz": v.normal_z}
                    for v in mesh.vertices
                ],
                "faces": [
                    {"indices": list(f.vertex_indices), "material_id": f.material_id}
                    for f in mesh.faces
                ]
            }
            
            with open(output_path, 'w') as f:
                json.dump(mesh_data, f, indent=2)
            
            return True
        except Exception as e:
            print(f"Error exporting mesh: {e}")
            return False

# Global instance
polygon_generator = None

def init_polygon_generator(detail_level: str = "high"):
    """Initialize the polygon generator globally."""
    global polygon_generator
    polygon_generator = PolygonGenerator(detail_level)
    return polygon_generator

def generate_polygons(input_path: str, output_path: str, detail: str = "high") -> bool:
    """
    Generate polygon mesh from an image.
    
    Args:
        input_path: Path to input image
        output_path: Path to save output mesh (JSON)
        detail: Detail level (low, medium, high, ultra)
        
    Returns:
        True if successful, False otherwise
    """
    global polygon_generator
    
    if polygon_generator is None:
        init_polygon_generator(detail)
    
    try:
        mesh = polygon_generator.generate_from_image(input_path)
        return polygon_generator.export_mesh(mesh, output_path)
    except Exception as e:
        print(f"Error generating polygons: {e}")
        return False

if __name__ == "__main__":
    init_polygon_generator("high")
    print("Polygon Generator ready")
