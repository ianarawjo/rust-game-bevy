# Part 4: More Characters

In Part 3, we got our player character moving:

![Player character moves in eight directions](images/thomas-movement.gif)

So far so good, right? Our natural inclination might be to proceeding to filling in the gray background ---to load tilemaps and do collision with walls. However, are we really "done" with characters? What if we want to add a second, non-player character, and have it walk around? Do our existing abstractions make this possible?

Nope. Looking closer, we've done a pretty poor job: all of our movement code is in our keyboard event, `player_input`. In other words, all walking movement and animation is triggered solely by key presses, and only for our player character. We can’t use this code for a [non-player character (NPC)](https://en.wikipedia.org/wiki/Non-player_character), or they’d only move in response to key presses, too. If we want them to walk, say, along a preset path, we need two things:
- a general system that works for any “character” (player or non-player) that controls their walk and movement animations,
- some way to specify a path or direction a character should walk in

Together, this means we need some way to control character *state*, abstracted away from any particular means of *triggering* state changes. Our current model is too low-level: whenever we want to move the character, we have to independently change the animations and velocity. What we’d prefer to do is tell a character entity "move up at speed X," and have the movement speed and animation frame rate respond together. Then we can build even higher-level abstractions that puppeteer characters, say on set paths (like in cutscenes).

## Developing a `Character` component

Let’s add a component for characters. A `Character` will have a state, and be initialized with a set of possible states it could be in. Think “walk-left,” “run-up,” “sleep,” “fall-over,” and so on. Each state has a corresponding `SpritesheetAnimator` state (how the character should be rendered for that state), fps (how fast the animation should play), and movement vector (velocity).[^note1] With a `Character`, we can simply change the character’s overall state, and then write a system that updates the character’s `SpritesheetAnimator` and `Transform` components accordingly. To demonstrate the advantage of this setup, we will add a second character, and have them walk along a preset path.[^note2]

To take full advantage of `Character` components, we’ll first offload our velocity-based update of an entity’s `Transform` to a general component, `Velocity`. This component will simply store velocity and update an entity’s `Transform` component every cycle. In other words, instead of these lines in our `player_input` method:

```rust
move_delta = (move_dir.0 * move_speed * time_delta,
              move_dir.1 * move_speed * time_delta);
// Apply move delta to character position:
transform.translation.x += move_delta.0;
transform.translation.y += move_delta.1;
```

we will move them to a separate System for a new component, `Velocity`.

### Defining `Velocity` and a `delta_seconds!` macro

We define the component,

```rust
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}
```

and write a system that operates on all entities with a `Velocity` and a `Transform`:

```rust
fn apply_velocity(
    time: Res<Time>,
	mut query: Query<(&Velocity, &mut Transform)>,
) {
    let time_delta: f32 = time.delta_seconds();
	for (velocity, mut transform) in query.iter_mut() {
		let translation = &mut transform.translation;
		translation.x += velocity.x * time_delta;
		translation.y += velocity.y * time_delta;
    }
}
```

Again, we used a `Time` resource so we can use velocity values independent of the game refresh rate.[^note3]

We might think we're done here, but consider what would happen if `time_delta` was "large", say a full second. Consider if we had implemented collision between objects. Objects moving fast might go straight through each other, or clip through walls, if the system hardware slowed down for a second! That could break our game. To prevent this, we can clamp the magnitude of `time_delta` to something reasonable, like 10 fps:

```rust
let time_delta: f32 = time.delta_seconds().max(0).min(1./10.);
```

Now no matter how much the system slows down, the game itself will only see update events at most 0.05 "seconds" long, and no longer.

However, this is only true for the `Velocity` system... whenever we want to get the time delta, we'll need to clamp the values. So it's better to create general macro for this operation:

```rust
macro_rules! delta_seconds {
    ($t:ident) => {$t.delta_seconds().max(0.).min(1./10.)}
}
```

Then:

```rust
let time_delta: f32 = delta_seconds!(time);
```

and we can use `delta_seconds!(time)` whenever we need to get the elapsed time between updates, and forget about the clamping to 10 fps.[^note4]

### Defining `Character`

Now we're ready to define a component for characters. A `Character` is like our `SpritesheetAnimator`, in that it has a current state, and a set of possible states it could be in:

```rust
#[derive(Component)]
struct Character {
    states: HashMap<String, ???>,
    cur_state: String,
    prev_state: String,
}
```

But what is the type of the `HashMap`'s values? Well, we said above that a character's state involves three things: an animation state, a frame rate for the animation to run at, and a movement vector (a velocity). We can separate this out into a struct, `CharacterState`:

```rust
struct CharacterState {
    animation: String,
    fps: f32,
    movement: Velocity,
}
```

Thus, a Character assumes an entity it is attached to has a Velocity and an Animator, and its system updates only those components. In turn, those components have their own independent systems for updating the entity’s Transform and Texture. (Notice that we re-use our `Velocity` struct here ---but this time, just as a struct, not a component in Bevy.)

Going back to our `Character` component, now we have:

```rust
#[derive(Component)]
struct Character {
    states: HashMap<String, CharacterState>,
    cur_state: String,
    prev_state: String,
}
```

Now we will write a system which (1) detects changes to a `Character`'s state `cur_state`, and (2) updates an entity's `SpritesheetAnimator` and `Velocity` components ---not just for our player, but *for any character* in the game:

```rust
fn update_character_state(
	mut query: Query<(&mut Character, &mut SpritesheetAnimator, &mut TextureAtlasSprite, &mut Velocity)>,
) {
	for (mut character, mut animator, mut sprite, mut velocity) in query.iter_mut() {
		// Check if cur_state differs from prev_state
        if character.cur_state == character.prev_state { continue; }

        // Update the state by changing animator and transforms:
        new_state: CharacterState = states.get(&character.cur_state);
        if let Some(char_state) = new_state {
            animator.set_state(char_state.animation, sprite, char_state.fps);
            velocity.x = char_state.velocity.x;
            velocity.y = char_state.velocity.y;
        }

        character.prev_state = character.cur_state;
    }
}
```

Now we won't touch the `SpritesheetAnimator` directly ---instead, if we want to change the `Character` animation, we must declare a `CharacterState` and change it indirectly through this system. While this approach may seem overly abstract, it is very powerful: it allows us to create a second, non-player character, or indeed however many characters we want.

## Spawning a friend

Let's spawn a friend for Thomas to interact with. Our second character will be Missy:

![Missy's spritesheet for her walk animation](images/missy_walk.png)

(Notice that Missy's spritesheet is in the same format as Thomas's. We'll use this to easily define her walk animations analogously to Thomas's.)

How do we spawn Missy in code? Of course we update our `setup` system, but do we have to do anything special outside of what we've done for Thomas?

**No!** We don't! That's amazing --effectively, both Thomas and Missy will be seen as `Character`s. Only the `Player` flag on Thomas' character entity will distinguish him from Missy. (In fact, if we wanted to change characters mid-game, all we'll have to do is remove the `Player` component from the Thomas entity and give it to the Missy entity! By the end of this lesson, we'll have done exactly this ---to change who we're playing as by pressed the Space bar.)



... WIP ...

## Conclusion

...

### Footnotes

[^note1]: The reason we develop a component called `Character` and not, say, a broader component like `Puppeteer` is because other game entities might not behave like human characters. For instance, bats could have a generic fly animation and travel towards the player. For characters, movement is more constrained and attached to specific directions and animations.

[^note2]: In Part 5, we will then add a way to interact with this NPC, and change their state to "stand" when the player talks with them. That’ll involve new components, like `Collider` and `InteractBox`.

[^note3]: Later on, this gets more complicated with Collision. There are different ways to handle collision events; most likely, we will do collision detection at the end of an update frame, and correct an entity’s transform so that it moves just out of range of the colliding entity. We might also modify the entity’s Velocity.

[^note4]: Some readers might find this part an unnecessary optimization. But, as a working game developer, I often ran into this time-delta issue during unexpected system slow-downs. Yes, the game might appear to run slower with this method, but we don't expect it to be clipping down to 10 fps anytime soon. And, it's better to have a game slow down than to have it break entirely.
