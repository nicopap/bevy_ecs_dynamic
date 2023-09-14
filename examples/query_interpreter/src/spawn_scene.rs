use std::f32::consts::PI;
use std::f32::consts::TAU;

use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Red;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Green;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Blue;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Sphere;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Cube;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct Cycle {
    phase: f32,
    speed: f32,
    ray: Vec3,
    center: Vec3,
    up: Vec3,
}

pub struct SpawnScenePlugin;

impl Plugin for SpawnScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Red>()
            .register_type::<Green>()
            .register_type::<Blue>()
            .register_type::<Sphere>()
            .register_type::<Cube>()
            .register_type::<Cycle>()
            .add_systems(Startup, setup)
            .add_systems(Update, (cycle, animate_light_direction));
    }
}
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(10.0).into()),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..default()
        }),
        ..default()
    });

    // left wall
    let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
    transform.rotate_z(PI / 2.);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
            transform,
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                perceptual_roughness: 1.0,
                ..default()
            }),
            ..default()
        },
        Red,
        Cube,
    ));
    // back (right) wall
    let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
    transform.rotate_x(PI / 2.);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 0.15, 5.0))),
            transform,
            material: materials.add(StandardMaterial {
                base_color: Color::BLUE,
                perceptual_roughness: 1.0,
                ..default()
            }),
            ..default()
        },
        Blue,
        Cube,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes
                .add(Mesh::try_from(shape::Icosphere { radius: 0.6, subdivisions: 5 }).unwrap()),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                perceptual_roughness: 1.0,
                cull_mode: None,
                ..default()
            }),
            ..default()
        },
        Cycle {
            phase: PI / 8.,
            speed: 1.,
            ray: Vec3::Y,
            center: Vec3::new(-2.2, 0.5, 1.0),
            up: Vec3::X,
        },
        Red,
        Sphere,
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes
                .add(Mesh::try_from(shape::Icosphere { radius: 0.4, subdivisions: 5 }).unwrap()),
            material: materials.add(StandardMaterial { base_color: Color::GREEN, ..default() }),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Cycle {
            phase: 0.,
            speed: 3.,
            ray: Vec3::X,
            center: Vec3::new(0.0, 0.5, 0.0),
            up: Vec3::Y,
        },
        Green,
        Sphere,
    ));
    // sphere
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, ..default() })),
            material: materials.add(StandardMaterial { base_color: Color::BLUE, ..default() }),
            ..default()
        },
        Cycle {
            phase: 0.,
            speed: 0.5,
            ray: Vec3::X,
            center: Vec3::new(1.5, 1.0, 1.5),
            up: Vec3::Z,
        },
        Blue,
        Sphere,
    ));

    // ambient light
    commands.insert_resource(AmbientLight { color: Color::ORANGE_RED, brightness: 0.02 });

    // red point light
    commands
        .spawn(PointLightBundle {
            // transform: Transform::from_xyz(5.0, 8.0, 2.0),
            transform: Transform::from_xyz(1.0, 2.0, 0.0),
            point_light: PointLight {
                intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::RED,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.1, ..default() })),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    emissive: Color::rgba_linear(7.13, 0.0, 0.0, 0.0),
                    ..default()
                }),
                ..default()
            });
        });

    // green spot light
    commands
        .spawn(SpotLightBundle {
            transform: Transform::from_xyz(-1.0, 2.0, 0.0)
                .looking_at(Vec3::new(-1.0, 0.0, 0.0), Vec3::Z),
            spot_light: SpotLight {
                intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::GREEN,
                shadows_enabled: true,
                inner_angle: 0.6,
                outer_angle: 0.8,
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                transform: Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    depth: 0.125,
                    radius: 0.1,
                    ..default()
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    emissive: Color::rgba_linear(0.0, 7.13, 0.0, 0.0),
                    ..default()
                }),
                ..default()
            });
        });

    // blue point light
    commands
        .spawn((
            PointLightBundle {
                // transform: Transform::from_xyz(5.0, 8.0, 2.0),
                transform: Transform::from_xyz(0.0, 4.0, 0.0),
                point_light: PointLight {
                    intensity: 1600.0, // lumens - roughly a 100W non-halogen incandescent bulb
                    color: Color::BLUE,
                    shadows_enabled: true,
                    ..default()
                },
                ..default()
            },
            Blue,
        ))
        .with_children(|builder| {
            builder.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.1, ..default() })),
                material: materials.add(StandardMaterial {
                    base_color: Color::BLUE,
                    emissive: Color::rgba_linear(0.0, 0.0, 7.13, 0.0),
                    ..default()
                }),
                ..default()
            });
        });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 300.,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // example instructions
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Blue wall and sphere has the 'Blue' component, same for 'Green' and 'Red'\n",
                TextStyle { font_size: 20.0, ..default() },
            ),
            TextSection::new(
                "Walls have the 'Cube' component\n",
                TextStyle { font_size: 20.0, ..default() },
            ),
            TextSection::new(
                "Spheres have the 'Sphere' component\n",
                TextStyle { font_size: 20.0, ..default() },
            ),
            TextSection::new(
                "Try entering 'mut Transform, Sphere' in blue field and '.scale.x += 1' in yellow field\n",
                TextStyle { font_size: 20.0, ..default() },
            ),
            TextSection::new(
                "'.speed *= 5' and 'Query<&mut Cycle>' is also fun\n",
                TextStyle { font_size: 20.0, ..default() },
            ),
            TextSection::new(
                "\nPress enter while blue field is focused to see the result",
                TextStyle { font_size: 20.0, ..default() },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
}

fn cycle(time: Res<Time>, mut query: Query<(&mut Transform, &mut Cycle)>) {
    let delta = time.delta_seconds();
    for (mut transform, mut cycle) in &mut query {
        let delta = delta * cycle.speed;
        cycle.phase = (cycle.phase + delta) % TAU;
        let rotate = Quat::from_axis_angle(cycle.up, cycle.phase);
        transform.translation = cycle.center + rotate * cycle.ray;
    }
}
