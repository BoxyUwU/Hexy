use std::collections::HashSet;

use crate::{
    hexmap::HexPos, loading::HexObjectAsset, simulation::TileKind, surfaces::CurrentHexMap,
    AppState,
};
use bevy::{
    gltf::{Gltf, GltfMesh},
    math::vec2,
    prelude::*,
    render::{camera::Projection, primitives::Frustum},
    utils::FixedState,
};
use iyes_loopless::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
// use std::f32::consts::PI;

// FIXME stop `x20`'ing these numbers
const HEX_WIDTH: f32 = 40.0;
const HEX_HEIGHT: f32 = 34.0;
const HEX_HORIZ_SPACING: f32 = 30.0;

#[derive(Component, Copy, Clone, Eq, PartialEq)]
struct RenderTileEntity {
    q: i32,
    r: i32,
}

#[derive(Copy, Clone, PartialEq)]
struct WindowSize(f32, f32);

pub fn init_app(app: &mut App) {
    app.add_plugin(InputManagerPlugin::<Action>::default());
    app.add_startup_system(|mut cmds: Commands<'_, '_>, windows: Res<Windows>| {
        let window = windows.get_primary().unwrap();
        cmds.insert_resource(WindowSize(window.width(), window.height()));
    });
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0 / 5.0f32,
    });
    app.add_startup_system(|mut cmds: Commands<'_, '_>| {
        const HALF_SIZE: f32 = 1.0; // TODO: learn about the magic of this magic number
        cmds.spawn_bundle(DirectionalLightBundle {
            transform: Transform::default().with_rotation(Quat::from_euler(
                EulerRot::ZYX,
                0.0,
                2.0 as f32 * std::f32::consts::TAU / 10.0,
                -std::f32::consts::FRAC_PI_4,
            )),
            directional_light: DirectionalLight {
                shadow_projection: OrthographicProjection {
                    left: -HALF_SIZE,
                    right: HALF_SIZE,
                    bottom: -HALF_SIZE,
                    top: HALF_SIZE,
                    near: -10.0 * HALF_SIZE,
                    far: 10.0 * HALF_SIZE,
                    ..default()
                },
                shadows_enabled: true,
                ..default()
            },
            ..default()
        });
    });
    #[derive(SystemLabel)]
    struct UpdateCameraPos;
    app.add_system(
        update_camera_pos
            .run_in_state(AppState::Playing)
            .label(UpdateCameraPos),
    )
    .add_system(
        update_render_entities
            .run_in_state(AppState::Playing)
            .after(UpdateCameraPos),
    )
    // FIXME this ought to be AppState::Playing but no instant commands Sigh bevy
    .add_enter_system(AppState::Loading, default_camera);
}

#[derive(Actionlike, Copy, Clone)]
enum Action {
    MoveCamera,
}

fn default_camera(mut cmds: Commands<'_, '_>) {
    cmds.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, -512.0, 512.0).looking_at(Vec3::ZERO, Vec3::Z),
        projection: Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..default()
        }),
        ..default()
    })
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

// Helper for outlining an area to be hexified/covered in hex visuals
struct HexRect {
    q: i32,
    r: i32,
    width: i32,
    height: i32,
}

impl HexRect {
    fn new(q: i32, r: i32, width: i32, height: i32) -> HexRect {
        HexRect {
            q,
            r,
            width,
            height,
        }
    }

    // Note: inclusive
    fn is_in_rect(&self, q: i32, r: i32) -> bool {
        q >= self.q && q <= self.q + self.width && r >= self.r && r <= self.r + self.height
    }

    // Adds padding around the rect to ensure hexes are drawn just
    // offscreen to prevent visual glitches when moving quickly
    fn is_in_padded_rect(&self, q: i32, r: i32, padding: i32) -> bool {
        let padded_rect = HexRect::new(
            self.q - padding,
            self.r - padding,
            self.width + padding,
            self.height + padding,
        );
        padded_rect.is_in_rect(q, r)
    }
}

fn create_hex_visual(
    selected: bool,
    tile_kind: TileKind,
    hex_object_asset: &HexObjectAsset,
    assets_gltf: &Assets<Gltf>,
    assets_gltfmesh: &Assets<GltfMesh>,
) -> (Handle<Mesh>, Handle<StandardMaterial>) {
    // (unwrap safety: we know the GLTF has loaded already)
    let gltf = assets_gltf.get(&hex_object_asset.0).unwrap();
    let hex_visual = assets_gltfmesh
        .get(&gltf.named_meshes["Cylinder".into()])
        .unwrap();

    // FIXME remove assert
    assert_eq!(hex_visual.primitives.len(), 1);

    (
        hex_visual.primitives[0].mesh.clone(),
        match selected {
            true => gltf.named_materials["Selected".into()].clone(),
            false => gltf.named_materials[tile_kind.material_name().into()].clone(),
        },
    )
}

fn update_render_entities(
    mut cmds: Commands<'_, '_>,
    mut render_entities: Query<(Entity, &mut RenderTileEntity), Without<Camera>>,
    window_size: Res<WindowSize>,
    camera: Query<(&Transform, &Frustum), With<Camera>>,
    asset_server: Res<AssetServer>,
    map: CurrentHexMap<'_, '_>,
    window: Res<Windows>,
    (hex_object_asset, assets_gltf, assets_gltfmesh): (
        Res<HexObjectAsset>,
        Res<Assets<Gltf>>,
        Res<Assets<GltfMesh>>,
    ),
) {
    let plane_center = {
        let (camera_pos, camera_frustum) = camera.single();
        let ray_dir = camera_frustum.planes[4].normal();
        ray_intersects_xy_plane(0.0, camera_pos.translation, ray_dir.into()).unwrap()
    };

    let start_x = plane_center.x - window_size.0 / 2. - HEX_WIDTH * 2.;
    let end_x = plane_center.x + window_size.0 / 2. + HEX_WIDTH * 2.;
    let start_y = plane_center.y - window_size.1 / 2. - HEX_HEIGHT * 2.;
    let end_y = plane_center.y + window_size.1 / 2. + HEX_HEIGHT * 2.;

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
                },));
                // TODO: separate this out into a system that creates and manages a pool of hex meshes, and this system which moves and updates them as needed
            }
            (Some((entity, _)), None) => cmds.entity(entity).despawn(),
            (None, None) => break,
        }
    }

    let map = map.hexmap();

    let camera = camera.single();
    let cam_pos = camera.0;
    let window = window.get_primary().unwrap();

    let selected_hex = match window.cursor_position() {
        Some(cursor_pos) => pos_to_hex_pos(
            cursor_pos.x + cam_pos.translation.x - window.width() / 2.,
            cursor_pos.y + cam_pos.translation.y - window.height() / 2.,
        ),
        None => pos_to_hex_pos(cam_pos.translation.x, cam_pos.translation.y),
    };
    let selected_hex = crate::hexmap::wrap_hex_pos(selected_hex, 16, 16);

    for (entity, render_tile) in render_entities.iter_mut() {
        let tile_pos = HexPos {
            q: render_tile.q,
            r: render_tile.r,
        };
        let wrapped_tile_pos = crate::hexmap::wrap_hex_pos(tile_pos, 16, 16);
        let tile = map.get(wrapped_tile_pos);

        let (mesh, material) = create_hex_visual(
            selected_hex == wrapped_tile_pos,
            tile.kind,
            &hex_object_asset,
            &assets_gltf,
            &assets_gltfmesh,
        );

        cmds.entity(entity).insert_bundle(PbrBundle {
            transform: Transform::from_translation(hex_pos_to_pos(tile_pos).extend(0.0))
                .with_scale(Vec3::ONE * 20.25), // this should be `20` but then we get seams between edges because 3D sucks
            mesh,
            material,
            ..default()
        });
    }
}

fn update_camera_pos(
    mut cam: Query<(&mut Transform, &ActionState<Action>), With<Camera>>,
    map: CurrentHexMap<'_, '_>,
    mut window_size: ResMut<WindowSize>,
    windows: Res<Windows>,
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
        pos.translation = new_pos.extend(pos.translation.z);
    }

    let window = windows.get_primary().unwrap();
    let size = WindowSize(window.width(), window.height());
    if *window_size != size {
        *window_size = size;
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

fn ray_intersects_xy_plane(plane_z: f32, ray_pos: Vec3, ray_dir: Vec3) -> Option<Vec2> {
    if (ray_pos.z < plane_z && ray_dir.z < 0.0) || (ray_pos.z > plane_z && ray_dir.z > 0.0) {
        return None;
    }

    let dist_z = (plane_z - ray_pos.z).abs();
    Some(vec2(
        ray_pos.x + ray_dir.x * dist_z,
        ray_pos.y + ray_dir.y * dist_z,
    ))
}
