use std::collections::HashSet;

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use hexmap::{HexMap, HexPos};
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
        .add_startup_system(default_camera)
        .add_startup_system(init_map)
        .add_system(update_hexmap_render)
        .run();
}

fn default_camera(mut cmds: Commands<'_, '_>) {
    cmds.spawn_bundle(Camera2dBundle { ..default() });
}

fn init_map(mut cmds: Commands<'_, '_>) {
    let map = HexMap::new(
        10,
        10,
        {
            let mut i = 0;
            let cmds = &mut cmds;
            std::iter::from_fn(move || {
                i += 1;
                Some(MyTileData {
                    render: cmds.spawn_bundle((RenderTileEntity,)).id(),
                    kind: if i % 6 == 0 {
                        TileKind::Wall
                    } else {
                        TileKind::Ground
                    },
                })
            })
        }
        .take(10 * 10),
    );
    cmds.insert_resource(map);
}

fn update_hexmap_render(mut cmds: Commands<'_, '_>, map: Res<HexMap<MyTileData>>) {
    let mut hashmap = HashSet::new();
    for q in 0..10 {
        for r in 0..10 {
            let tile = map.get(HexPos { q, r });
            let e = tile.render;
            assert_eq!(hashmap.contains(&e), false);
            hashmap.insert(e);
            cmds.entity(e).insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: tile.kind.color(),
                    custom_size: Some(vec2(30., 30.)),
                    ..default()
                },
                transform: Transform::from_translation(vec3(
                    (q * 32) as f32,
                    (r * 32) as f32 + (q * 16) as f32 - 256.,
                    0.,
                )),
                ..default()
            });
        }
    }
}
