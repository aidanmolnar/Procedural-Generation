mod grid;
mod rtin;

pub use grid::heightmap_to_grid_mesh;
pub use rtin::heightmap_to_rtin_mesh;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

pub struct TerrainMeshData {
    vertices: Vec<Vec3>,
    triangles: Vec<u32>,
    normals: Vec<Vec3>,
}

impl TerrainMeshData {
    pub fn into_render_mesh(self, color_by_normals: bool) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);

        if color_by_normals {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                self.normals
                    .iter()
                    .map(|v| Vec4::new(v.x, v.y, v.z, 1.))
                    .collect::<Vec<_>>(),
            );
        };
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);

        mesh.set_indices(Some(Indices::U32(self.triangles)));

        mesh
    }
}
