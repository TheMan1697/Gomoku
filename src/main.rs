//! Illustrates bloom post-processing in 2d.

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::DARK_GRAY))
        .insert_resource(MouseState::default()) // 추가: MouseState 리소스 초기화
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(update_bloom_settings)
        .add_system(mouse_movement_system) // 추가: 마우스 이동 시스템
        .add_system(mouse_click_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings {
            intensity: 0.35,
            ..BloomSettings::default()
        }, // 3. Enable bloom for the camera
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(600.)),
        material: materials.add(ColorMaterial::from(Color::rgb(0.92, 0.75, 0.45))),
        ..default()
    });

    let line_width = 2.0;
    let line_length = 600.0 - line_width;
    let line_color = Color::rgb(0.0, 0.0, 0.0);

    for i in 0..19 {
        let position = -300.0 + line_width / 2.0 + i as f32 * (line_length / 18.0);
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad {
                    size: Vec2::new(line_length, line_width),
                    ..Default::default()
                }))
                .into(),
            transform: Transform::from_translation(Vec3::new(0.0, position, 1.0)),
            material: materials.add(ColorMaterial::from(line_color)),
            ..Default::default()
        });
        commands.spawn(MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad {
                    size: Vec2::new(line_width, line_length),
                    ..Default::default()
                }))
                .into(),
            transform: Transform::from_translation(Vec3::new(position, 0.0, 1.0)),
            material: materials.add(ColorMaterial::from(line_color)),
            ..Default::default()
        });
    }

    // UI
    commands.spawn(
        TextBundle::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 18.0,
                color: Color::WHITE,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        }),
    );
}

// ------------------------------------------------------------------------------------------------
#[derive(Default, Clone, Debug)]
struct MouseState {
    cursor_pos: Option<Vec2>,
}

impl Resource for MouseState {}

// 추가: 마우스 이동 시스템
fn mouse_movement_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_state: ResMut<MouseState>,
) {
    for event in cursor_moved_events.iter() {
        mouse_state.cursor_pos = Some(event.position);
    }
}

fn mouse_click_system(
    windows: Query<&Window>,
    mut commands: Commands,
    mouse_button_inputs: Res<Input<MouseButton>>,
    mouse_state: Res<MouseState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if mouse_button_inputs.just_pressed(MouseButton::Left) {
        if let Some(pos) = mouse_state.cursor_pos {
            let window = windows.single();
            let size = Vec2::new(window.width() as f32, window.height() as f32);

            let mut world_pos =
                ((pos / size) - Vec2::new(0.5, 0.5)) * 2.0 * Vec2::new(300.0, 300.0);
            world_pos.y *= 1.0;

            // Round the world_pos to the nearest grid intersection
            let grid_size = 600.0 / 19.0; // Grid size of the Gomoku board
            world_pos /= grid_size;
            world_pos = (world_pos + Vec2::splat(0.5)).floor() * grid_size;


            println!("Mouse pos: {:?}", pos);
            println!("World pos: {:?}", world_pos);

            let stone_color = Color::rgb(0.0, 0.0, 0.0);

            commands.spawn(MaterialMesh2dBundle {
                mesh: meshes
                    .add(Mesh::from(shape::Circle {
                        radius: grid_size * 0.4,
                        ..Default::default()
                    }))
                    .into(),
                transform: Transform::from_xyz(world_pos.x, world_pos.y, 2.0),
                material: materials.add(ColorMaterial::from(stone_color)),
                ..Default::default()
            });
        }
    }
}

fn update_bloom_settings(
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<Camera>>,
    mut text: Query<&mut Text>,
    mut commands: Commands,
    keycode: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let bloom_settings = camera.single_mut();
    let mut text = text.single_mut();
    let text = &mut text.sections[0].value;

    match bloom_settings {
        (entity, Some(mut bloom_settings)) => {
            *text = "BloomSettings (Toggle: Space)\n".to_string();
            text.push_str(&format!("(Q/A) Intensity: {}\n", bloom_settings.intensity));
            text.push_str(&format!(
                "(W/S) Low-frequency boost: {}\n",
                bloom_settings.low_frequency_boost
            ));
            text.push_str(&format!(
                "(E/D) Low-frequency boost curvature: {}\n",
                bloom_settings.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(R/F) High-pass frequency: {}\n",
                bloom_settings.high_pass_frequency
            ));
            text.push_str(&format!(
                "(T/G) Mode: {}\n",
                match bloom_settings.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!(
                "(Y/H) Threshold: {}\n",
                bloom_settings.prefilter_settings.threshold
            ));
            text.push_str(&format!(
                "(U/J) Threshold softness: {}\n",
                bloom_settings.prefilter_settings.threshold_softness
            ));

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).remove::<BloomSettings>();
            }

            let dt = time.delta_seconds();

            if keycode.pressed(KeyCode::A) {
                bloom_settings.intensity -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::Q) {
                bloom_settings.intensity += dt / 10.0;
            }
            bloom_settings.intensity = bloom_settings.intensity.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::S) {
                bloom_settings.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::W) {
                bloom_settings.low_frequency_boost += dt / 10.0;
            }
            bloom_settings.low_frequency_boost = bloom_settings.low_frequency_boost.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::D) {
                bloom_settings.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::E) {
                bloom_settings.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom_settings.low_frequency_boost_curvature =
                bloom_settings.low_frequency_boost_curvature.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::F) {
                bloom_settings.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::R) {
                bloom_settings.high_pass_frequency += dt / 10.0;
            }
            bloom_settings.high_pass_frequency = bloom_settings.high_pass_frequency.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::G) {
                bloom_settings.composite_mode = BloomCompositeMode::Additive;
            }
            if keycode.pressed(KeyCode::T) {
                bloom_settings.composite_mode = BloomCompositeMode::EnergyConserving;
            }

            if keycode.pressed(KeyCode::H) {
                bloom_settings.prefilter_settings.threshold -= dt;
            }
            if keycode.pressed(KeyCode::Y) {
                bloom_settings.prefilter_settings.threshold += dt;
            }
            bloom_settings.prefilter_settings.threshold =
                bloom_settings.prefilter_settings.threshold.max(0.0);

            if keycode.pressed(KeyCode::J) {
                bloom_settings.prefilter_settings.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::U) {
                bloom_settings.prefilter_settings.threshold_softness += dt / 10.0;
            }
            bloom_settings.prefilter_settings.threshold_softness = bloom_settings
                .prefilter_settings
                .threshold_softness
                .clamp(0.0, 1.0);
        }

        (entity, None) => {
            *text = "Bloom: Off (Toggle: Space)".to_string();

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(BloomSettings::default());
            }
        }
    }
}
