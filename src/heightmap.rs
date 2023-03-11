use bevy::prelude::Vec3;
use ndarray::Array2;

pub struct HeightMap(pub Array2<f32>);

impl HeightMap {
    pub fn vertex_at(&self, x: usize, y: usize) -> Vec3 {
        Vec3 {
            x: x as f32 - self.0.dim().0 as f32 / 2.,
            y: self.0[[x, y]],
            z: y as f32 - self.0.dim().1 as f32 / 2.,
        }
    }

    pub fn height_at(&self, x: usize, y: usize) -> f32 {
        self.0[[x, y]]
    }

    pub fn normal_at(&self, x: usize, y: usize) -> Vec3 {
        let dx = if x == 0 {
            self.0[[x + 1, y]] - self.0[[x, y]]
        } else if x == self.0.dim().0 - 1 {
            self.0[[x - 1, y]] - self.0[[x, y]]
        } else {
            (self.0[[x + 1, y]] - self.0[[x - 1, y]]) / 2.
        };
        let dy = if y == 0 {
            self.0[[x, y + 1]] - self.0[[x, y]]
        } else if y == self.0.dim().0 - 1 {
            self.0[[x, y - 1]] - self.0[[x, y]]
        } else {
            (self.0[[x, y + 1]] - self.0[[x, y - 1]]) / 2.
        };

        Vec3 {
            x: dx,
            y: 1.,
            z: dy,
        }
        .normalize()
    }

    pub fn dim(&self) -> (usize, usize) {
        self.0.dim()
    }
}
