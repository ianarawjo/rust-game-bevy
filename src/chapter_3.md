# Part 3: Character character_movement

In Part 2, we ...

## "Facing" and Direction

In Part 1, we added a `Direction` component as an enum with 8 cardinal directions:

```rust
#[derive(Component)]
enum Direction {
    N, NE, E, SE, S, SW, W, NW,
}
```

But this way of formatting direction is actually slightly annoying: it contains two pieces of information, the "horizontal" (East or West) and the "vertical" (North or South) directions. If we want to change one of these directions, we have to worry about the other. It's cleaner to separate these two directions out:

```rust
enum HorizontalDirection {
    East, West, None,
}
enum VerticalDirection {
    North, South, None,
}

#[derive(Component)]
struct Direction {
    hor: HorizontalDirection,
    vert: VerticalDirection
}
```

Now when we want to change the horizontal direction (say, from East to West), we don't need to worry about the vertical direction, and vice-versa.[^note1] We can use this new `Direction` to specify where our character sprite is "facing," but also other things, like the state of a gamepad thumb stick.

But how do we tell what the "default" direction is? Above, we haven't specified the default direction as `hor`=`None`, `vert`=`None`. To set this, we can use the [Default](https://doc.rust-lang.org/stable/std/default/trait.Default.html) trait in Rust:

```rust
#[derive(Default)]
enum HorizontalDirection {
    East,
    West,
    #[default]
    None,
}

#[derive(Default)]
enum VerticalDirection {
    North,
    South,
    #[default]
    None,
}

#[derive(Component, Default)]
struct Direction {
    hor: HorizontalDirection,
    vert: VerticalDirection
}
```

Now `None` is specified as the default for both enums. The struct `Direction`, appended with the `Default` trait, will automatically set `hor` and `vert` properties to `None`. (Notice that to add "Default" to our `#[derive(...)]` directive, we tacked it after Component, using a comma.)

Our code might be getting cluttered already, but we'll worry about refactoring it later.

## Movement function

With our better `Direction` component in tow, we can amend our `move_character` function to:
- read what arrow keys are being pressed, and
- update the player character's `Direction` accordingly

Recall that our current movement function is something like:

```rust
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
```

I didn't say much about `just_pressed` and `just_released` before, but they're important, arguably more so than basic `pressed`.[^note2] In the end, we don't want to directly move our character sprite in this function ---other things, like character animation, might need to trigger here, and the movement could be dependent on the current game state, collisions, etc. In fact, we'd do better to rename the function and not be able to access `Transform` at all, only `Direction`:

```rust
fn player_input (keyboard_input: Res<Input<KeyCode>>,
                 mut query: Query<&mut Direction, With<Player>>) {
    ...
}
```

...

Why do this? Well, there might be other ways we could set the player's `Direction` --such as cutscenes, gamepad input, or long chats with career counselors. How the player character displays in response to `Direction` should be in a separate System, so we only have to define it once:

```rust
fn character_movement (mut query: Query<(&mut Transform, &mut Direction), With<Character>>) {
    // Read the character's Direction, and change its state in response
    ...
}
```

Notice anything odd about this statement? I didn't use `Player` here ---I used a new Component, `Character`. Why?

A `Player` is special kind of `Character` in our game, one that we play as and control. But there may be many other characters (NPCs, short for non-player-character), which behave similarly to the player ---they have a direction they are facing in, can move, animate, etc. To save time, the behavior of all entities with a `Character` component should be described in a single function ("System").

Let's first define a `Character` component:

```rust
#[derive(Component)]
struct Character {
    facing: Direction,
    moving: Direction,
    speed: f32,
    name: String,
}
```

Once again, we'd like to set defaults for `Character`s, like what direction it is facing in when it is spawned. To do this, we can `impl Default` (add after the definition for `Character`):

```rust
impl Default for Character {
    // By default, Characters face south (down) and don't move:
    fn default() -> Self { Character {
        facing: Direction {
            hor: HorizontalDirection::None, vert: VerticalDirection::South },
        moving: Direction {
            hor: HorizontalDirection::None, vert: VerticalDirection::None },
        speed: 0.0,
        name: "Unnamed".to_string(),
    }}
}
```

Now when we spawn our player character, we can give him a name:

```rust
commands.spawn((
    Player,
    Character {
        name: "Thomas".to_string(),
        ..default()
    },
    SpriteBundle {
        texture: asset_server.load("images/thomas_stand.png"),
        ..default()  // Set remaining arguments to their default values
    },
));
```

and he will spawn facing down by default.

Notice that I've removed the `Direction` property ---this is now contained as a property of the `Character` component. We must change our `player_input` function accordingly:

```rust
fn player_input (keyboard_input: Res<Input<KeyCode>>,
                 mut query: Query<&mut Character, With<Player>>) {
    let mut player_character = query.single_mut();
    ...
}
```

## Conclusion

...

### Footnotes

[^note1]: What is the direction when both properties are "None"? This depends on the use case, but we include this possibility here to, for instance, encode the base state of the thumb stick of a gamepad. Our character sprite must always face in a direction, but we can read "None" as facing down/south.

[^note2]: In fact, without Bevy, we might have to code the logic for the  `just_pressed` and `just_released` events ourselves ---which can be time-consuming and (sometimes) tricky. We're lucky we have Bevy to handle these common events for us.
