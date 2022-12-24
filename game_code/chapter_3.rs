use bevy::{prelude::*, utils::HashMap};

#[derive(Component)]
struct Player;

#[derive(Component)]
enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}

// A timer for animations
#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// How the animation should continue after it reaches the last frame
enum AnimationStyle {
    Once,    // Play once and end at last frame
    Looping, // Loop from frame 1 to n, then from 1 to n, ad infinitum
}

// A SpritesheetAnimation is a series of indexes for a TextureAtlas,
// referencing the frames to use for a single animation. The "fps" is
// how fast to display the animation.
// NOTE: You will be able to use negative frame id's to represent x-flipped textures
const DEFAULT_ANIMATION_FPS: f32 = 5.0;
struct SpritesheetAnimation {
    frames: Vec<i8>, // the frames of the animation, as the TextureAtlas' indices + 1
    fps: f32, // how quickly to go to the next frame, in frames per second
    looping: AnimationStyle // whether and how to loop the animation
}
impl SpritesheetAnimation {
    fn from_frames(frames: Vec<i8>) -> Self {
        Self {
            frames,
            fps: DEFAULT_ANIMATION_FPS,
            looping: AnimationStyle::Looping
        }
    }
}

// A SpriteAnimator is a map from "states" (strings)
// to individual animations.
#[derive(Component)]
struct SpritesheetAnimator {
    states: HashMap<String, SpritesheetAnimation>,
    timer: AnimationTimer,
    cur_state: String,
    cur_frame_idx: usize,
}
impl SpritesheetAnimator {
    fn new(states: HashMap<String, SpritesheetAnimation>,
           start_state: String) -> Self {
        match states.get(&start_state) {
            Some(anim) => {
                if anim.fps as f32 == 0.0 {
                    panic!("Frames per second must be positive, nonzero value")
                }
                Self {
                    timer: AnimationTimer(Timer::from_seconds(1.0 / anim.fps, TimerMode::Repeating)),
                    states: states,
                    cur_state: start_state,
                    cur_frame_idx: 0,
                }
            },
            None => {
                panic!("Start state {} not found", start_state)
            },
        }
    }
    fn set_state(&mut self, state_name: String, sprite: &mut TextureAtlasSprite) -> bool {
        match self.states.get(&state_name) {
            Some(state) => {
                if state.fps as f32 == 0.0 {
                    panic!("Frames per second must be positive, nonzero value")
                }
                self.cur_state = state_name;
                self.cur_frame_idx = 0;
                self.timer = AnimationTimer(Timer::from_seconds(1.0 / state.fps,
                                            TimerMode::Repeating));
                // Set the sprite frame and x-flip value
                if let Some(texture_idx) = state.frames.get(0) {
                    sprite.index = ((*texture_idx).abs()-1) as usize;
                    sprite.flip_x = (*texture_idx) < 0; // flip texture if negative
                }
                true
            },
            None => false,
        }
    }
}

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
        .add_system(animate_sprites)
        .add_system(player_input)
        .run();
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>) {

    let texture_handle = asset_server.load("images/thomas_walk.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle,
                                Vec2::new(16.0, 32.0),
                                15, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let player_animations = SpritesheetAnimator::new(
        HashMap::from([
            ("stand-down".to_string(), SpritesheetAnimation::from_frames(vec![1])),
            ("stand-down-left".to_string(), SpritesheetAnimation::from_frames(vec![4])),
            ("stand-left".to_string(), SpritesheetAnimation::from_frames(vec![7])),
            ("stand-up-left".to_string(), SpritesheetAnimation::from_frames(vec![10])),
            ("stand-up".to_string(), SpritesheetAnimation::from_frames(vec![13])),
            ("stand-up-right".to_string(), SpritesheetAnimation::from_frames(vec![-10])),
            ("stand-right".to_string(), SpritesheetAnimation::from_frames(vec![-7])),
            ("stand-down-right".to_string(), SpritesheetAnimation::from_frames(vec![-4])),
            ("move-down".to_string(), SpritesheetAnimation::from_frames(vec![1, 2, 1, 3])),
            ("move-down-left".to_string(), SpritesheetAnimation::from_frames(vec![4, 5, 4, 6])),
            ("move-left".to_string(), SpritesheetAnimation::from_frames(vec![7, 8, 7, 9])),
            ("move-up-left".to_string(), SpritesheetAnimation::from_frames(vec![10, 11, 10, 12])),
            ("move-up".to_string(), SpritesheetAnimation::from_frames(vec![13, 14, 13, 15])),
            ("move-up-right".to_string(), SpritesheetAnimation::from_frames(vec![-10, -11, -10, -12])),
            ("move-right".to_string(), SpritesheetAnimation::from_frames(vec![-7, -8, -7, -9])),
            ("move-down-right".to_string(), SpritesheetAnimation::from_frames(vec![-4, -5, -4, -6])),
        ]),
        "move-down".to_string()
    );

    commands.spawn(
        Camera2dBundle {
            transform: Transform::from_scale(Vec3::new(0.5, 0.5, 1.0)),
            ..default()
        }
    );
    commands.spawn((
        Player,
        player_animations,
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            ..default()  // Set remaining arguments to their default values
        },
    ));
}

fn animate_sprites(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut SpritesheetAnimator,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut animator, mut sprite, texture_atlas_handle) in &mut query {
        let timer = &mut animator.timer;
        timer.tick(time.delta());
        if timer.just_finished() {
            // Get reference to spritesheet texture
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();

            // Get reference to current animation and advance to next frame
            let mut next_frame_idx: usize = animator.cur_frame_idx;
            if let Some(anim) = animator.states.get(&animator.cur_state) {

                // Advance to the index of the next frame
                let num_frames = anim.frames.len();
                if (animator.cur_frame_idx + 1) >= num_frames {
                    if matches!(anim.looping, AnimationStyle::Looping) {
                        next_frame_idx = 0;
                    }
                } else {
                    next_frame_idx = animator.cur_frame_idx + 1;
                }

                // Set the sprite frame and x-flip value
                let next_frame_texture = anim.frames.get(next_frame_idx);
                if let Some(texture_idx) = next_frame_texture {
                    sprite.index = (((*texture_idx).abs()-1) as usize) % texture_atlas.textures.len();
                    sprite.flip_x = (*texture_idx) < 0; // flip texture if negative
                }
            }

            animator.cur_frame_idx = next_frame_idx;
        }
    }
}

fn player_input (keyboard_input: Res<Input<KeyCode>>,
                 time: Res<Time>,
                 mut query: Query<(&mut SpritesheetAnimator,
                                   &mut TextureAtlasSprite,
                                   &mut Transform),
                                   With<Player>>) {

    let (mut animator,
        mut sprite,
        mut transform) = query.single_mut();

    let move_speed: f32 = 32.0;
    let mut move_dir: (f32, f32) = (0.0, 0.0); // (x_delta, y_delta)
    let move_delta: (f32, f32);
    let time_delta: f32 = time.delta_seconds();

    let (left_pressed, up_pressed, right_pressed, down_pressed) =
        (keyboard_input.pressed(KeyCode::Left), keyboard_input.pressed(KeyCode::Up),
        keyboard_input.pressed(KeyCode::Right), keyboard_input.pressed(KeyCode::Down));

    let mut facing: &str = "";
    if left_pressed {
        if up_pressed {
            facing = "move-up-left";
            move_dir = (-0.71, 0.71);
        } else if down_pressed {
            facing = "move-down-left";
            move_dir = (-0.71, -0.71);
        } else {
            facing = "move-left";
            move_dir = (-1.0, 0.0);
        }
    } else if right_pressed {
        if up_pressed {
            facing = "move-up-right";
            move_dir = (0.71, 0.71);
        } else if down_pressed {
            facing = "move-down-right";
            move_dir = (0.71, -0.71);
        } else {
            facing = "move-right";
            move_dir = (1.0, 0.0);
        }
    } else if up_pressed {
        facing = "move-up";
        move_dir = (0.0, 1.0);
    } else if down_pressed {
        facing = "move-down";
        move_dir = (0.0, -1.0);
    }

    // :: Move character ::
    // How far to move the character, in pixel coords:
    move_delta = (move_dir.0 * move_speed * time_delta,
                  move_dir.1 * move_speed * time_delta);
    // Apply move delta to character position:
    transform.translation.x += move_delta.0;
    transform.translation.y += move_delta.1;

    // :: Change character animation ::
    // If a key is pressed and the state would change, update the anim:
    if facing.len() > 0 && animator.cur_state != facing.to_string() {
        animator.set_state(facing.to_string(), &mut sprite);
    // If a key isn't pressed...
    } else if facing.len() == 0 {
        // check if the character animation is in a 'move'ing state,
         if animator.cur_state.starts_with("move") {
            // and if it is, set animator to the corresponding 'stand' state:
            let stand_state = "stand".to_string() + &animator.cur_state[4..].to_string();
            animator.set_state(stand_state, &mut sprite);
         }
    }
}
