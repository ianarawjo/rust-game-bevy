# Part 2: Sprite Movement and Animation

In Part 1, we created our player character and basic leftward movement:

![A 800x600 window with our character displayed, non-blurry](images/fixed-window.png)

In this part, we'll delve into the nitty-gritty of 8-directional movement and spritesheet animation. We'll substantially revise our keyboard event function, `move_character`, and create a new System for handling Character motion.

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

Now when we want to change the horizontal direction (say, from East to West), we don't need to worry about the vertical direction, and vice-versa.[^note1]

We can use this new `Direction` to specify where our character is "facing."

[^note1]: What is the direction when both properties are "None"? This depends on the use case, but we include this possibility here to, for instance, encode the base state of the thumb stick of a gamepad. Our character sprite must always face in a direction, but we can read "None" as facing down/south. 
