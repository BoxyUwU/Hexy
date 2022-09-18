use bevy::{math::vec2, prelude::*, sprite::Anchor};
use hexmap::{HexMap, HexPos};
use leafwing_input_manager::{prelude::*, user_input::InputKind};
// use iyes_loopless::prelude::*;

pub mod hexmap;

#[derive(Component)]
struct MyTileData {
    render: Entity,
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

#[derive(Component)]
struct RenderTileEntity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(default_camera)
        .add_startup_system(init_map)
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

fn init_map(mut cmds: Commands<'_, '_>) {
    let map = HexMap::new(
        16,
        16,
        {
            let mut i = 0;
            let cmds = &mut cmds;
            std::iter::from_fn(move || {
                i += 1;
                Some(MyTileData {
                    render: cmds.spawn_bundle((RenderTileEntity,)).id(),
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
}

fn update_hexmap_render(
    camera: Query<&Transform, With<Camera>>,
    mut cmds: Commands<'_, '_>,
    map: Res<HexMap<MyTileData>>,
    window: Res<Windows>,
) {
    let cam_pos = camera.single();
    let window = window.get_primary().unwrap();

    let selected_hex = match window.cursor_position() {
        Some(cursor_pos) => {
            dbg!(cursor_pos);
            pos_to_hex_pos(
                cursor_pos.x + cam_pos.translation.x - window.width() / 2.,
                cursor_pos.y + cam_pos.translation.y - window.height() / 2.,
            )
        }
        None => pos_to_hex_pos(cam_pos.translation.x, cam_pos.translation.y),
    };
    let selected_hex = hexmap::wrap_hex_pos(selected_hex, 16, 16);

    for q in 0..16 {
        for r in 0..16 {
            let tile = map.get(HexPos { q, r });
            let e = tile.render;

            let color = if selected_hex == (HexPos { q, r }) {
                Color::RED
            } else {
                tile.kind.color()
            };

            cmds.entity(e).insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(vec2(30., 30.)),
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                transform: Transform::from_translation(
                    hex_pos_to_pos(HexPos { q, r }, 32, 32).extend(0.0),
                ),
                ..default()
            });
        }
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
