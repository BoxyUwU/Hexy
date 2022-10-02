use bevy::prelude::*;
use hexmap::HexMap;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use surfaces::{SelectedSurface, Surfaces};
// use iyes_loopless::prelude::*;

pub mod draw;
pub mod hexmap;
pub mod simulation;
pub mod surfaces;

pub struct MyTileData {
    kind: TileKind,
}

pub enum TileKind {
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

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(default_lighting)
        .add_startup_system(default_camera)
        .add_startup_system(init_map)
        .add_system(simulate_surfaces);
    draw::init_app(&mut app);
    app.run();
}

#[derive(Actionlike, Copy, Clone)]
enum Action {
    MoveCamera,
}

fn default_lighting(mut cmds: Commands<'_, '_>) {
    const HALF_SIZE: f32 = 1.0; // TODO: learn about the magic of this magic number
    cmds.spawn_bundle(DirectionalLightBundle {
        // TODO: probably should be in draw.rs
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
}
fn default_camera(mut cmds: Commands<'_, '_>) {
    cmds.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
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

fn init_map(mut cmds: Commands<'_, '_>) {
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
    let mut surfaces = Surfaces::new();
    surfaces.new_surface(World::new(), map);
    simulation::add_systems(&mut surfaces);
    cmds.insert_resource(surfaces);
    cmds.insert_resource(SelectedSurface(0));
}

fn simulate_surfaces(mut surfaces: ResMut<Surfaces>) {
    surfaces.simulate_step();
}
