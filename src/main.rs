use bevy::prelude::*;
use hexmap::HexMap;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
// use iyes_loopless::prelude::*;

pub mod draw;
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

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(default_camera)
        .add_startup_system(init_map);
    draw::init_app(&mut app);
    app.run();
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
}
