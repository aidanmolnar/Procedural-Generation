use bevy::prelude::Vec3;
use ndarray::Array2;

use super::TerrainMeshData;
use crate::HeightMap;

// A position in the terrain/error grid
#[derive(Clone, Copy, Debug)]
struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    fn idx(&self) -> [usize; 2] {
        [self.x, self.y]
    }

    fn manhattan_distance(&self, other: &Self) -> usize {
        ((self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()) as usize
    }
}

// a and b are non 90 degree corners of right triangle
// c is 90 degree corner
#[derive(Clone, Copy, Debug)]
struct Triangle {
    a: Coord,
    b: Coord,
    c: Coord,
}

fn bottom_left_root(tile_size: usize) -> Triangle {
    Triangle {
        a: Coord { x: 0, y: 0 },
        b: Coord {
            x: tile_size,
            y: tile_size,
        },
        c: Coord { x: tile_size, y: 0 },
    }
}

fn top_right_root(tile_size: usize) -> Triangle {
    Triangle {
        a: Coord {
            x: tile_size,
            y: tile_size,
        },
        b: Coord { x: 0, y: 0 },
        c: Coord { x: 0, y: tile_size },
    }
}

impl Triangle {
    fn from_id(mut id: usize, tile_size: usize) -> Self {
        // Determine starting triangle
        let mut tri = if id & 1 > 0 {
            // Bottom-left triangle
            bottom_left_root(tile_size)
        } else {
            // Top-right triangle
            top_right_root(tile_size)
        };

        // Subdivide triangle
        while (id / 2) > 1 {
            id /= 2;

            tri = if id & 1 > 0 {
                // Left half
                tri.to_left_child()
            } else {
                // Right half
                tri.to_right_child()
            };
        }

        tri
    }

    // Returns midpoint of hypotenuse
    fn midpoint(&self) -> Coord {
        Coord {
            x: (self.a.x + self.b.x) / 2,
            y: (self.a.y + self.b.y) / 2,
        }
    }

    // Returns 90 degree corner of triangle
    // fn c(&self) -> Coord {
    //     let m = self.midpoint();
    //     Coord {
    //         x: m.x + m.y - self.a.y,
    //         y: m.y + self.a.x - m.x,
    //     }
    // }

    // Returns index into error/terrain array of left child
    // (Midpoint of child)
    fn left_child_idx(&self) -> [usize; 2] {
        [(self.a.x + self.c.x) / 2, (self.a.y + self.c.y) / 2]
    }

    // Returns index into error/terrain array of right child
    // (Midpoint of child)
    fn right_child_idx(&self) -> [usize; 2] {
        [(self.b.x + self.c.x) / 2, (self.b.y + self.c.y) / 2]
    }

    // Checks if triangle is not the lowest resolution
    fn not_leaf(&self) -> bool {
        self.a.manhattan_distance(&self.c) > 1
    }

    #[inline(always)]
    fn to_left_child(mut self) -> Self {
        let m = self.midpoint();
        self.b = self.a;
        self.a = self.c;
        self.c = m;
        self
    }

    #[inline(always)]
    fn to_right_child(mut self) -> Self {
        let m = self.midpoint();
        self.a = self.b;
        self.b = self.c;
        self.c = m;
        self
    }
}

struct Tile {
    terrain: HeightMap,
    errors: Array2<f32>,
}

struct TileMeshInfo {
    num_vertices: usize,
    num_triangles: usize,
    max_error: f32,
    indices: Array2<usize>,
}

struct TileMeshData {
    vertices: Vec<Vec3>,
    normal_sums: Vec<Vec3>,
    normal_counts: Vec<usize>,
    triangles: Vec<u32>,
    info: TileMeshInfo,
}

impl TileMeshData {
    fn new(mesh_info: TileMeshInfo) -> Self {
        TileMeshData {
            vertices: vec![Vec3::ZERO; mesh_info.num_vertices],
            normal_sums: vec![Vec3::ZERO; mesh_info.num_vertices],
            normal_counts: vec![0; mesh_info.num_vertices],
            triangles: vec![0; mesh_info.num_triangles * 3],
            info: mesh_info,
        }
    }
}

impl From<TileMeshData> for TerrainMeshData {
    fn from(mut mesh_data: TileMeshData) -> TerrainMeshData {
        // Average normals in place
        for (normal, count) in mesh_data
            .normal_sums
            .iter_mut()
            .zip(mesh_data.normal_counts.into_iter())
        {
            *normal /= count as f32;
            *normal = normal.normalize();
        }

        TerrainMeshData {
            vertices: mesh_data.vertices,
            triangles: mesh_data.triangles,
            normals: mesh_data.normal_sums,
        }
    }
}

impl Tile {
    fn new(terrain: HeightMap) -> Self {
        let grid_size = terrain.dim().0;
        let tile_size = grid_size - 1;

        assert!(terrain.dim().0 == terrain.dim().1, "Terrain must be square");
        assert!(tile_size.is_power_of_two(), "Terrain size must be 2^k+1");

        // TODO: explain these formulas
        let max_num_triangles = tile_size * tile_size * 2;
        let num_parent_triangles = max_num_triangles - tile_size * tile_size;

        let mut errors: Array2<f32> = Array2::zeros(terrain.dim());

        // Calculate the error for each possible triangles, starting from the smallest level
        for i in (0..max_num_triangles - 1).rev() {
            let tri = Triangle::from_id(i + 2, grid_size - 1);

            // Calculate error in the middle of the long edge of the triangle
            let interpolated_height = (terrain.0[tri.a.idx()] + terrain.0[tri.b.idx()]) / 2.;
            let middle_index = tri.midpoint().idx();
            errors[middle_index] = (interpolated_height - terrain.0[middle_index])
                .abs()
                .max(errors[middle_index]);

            if i < num_parent_triangles {
                // Parent triangles accumulate error of children
                errors[middle_index] = errors[middle_index]
                    .max(errors[tri.left_child_idx()])
                    .max(errors[tri.right_child_idx()]);
            }
        }

        Self { errors, terrain }
    }

    fn grid_size(&self) -> usize {
        self.terrain.dim().0
    }

    fn get_mesh(&self, max_error: f32) -> TerrainMeshData {
        let size = self.grid_size();
        let tile_size = size - 1;

        // Use an index grid to keep track of vertices that were already used to
        // avoid duplication
        let mut mesh_info = TileMeshInfo {
            num_vertices: 0,
            num_triangles: 0,
            max_error,
            indices: Array2::zeros((size, size)),
        };

        // Retrieve mesh in two stages that both traverse the error map:
        // - countElements: find used vertices (and assign each an index), and count triangles (for minimum allocation)
        // - processTriangle: fill the allocated vertices & triangles typed arrays

        self.count_elements(&mut mesh_info, bottom_left_root(tile_size));
        self.count_elements(&mut mesh_info, top_right_root(tile_size));

        let mut mesh_data = TileMeshData::new(mesh_info);

        self.process_triangle(&mut mesh_data, bottom_left_root(tile_size));
        self.process_triangle(&mut mesh_data, top_right_root(tile_size));

        mesh_data.into()
    }

    fn count_elements(&self, mesh_info: &mut TileMeshInfo, tri: Triangle) {
        if tri.not_leaf() && (self.errors[tri.midpoint().idx()] > mesh_info.max_error) {
            self.count_elements(mesh_info, tri.to_left_child());
            self.count_elements(mesh_info, tri.to_right_child());
        } else {
            if mesh_info.indices[tri.a.idx()] == 0 {
                // Add 1 first because 0 is reserved to mean no vertex
                mesh_info.num_vertices += 1;
                mesh_info.indices[tri.a.idx()] = mesh_info.num_vertices;
            }

            if mesh_info.indices[tri.b.idx()] == 0 {
                mesh_info.num_vertices += 1;
                mesh_info.indices[tri.b.idx()] = mesh_info.num_vertices;
            }

            if mesh_info.indices[tri.c.idx()] == 0 {
                mesh_info.num_vertices += 1;
                mesh_info.indices[tri.c.idx()] = mesh_info.num_vertices;
            }
            mesh_info.num_triangles += 1;
        }
    }

    fn process_triangle(&self, mesh_data: &mut TileMeshData, tri: Triangle) {
        if tri.not_leaf() && (self.errors[tri.midpoint().idx()] > mesh_data.info.max_error) {
            // Triangle doesn't approximate the surface well enough; drill down further
            self.process_triangle(mesh_data, tri.to_left_child());
            self.process_triangle(mesh_data, tri.to_right_child());
        } else {
            // Add a triangle
            // Subtract 1 because 0 is reserved to mean no vertex
            let a = mesh_data.info.indices[tri.a.idx()] - 1;
            let b = mesh_data.info.indices[tri.b.idx()] - 1;
            let c = mesh_data.info.indices[tri.c.idx()] - 1;

            mesh_data.vertices[a] = self.terrain.vertex_at(tri.a.x, tri.a.y);
            mesh_data.vertices[b] = self.terrain.vertex_at(tri.b.x, tri.b.y);
            mesh_data.vertices[c] = self.terrain.vertex_at(tri.c.x, tri.c.y);

            // Track sum of normals of surrounding triangles and count to average at each vertex later
            let tangent_1 = mesh_data.vertices[a] - mesh_data.vertices[b];
            let tangent_2 = mesh_data.vertices[a] - mesh_data.vertices[c];
            let normal = tangent_1.cross(tangent_2);
            mesh_data.normal_sums[a] += normal;
            mesh_data.normal_sums[b] += normal;
            mesh_data.normal_sums[c] += normal;
            mesh_data.normal_counts[a] += 1;
            mesh_data.normal_counts[b] += 1;
            mesh_data.normal_counts[c] += 1;

            mesh_data.triangles.push(a as u32);
            mesh_data.triangles.push(b as u32);
            mesh_data.triangles.push(c as u32);
        }
    }
}

pub fn heightmap_to_rtin_mesh(terrain: HeightMap, max_error: f32) -> TerrainMeshData {
    // Generate RTIN hierarchy from terrain height map
    let tile = Tile::new(terrain);

    // Get a mesh (vertices and triangles indices)
    tile.get_mesh(max_error)
}
