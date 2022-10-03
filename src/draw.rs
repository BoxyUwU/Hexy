use std::collections::HashSet;

use crate::{hexmap::HexPos, surfaces::CurrentHexMap, Action};
use bevy::{
    math::{vec2, vec3},
    prelude::*,
    sprite::Anchor,
    utils::FixedState,
};
use leafwing_input_manager::prelude::*;
// use std::f32::consts::PI;
// use iyes_loopless::prelude::*;

// FIXME stop `x10`'ing these numbers
const HEX_WIDTH: f32 = 20.0;
const HEX_HEIGHT: f32 = 17.0;
const HEX_HORIZ_SPACING: f32 = 15.0;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct RenderTileEntity {
    q: i32,
    r: i32,
}

#[derive(Copy, Clone, PartialEq)]
struct WindowSize(f32, f32);

pub fn init_app(app: &mut App) {
    app.add_startup_system(|mut cmds: Commands<'_, '_>, windows: Res<Windows>| {
        let window = windows.get_primary().unwrap();
        cmds.insert_resource(WindowSize(window.width(), window.height()));
    });
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    });
    app.add_system(update_camera_pos)
        .add_system(animate_light_direction)
        .add_system(update_window_size.after(update_camera_pos))
        .add_system(update_render_entities.after(update_window_size))
        .add_system(update_hexmap_render.after(update_render_entities));
}

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
    asset_server: Res<AssetServer>,
) {
    let camera_pos = camera.single();
    let start_x = camera_pos.translation.x - window_size.0 / 2. - HEX_WIDTH;
    let end_x = camera_pos.translation.x + window_size.0 / 2. + HEX_WIDTH;
    let start_y = camera_pos.translation.y - window_size.1 / 2. - HEX_HEIGHT;
    let end_y = camera_pos.translation.y + window_size.1 / 2. + HEX_HEIGHT;

    let mut tiles = HashSet::with_hasher(FixedState);
    let mut current_y = 0;
    while start_y + current_y as f32 * HEX_HEIGHT <= end_y {
        let mut current_x = 0;
        while start_x + current_x as f32 * HEX_HORIZ_SPACING <= end_x {
            let y_offset = (current_x % 2) as f32 * HEX_HEIGHT / 2.0;
            let x = start_x + current_x as f32 * HEX_HORIZ_SPACING;
            let y = start_y + current_y as f32 * HEX_HEIGHT + y_offset;

            let hex_pos = pos_to_hex_pos(x, y);
            tiles.insert(hex_pos);

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
                },))
                    .insert_bundle(SceneBundle {
                        scene: asset_server.load("tile.glb#Scene0"),
                        ..default()
                    });
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
    map: CurrentHexMap<'_, '_>,
    window: Res<Windows>,
    asset_server: Res<AssetServer>,
) {
    let map = map.hexmap();

    let cam_pos = camera.single();
    let window = window.get_primary().unwrap();

    let selected_hex = match window.cursor_position() {
        Some(cursor_pos) => pos_to_hex_pos(
            cursor_pos.x + cam_pos.translation.x - window.width() / 2.,
            cursor_pos.y + cam_pos.translation.y - window.height() / 2.,
        ),
        None => pos_to_hex_pos(cam_pos.translation.x, cam_pos.translation.y),
    };
    let selected_hex = crate::hexmap::wrap_hex_pos(selected_hex, 16, 16);

    for (entity, render_tile) in render_tiles.iter() {
        let tile_pos = HexPos {
            q: render_tile.q,
            r: render_tile.r,
        };
        let wrapped_tile_pos = crate::hexmap::wrap_hex_pos(tile_pos, 16, 16);
        let tile = map.get(wrapped_tile_pos);

        let color = match selected_hex == wrapped_tile_pos {
            true => Color::RED,
            false => tile.kind.color(),
        };

        // cmds.entity(entity).insert_bundle(SpriteBundle {
        //     sprite: Sprite {
        //         color,
        //         custom_size: Some(vec2(30., 30.)),
        //         anchor: Anchor::BottomLeft,
        //         ..default()
        //     },
        //     transform: Transform::from_translation(hex_pos_to_pos(tile_pos, 32, 32).extend(0.0)),
        //     ..default()
        // });
        cmds.entity(entity).insert(
            Transform::from_translation(hex_pos_to_pos(tile_pos).extend(0.0))
                .with_scale(Vec3::ONE * 10.25), // this should be `10` but then we get seams between edges because 3D sucks
        );
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}

fn animate_camera_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}

fn update_camera_pos(
    mut cam: Query<(&mut Transform, &ActionState<Action>), With<Camera>>,
    map: CurrentHexMap<'_, '_>,
) {
    let map = map.hexmap();
    const CAM_SPEED: f32 = 4.0;

    let (mut pos, actions) = cam.single_mut();
    if actions.pressed(Action::MoveCamera) {
        let movement = actions.clamped_axis_pair(Action::MoveCamera).unwrap().xy();
        pos.translation += movement.extend(0.0) * CAM_SPEED;
        pos.translation = pos.translation.as_ivec3().as_vec3();
    }

    // wrap pos around map
    let hex_pos = pos_to_hex_pos(pos.translation.x, pos.translation.y);
    let wrapped_pos = crate::hexmap::wrap_hex_pos(hex_pos, map.width() as u32, map.height() as u32);
    if hex_pos != wrapped_pos {
        let snapped_pos = hex_pos_to_pos(hex_pos);
        let offset = snapped_pos - pos.translation.truncate();
        let new_pos = hex_pos_to_pos(wrapped_pos) - offset;
        pos.translation = new_pos.extend(128.0);
    }
}

/// x y should be in "world space", not screen space.
fn pos_to_hex_pos(x: f32, y: f32) -> HexPos {
    let q = f32::round(x / HEX_HORIZ_SPACING) as i32;
    let vert_offset = q as f32 * HEX_HEIGHT / 2.0;
    let r = f32::round((y - vert_offset) / HEX_HEIGHT) as i32;
    HexPos { q, r }
}

fn hex_pos_to_pos(pos: HexPos) -> Vec2 {
    let x = pos.q as f32 * HEX_HORIZ_SPACING;
    let y = pos.q as f32 * HEX_HEIGHT / 2.0 + pos.r as f32 * HEX_HEIGHT;
    vec2(x, y)
}
