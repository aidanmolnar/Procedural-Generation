use super::TerrainMeshData;
use crate::HeightMap;

pub fn heightmap_to_grid_mesh(terrain: HeightMap) -> TerrainMeshData {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();

    for x in 0..terrain.dim().1 {
        for y in 0..terrain.dim().0 {
            vertices.push(terrain.vertex_at(x, y));
            normals.push(terrain.normal_at(x, y));
        }
    }

    let mut triangles = Vec::new();
    let idx = |x: usize, y: usize| -> u32 { (x + y * terrain.dim().0) as u32 };

    for x in 0..terrain.dim().0 - 1 {
        for y in 0..terrain.dim().1 - 1 {
            // Add quad to indices
            triangles.extend([idx(x, y), idx(x + 1, y), idx(x + 1, y + 1)]);
            triangles.extend([idx(x, y), idx(x + 1, y + 1), idx(x, y + 1)]);
        }
    }

    TerrainMeshData {
        vertices,
        triangles,
        normals,
    }
}
