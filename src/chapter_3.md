# Part 3: Character movement

In Part 2, we covered spritesheet animation, and had our character change their "move" animation in response to arrow key presses:

![Thomas rotates in eight directions](images/thomas-rotate.gif)

But there's a problem: our character isn't actually moving! (And, if we want the character to stop and stand still, we can't do that, either.) Let's fix this.

## Moving in response to keyboard input

Changing our code to make our character move is actually surprisingly simple, at least compared to the complexity of code we added in Part 2. All we have to do is modify the `player_input` event to:
- gain access to the player entity's `Transform` component,
- gain access to the system's update speed, so we can ensure the frame rate of the user's hardware doesn't affect the movement speed

We can accomplish both by adding a Bevy [`Time` resource](https://bevy-cheatbook.github.io/features/time.html) to our method parameter, and a `&mut Transform` component to our query. Our new method signature is:

```rust
fn player_input (keyboard_input: Res<Input<KeyCode>>,
                 time: Res<Time>,
                 mut query: Query<(&mut SpritesheetAnimator,
                                   &mut TextureAtlasSprite,
                                   &mut Transform),
                                   With<Player>>) {
    let (mut animator,
         mut sprite,
         mut transform) = query.single_mut();

    // body of method goes here //
}
```

To move our character in response to input, we want to setup some variables to set the movement "speed" as a float, the move direction as a unit vector in 2D space (a 2D vector of max length 1), the update delta (in seconds), and a place to store our final compiled movement vector:

```rust
let move_speed: f32 = 32.0; // a constant (for now)
let mut move_dir: (f32, f32) = (0.0, 0.0); // (x_delta, y_delta)
let move_delta: (f32, f32); // will be (move_dir * move_speed) * time_delta
let time_delta: f32 = time.delta_seconds();
```

`move_delta` will equal (move_dir * move_speed) * time_delta.[^note1]

Now we're ready to amend our "facing" update logic. Recall that our current code looks like:

```rust
let mut facing: &str = "";
if left_pressed {
    if up_pressed {
        facing = "move-up-left";
    } else if down_pressed {
        facing = "move-down-left";
    } else {
        facing = "move-left";
    }
} else if right_pressed {
    if up_pressed {
        facing = "move-up-right";
    } else if down_pressed {
        facing = "move-down-right";
    } else {
        facing = "move-right";
    }
} else if up_pressed {
    facing = "move-up";
} else if down_pressed {
    facing = "move-down";
}
```

How should we modify this to update our move variables? Well, we only need to update `move_dir` (the unit vector) to point it in the direction of the movement. You can probably figure this out yourself: **try it!**

Done? You should end up with something like this:

```rust
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
```

(Here, I use `0.71` on diagonal movements, as the "length" of this unit vector, which is calculated with an equation like `sqrt(x^2 + y^2)`, totals roughly `1.0`. If we used `1.0` instead, the unit vector would have a length greater than `1.0`, and the character would appear to run faster when moving diagonally relative to one-directional movement.)

Right after this code, we can compute the `move_delta` with some simple logic, and then apply it directly to the player entity's transform:

```rust
// :: Move character ::
// How far to move the character, in pixel coords:
move_delta = (move_dir.0 * move_speed * time_delta,
              move_dir.1 * move_speed * time_delta);
// Apply move delta to character position:
transform.translation.x += move_delta.0;
transform.translation.y += move_delta.1;
```

Run this code. You should see movement, with one problem: the character animation doesn't change to standing when you aren't touching the arrow keys. There is actually a surprisingly simple fix for this, given the way we've named the animation states ("move-up", "stand-up", and so on). **Can you think of a simple way to fix your existing code to get the character to stand still when a key isn't pressed?**

## Stop moving when no keys are pressed

What I did was amend our existing `if` statement on `facing` with an `else if` for the case when no keys are pressed. Within that block, we can check if the character is currently in a "move-"ing state, and if so, change it to "stand-" state by replacing the prefix "move" with "stand" in the `animator.cur_state` string:

```rust
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
```

(This depends on your states being named the same as Part 2.)

Now run your code, and voila:

![Player character moves in eight directions](images/thomas-movement.gif)

## Conclusion

Compared to the Part 2, this part was surprisingly short and simple. A lot of that has to do with the way we setup animations, and the power of our `SpritesheetAnimator` model. You might have noticed we didn't touch the `Direction` component we setup in Part 1. In the future, we will want to move most of our animation and movement logic out of our keyboard input handler and into its own system for `Character` component entities, so we can easily add more characters and have them follow similar logic. A `Direction` property (or some version of it) will become important. For now, though, bask in the joy of what you've accomplished thus far! :)

-------------

### Code
[Get the finished code for this part.](game_code/chapter_3.rs)

### Footnotes

[^note1]: Another term for (move_dir * move_speed) is velocity.
