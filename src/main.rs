use procedural_generation::{
    generation::{perlin_terrain, NoiseSettings},
    meshing::*,
};

use bevy::{
    pbr::{wireframe::WireframePlugin, DirectionalLightShadowMap},
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_atmosphere::{
    prelude::{AtmosphereCamera, AtmosphereModel, AtmospherePlugin, Nishita},
    system_param::AtmosphereMut,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            },
        }))
        .insert_resource(Msaa::Sample4)
        .insert_resource(AtmosphereModel::default())
        .insert_resource(DirectionalLightShadowMap { size: 4 * 2048 })
        .add_plugin(AtmospherePlugin)
        .add_plugin(WireframePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup_mesh_example)
        .run();
}

fn setup_mesh_example(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atmosphere: AtmosphereMut<Nishita>,
) {
    let (width, height) = (513, 513);

    let height_multiplier = width as f32 * 0.25;

    let mut terrain = perlin_terrain((width, height), 2, NoiseSettings::default());

    terrain.clamp(0.4, 1.);
    terrain.multiply(height_multiplier);

    let mesh = heightmap_to_rtin_mesh(terrain, 0.001 * height_multiplier).into_render_mesh(false);

    add_camera(&mut commands, height_multiplier, 2000.);
    add_lights(&mut commands, atmosphere, -0.1 * std::f32::consts::PI);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("ffd891").unwrap(),
                // metallic: 0.,
                // reflectance: 0.,
                perceptual_roughness: 0.5,
                unlit: false,
                ..default()
            }),
            ..default()
        })
        .insert(bevy::pbr::wireframe::Wireframe);
}

fn add_camera(commands: &mut Commands, start_height: f32, view_distance: f32) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3 {
                x: 0.0,
                y: start_height,
                z: 0.0,
            }),
            ..default()
        })
        .insert(FlyCamera::default())
        .insert(AtmosphereCamera::default())
        .insert(Projection::Perspective(PerspectiveProjection {
            far: view_distance,
            ..default()
        }));
}

fn add_lights(commands: &mut Commands, mut atmosphere: AtmosphereMut<Nishita>, sun_angle: f32) {
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.05,
    });

    let sun_rot = Quat::from_rotation_x(sun_angle);

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            // color: todo!(),
            // shadow_depth_bias: todo!(),
            // shadow_normal_bias: todo!(),
            ..default()
        },
        transform: Transform::from_rotation(sun_rot),
        ..default()
    });

    atmosphere.sun_position = sun_rot * Vec3::Z;
    atmosphere.sun_intensity = 10.;
}
