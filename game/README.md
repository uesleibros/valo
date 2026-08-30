# Games

Programs written in Valo that exercise the language against something real.

Unlike [`examples/`](../examples/README.md), which demonstrates one feature at a
time, these are whole programs. They exist to find the gaps that only show up
when features have to work together, and they have. Every entry in "What this
found" below was a real defect, fixed in the commit that reports it.

## Breakout

[`breakout/`](breakout/) is a playable Breakout drawn with SDL3 through Valo's
native FFI.

```sh
valo run game/breakout/main.valo
```

The paddle plays by itself until you press a key, so simply running it
demonstrates the game. After that: **←/→** or **A/D** to move, **R** to serve a
new ball, **Esc** to quit. The window title carries the score. The program stops
on its own after a fixed number of frames, so a run always terminates.

### Getting SDL3

The declarations name the library as `SDL3`, which the loader resolves against
the current directory, the executable's directory, and `PATH`. Put the library
in one of those:

| Platform | File | Where to get it |
|---|---|---|
| Windows | `SDL3.dll` | [SDL releases](https://github.com/libsdl-org/SDL/releases) |
| Linux | `libSDL3.so.0` | `libsdl3-0` from your package manager |
| macOS | `libSDL3.dylib` | `brew install sdl3` |

Without it the program says so and exits rather than failing at the first call.

### How it is put together

| File | Holds |
|---|---|
| [`Sdl.valo`](breakout/Sdl.valo) | Every `Declare`, plus the two structs that mirror the native ABI |
| [`Game.valo`](breakout/Game.valo) | Geometry, entities, and the rules that move them |
| [`main.valo`](breakout/main.valo) | The window, the loop, input, and drawing |

Nothing outside `Sdl.valo` mentions the native library, so the model can be read
and changed without knowing anything about SDL.

### What the language is doing here

- **Native interop**: `Declare` for every SDL entry point, a `Type` passed
  `ByRef` for `SDL_FRect`, and a 128-byte `Type` that the native side fills in
  for `SDL_Event`
- **Classes and inheritance**: `Entity` with `Paddle`, `Ball`, and `Brick`
  overriding `Advance`
- **An interface**: the loop draws through `IDrawable` without knowing what
  each thing is
- **Events**: a brick announces its own destruction; scoring lives with the
  event rather than inside the brick
- **Operator overloading**: `Vector2` addition and scaling, so the physics
  reads as arithmetic
- **Properties**: `Left`, `Right`, `Top`, `Bottom`, and `CenterX` derived from
  a position and a size
- **String interpolation**: the HUD, including a format specifier
- **Shift operators**: colours packed into one value and unpacked to draw
- **`Select Case`, `Enum`, `Continue For`, compound assignment, `Optional`
  parameters, object initializers, and `Collection` with `For Each`**

### What this found

Writing it surfaced five defects that the test suite and the single-feature
examples had not. All five are fixed:

1. Reading a property off another instance ran the getter against the wrong
   object, so every comparison between two objects through their properties was
   wrong.
2. A member could not be reached through a parenthesised expression:
   `(a + b).Describe()` did not parse.
3. `Imports` did not bring a type into scope: `As Thing` was rejected, and the
   qualified `As Shapes.Thing` was accepted without being checked at all.
4. A class could not inherit from one declared in the same imported module.
5. Assignment through an unqualified member did not resolve: inside a class,
   `Member.X = value` needed `Me.` spelled out, though *reading* it did not.
   The `Me.` this file used to carry are gone, and the game plays the same.

It also showed where the interpreter was slow. A method call cost five times
what reading a field cost, because resolution copied the whole class, every
method body it declares included, to reach one member.

### Known rough edges

- A frame costs roughly 16ms of work, which is what the loop asks for, so the
  game is only just keeping up. It started at 86ms. The interpreter stopped
  copying a class and a method body on every call, the collision scan stopped
  reaching for values it could hold, and then a round of work on how names are
  resolved took it from 24ms to 16ms. SDL itself accounts for 6ms of that; the
  rest is the tree-walking interpreter, which is what the
  [bytecode VM](../docs/architecture/roadmap.md) on the roadmap is for.

  Measured by timing a whole 900-frame run and subtracting the 16ms the loop
  sleeps each frame, which is the only way to separate the work from the wait.

  Measuring this needs care: a single run on a developer machine varies by
  tens of percent, enough to show an improvement where there is none. Every
  figure here is the best of several runs.
- `Event` and `Lib` are keywords, so neither can name a variable or parameter.
  Escaped identifiers work: `[event]`.
