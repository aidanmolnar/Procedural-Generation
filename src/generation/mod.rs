use super::heightmap::HeightMap;

use ndarray::prelude::*;
use noise::{NoiseFn, Perlin};

pub struct NoiseSettings {
    scale: f32,
}

impl Default for NoiseSettings {
    fn default() -> Self {
        Self { scale: 5e-3 }
    }
}

pub fn perlin_terrain(
    (width, height): (usize, usize),
    seed: u32,
    noise_settings: NoiseSettings,
) -> HeightMap {
    let octaves = 8;
    let scale_start = noise_settings.scale;

    let perlin = Perlin::new(seed);

    let mut data = Array::zeros((width, height));

    for y in 0..height {
        for x in 0..width {
            let mut scale = 1.;

            for i in 0..octaves {
                data[[x, y]] += scale
                    * perlin.get([
                        (i as f32 * 1000. + scale_start / scale * x as f32) as f64,
                        (scale_start / scale * y as f32) as f64,
                    ]) as f32;
                scale /= 2.;
            }
        }
    }

    // Calculate the maximum magnitude of the terrain
    let (max_magnitude, _) = (0..octaves).fold((0.0, 1.0), |(max_magnitude, scale), _| {
        (max_magnitude + scale, scale / 2.0)
    });

    // Covert the values from -max_magnitude..max_magnitude to 0..1
    HeightMap((data / max_magnitude + 1.) / 2.)
}

impl HeightMap {
    pub fn multiply(&mut self, mult: f32) {
        self.0.map_mut(|v| *v *= mult);
    }

    pub fn clamp(&mut self, min: f32, max: f32) {
        self.0.map_mut(|v| *v = v.clamp(min, max));
    }
}
