use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use iyes_loopless::prelude::*;

pub mod draw;
pub mod hexmap;
pub mod loading;
pub mod simulation;
pub mod surfaces;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum AppState {
    Loading,
    Playing,
}

fn main() {
    let mut app = App::new();
    app.add_loopless_state(AppState::Loading)
        .add_plugins(DefaultPlugins)
        // Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(LogDiagnosticsPlugin::default());
    draw::init_app(&mut app);
    simulation::init_app(&mut app);
    loading::init_app(&mut app);
    app.run();
}
