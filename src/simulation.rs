use crate::{
    hexmap::HexMap,
    surfaces::{SelectedSurface, Surfaces},
    AppState,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use iyes_loopless::prelude::*;

#[derive(Debug, Inspectable)]
pub struct MyTileData {
    pub height: u8,
    pub kind: TileKind,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Inspectable)]
pub enum TileKind {
    Water,
    Rock,
}

impl TileKind {
    pub fn material_name(&self) -> &'static str {
        match self {
            Self::Water => "Water",
            Self::Rock => "Rock",
        }
    }
}

pub fn init_app(app: &mut App) {
    app.add_enter_system(AppState::Loading, init_map)
        .add_system(simulate_surfaces.run_in_state(AppState::Playing));
}

fn init_map(mut cmds: Commands<'_, '_>) {
    let map = HexMap::new(
        16,
        16,
        {
            let mut i = 0;
            std::iter::from_fn(move || {
                i += 1;
                Some(MyTileData {
                    height: (i % 6) as u8,
                    kind: if i % 6 == 0 && i > 100 && i < 150 {
                        TileKind::Rock
                    } else {
                        TileKind::Water
                    },
                })
            })
        }
        .take(16 * 16),
    );
    let mut surfaces = Surfaces::new();
    surfaces.new_surface(World::new(), map);
    add_systems(&mut surfaces);
    cmds.insert_resource(surfaces);
    cmds.insert_resource(SelectedSurface(0));
}

fn simulate_surfaces(mut surfaces: ResMut<Surfaces>) {
    surfaces.simulate_step();
}

pub fn add_systems(_surfaces: &mut Surfaces) {}
