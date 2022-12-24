use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            // Set the image display format to nearest-neighbor for crisp pixels
            .set(ImagePlugin::default_nearest())

            // Set the window name and size
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "My Game Project".to_string(),
                    width: 800.,
                    height: 600.,
                    ..default()
                },
                ..default()
            }))
        .add_startup_system(setup)
        .add_system(move_character)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        Camera2dBundle {
            transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.0)),
            ..default()
        }
    );
    commands.spawn((
        Player,
        Direction::S,
        SpriteBundle {
            texture: asset_server.load("images/thomas_stand.png"),
            ..default()  // Set remaining arguments to their default values
        },
    ));
}

fn move_character (keyboard_input: Res<Input<KeyCode>>,
                   mut query: Query<&mut Transform, With<Player>>) {
    let mut player_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        info!("'Left arrow' currently pressed");
        player_transform.translation.x -= 1.0
    }

    if keyboard_input.just_pressed(KeyCode::Left) {
        info!("'Left arrow' just pressed");
    }

    if keyboard_input.just_released(KeyCode::Left) {
        info!("'Left arrow' just released");
    }
}
