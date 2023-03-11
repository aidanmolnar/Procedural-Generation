use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use ndarray::Array2;

use crate::generation::NoiseSettings;

use super::generation::perlin_terrain;

#[allow(dead_code)]
fn setup_sprite_example(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let (width, height) = (512, 512);
    let terrain = perlin_terrain((width, height), 2, NoiseSettings::default());

    let bytes = array_to_pixels(terrain.0);

    let image = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8Unorm,
    );

    let image_handle = images.add(image);

    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: image_handle,
        ..Default::default()
    });
}

#[allow(dead_code)]
fn array_to_pixels(array: Array2<f32>) -> Vec<u8> {
    let (width, height) = array.dim();

    let mut bytes = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let val = (255. * array[[x, y]].clamp(0., 1.)) as u8;

            bytes.push(val);
            bytes.push(val);
            bytes.push(val);
            bytes.push(val);
        }
    }

    bytes
}
