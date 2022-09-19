use bevy::{math::vec2, prelude::*, sprite::Anchor};
use hexmap::{HexMap, HexPos};
use leafwing_input_manager::{prelude::*, user_input::InputKind};
// use iyes_loopless::prelude::*;

pub mod hexmap;

#[derive(Component)]
struct MyTileData {
    kind: TileKind,
}

enum TileKind {
    Ground,
    Wall,
}

impl TileKind {
    pub fn color(&self) -> Color {
        match self {
            Self::Ground => Color::BEIGE,
            Self::Wall => Color::DARK_GRAY,
        }
    }
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct RenderTileEntity {
    q: i32,
    r: i32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(default_camera)
        .add_startup_system(init_map)
        .add_system(update_window_size)
        .add_system(update_render_entities)
        .add_system(update_hexmap_render)
        .add_system(update_camera_pos)
        .run();
}

#[derive(Actionlike, Copy, Clone)]
enum Action {
    MoveCamera,
}

fn default_camera(mut cmds: Commands<'_, '_>) {
    cmds.spawn_bundle(Camera2dBundle { ..default() })
        .insert_bundle(InputManagerBundle {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::left_stick(), Action::MoveCamera)
                .insert(
                    VirtualDPad {
                        up: InputKind::GamepadButton(GamepadButtonType::DPadUp),
                        down: InputKind::GamepadButton(GamepadButtonType::DPadDown),
                        left: InputKind::GamepadButton(GamepadButtonType::DPadLeft),
                        right: InputKind::GamepadButton(GamepadButtonType::DPadRight),
                    },
                    Action::MoveCamera,
                )
                .insert(
                    VirtualDPad {
                        up: InputKind::Keyboard(KeyCode::W),
                        down: InputKind::Keyboard(KeyCode::S),
                        left: InputKind::Keyboard(KeyCode::A),
                        right: InputKind::Keyboard(KeyCode::D),
                    },
                    Action::MoveCamera,
                )
                .build(),
        });
}

fn init_map(mut cmds: Commands<'_, '_>, windows: Res<Windows>) {
    let map = HexMap::new(
        16,
        16,
        {
            let mut i = 0;
            std::iter::from_fn(move || {
                i += 1;
                Some(MyTileData {
                    kind: if i % 6 == 0 && i > 100 && i < 150 {
                        TileKind::Wall
                    } else {
                        TileKind::Ground
                    },
                })
            })
        }
        .take(16 * 16),
    );
    cmds.insert_resource(map);
    let window = windows.get_primary().unwrap();
    cmds.insert_resource(WindowSize(window.width(), window.height()));
}

#[derive(Copy, Clone, PartialEq)]
struct WindowSize(f32, f32);

fn update_window_size(mut window_size: ResMut<WindowSize>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let size = WindowSize(window.width(), window.height());
    if *window_size != size {
        *window_size = size;
    }
}

fn update_render_entities(
    mut cmds: Commands<'_, '_>,
    mut render_entities: Query<(Entity, &mut RenderTileEntity)>,
    window_size: Res<WindowSize>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_pos = camera.single();
    let start_x = camera_pos.translation.x - window_size.0 / 2. - 32.;
    let end_x = camera_pos.translation.x + window_size.0 / 2. + 32.;
    let start_y = camera_pos.translation.y - window_size.1 / 2. - 32.;
    let end_y = camera_pos.translation.y + window_size.1 / 2. + 32.;

    let mut tiles = vec![];

    let mut current_y = 0;
    while start_y + current_y as f32 * 32. <= end_y {
        let mut current_x = 0;
        while start_x + current_x as f32 * 32. <= end_x {
            let y_offset = (current_x % 2) * 16;
            let x = start_x + current_x as f32 * 32.;
            let y = start_y + current_y as f32 * 32. + y_offset as f32;

            let hex_pos = pos_to_hex_pos(x, y);
            tiles.push(hex_pos);

            current_x += 1;
        }
        current_y += 1;
    }
    let mut tile_iter = tiles.into_iter();
    let mut query_iter = render_entities.iter_mut();

    loop {
        match (query_iter.next(), tile_iter.next()) {
            (Some((_, mut tile_pos)), Some(tile)) => {
                tile_pos.q = tile.q;
                tile_pos.r = tile.r;
            }
            (None, Some(tile)) => {
                cmds.spawn_bundle((RenderTileEntity {
                    q: tile.q,
                    r: tile.r,
                },));
            }
            (Some((entity, _)), None) => cmds.entity(entity).despawn(),
            (None, None) => break,
        }
    }
}

fn update_hexmap_render(
    render_tiles: Query<(Entity, &RenderTileEntity)>,
    camera: Query<&Transform, With<Camera>>,
    mut cmds: Commands<'_, '_>,
    map: Res<HexMap<MyTileData>>,
    window: Res<Windows>,
) {
    let cam_pos = camera.single();
    let window = window.get_primary().unwrap();

    let selected_hex = match window.cursor_position() {
        Some(cursor_pos) => pos_to_hex_pos(
            cursor_pos.x + cam_pos.translation.x - window.width() / 2.,
            cursor_pos.y + cam_pos.translation.y - window.height() / 2.,
        ),
        None => pos_to_hex_pos(cam_pos.translation.x, cam_pos.translation.y),
    };
    let selected_hex = hexmap::wrap_hex_pos(selected_hex, 16, 16);

    for (entity, render_tile) in render_tiles.iter() {
        let tile_pos = HexPos {
            q: render_tile.q,
            r: render_tile.r,
        };
        let wrapped_tile_pos = hexmap::wrap_hex_pos(tile_pos, 16, 16);
        let tile = map.get(wrapped_tile_pos);

        let color = match selected_hex == wrapped_tile_pos {
            true => Color::RED,
            false => tile.kind.color(),
        };

        cmds.entity(entity).insert_bundle(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(vec2(30., 30.)),
                anchor: Anchor::BottomLeft,
                ..default()
            },
            transform: Transform::from_translation(hex_pos_to_pos(tile_pos, 32, 32).extend(0.0)),
            ..default()
        });
    }
}

/// x y should be in "world space", not screen space.
fn pos_to_hex_pos(x: f32, y: f32) -> HexPos {
    let x = f32::floor(x / 32.) as i32;
    let y = y - x as f32 * 16.;
    let y = f32::floor(y / 32.) as i32;
    HexPos { q: x, r: y }
}

fn hex_pos_to_pos(pos: HexPos, hex_width: u32, hex_height: u32) -> Vec2 {
    assert_eq!(hex_height % 2, 0);
    let x = pos.q * hex_width as i32;
    let y = pos.r * hex_height as i32 + pos.q * (hex_height as i32 / 2);
    vec2(x as f32, y as f32)
}

fn update_camera_pos(
    mut cam: Query<(&mut Transform, &ActionState<Action>), With<Camera>>,
    map: Res<HexMap<MyTileData>>,
) {
    const CAM_SPEED: f32 = 4.0;

    let (mut pos, actions) = cam.single_mut();
    if actions.pressed(Action::MoveCamera) {
        let movement = actions.clamped_axis_pair(Action::MoveCamera).unwrap().xy();
        pos.translation += movement.extend(0.0) * CAM_SPEED;
        pos.translation = pos.translation.as_ivec3().as_vec3();
    }

    // wrap pos around map
    let hex_pos = pos_to_hex_pos(pos.translation.x, pos.translation.y);
    let wrapped_pos = hexmap::wrap_hex_pos(hex_pos, map.width() as u32, map.height() as u32);
    if hex_pos != wrapped_pos {
        let snapped_pos = hex_pos_to_pos(hex_pos, 32, 32);
        let offset = snapped_pos - pos.translation.truncate();
        let new_pos = hex_pos_to_pos(wrapped_pos, 32, 32) - offset;
        pos.translation = new_pos.extend(0.0);
    }
}
