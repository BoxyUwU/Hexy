use bevy::{gltf::Gltf, prelude::*};
use iyes_loopless::prelude::*;

use crate::AppState;

// Drawing from: https://bevy-cheatbook.github.io/3d/gltf.html
// Note: Gltf is not Inspectable
pub struct HexObjectAsset(pub Handle<Gltf>);

pub fn init_app(app: &mut App) {
    app.add_enter_system(AppState::Loading, load_hex_gltf)
        .add_system(try_transition.run_in_state(AppState::Loading));
}

fn load_hex_gltf(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let gltf = asset_server.load("tile.glb");
    cmds.insert_resource(HexObjectAsset(gltf));
}

fn try_transition(
    mut cmds: Commands<'_, '_>,
    res: Res<HexObjectAsset>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf) = assets_gltf.get(&res.0) {
        dbg!(gltf);
        cmds.insert_resource(NextState(AppState::Playing));
    }
}
