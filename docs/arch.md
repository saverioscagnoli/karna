
<h1 align="center">Karna</h1>

<p align="center">
  <em>Easy to use game framework</em>
</p>

## Prelude

This is a simple mono-paged document that explains and explores the internals of the framework, both for documenting purposes and to keep track of how the engine works; when I come back to working on it without the need to reread (or rewrite :3) every piece of code.

## Crates
### sdl3

Raw SDL3 bindings. unsafe, vendored.

### sdl3-image

Raw SDL3-image bindings. unsafe, vendored.

### Note on submodules

When fresh-cloning the repo, git submodules must be initialized, use:

```bash
git submodule update --init --recursive
```

This will initialize and clone every bit of SDL magic. To update SDL, you can use `scripts/update-sdl.sh`.

### Math
The math library that is used in all the engine cogs, in the code, everything that uses a struct belonging to this library, will be prefixed with m::, so remember to use `use math as m`, I like to do this to identify where structs come from.

### Logging

Personalized logger implementation, uses the `log` crate as the fronted. Also includes a `fatal` macro that is actually just an `error` macro with a system exit.

The `engine` crate will expose its initialization function.

Includes ways to:
- Create a personalized formatter, using the `Formatter` trait.
- Create custom targets, like the included `File` or `Console` (default)
- Apply stylings such as color, bold, italic, etc. for ansi-compatible outputs.

### Utils

As the name implies, various utilities, such as:
- Elegant way to display file sizes,
- Slotmap (might want to move this, as its not really an util at this point. I use it everywhere)
- Slice casting to uint8s
- Rectangle packing (for atlasing)
- Spin sleeper

And other useful things.

### Engine

The core of the framework.Name subject to change.
Uses a custom `build.rs` to transpile shaders to the correct format, using `glslc`, so a potential user must have it as a dependency on their system.

#### Structure

- `lib.rs` - App, main event loop. 
- `scene.rs` - Scene definition, the main block for constructing a game.
- `window` - The window module, includes the sdl window wrapper, window state, game loop, context definition.
- `events` - The event module, includes a event queue implementation, sdl event wrappers, key wrappers, etc. Also includes the actual User events

### QuickJS

JavaScript bindings, on top of `rquickjs`. A scene becomes a `.js` ES module whose default export is an object with `load` / `update` / `fixedUpdate` / `draw` / `unload`, registered with `WindowBuilder::with_js_scene`. Each method is called with that object as `this`, so a script keeps state on `this` the way a Rust scene keeps it in `Self`. Scripts reach the value types with `import { vec2, Key } from "karna"`; `console` is routed into `logging`. A script that throws is logged with its JS stack and then disabled, rather than taking the process down mid-frame.

#### Structure

The split is by lifetime, since that is the only hard problem here.

- `value.rs`, `enums.rs` - Owned values (`Vec2`, `Size`, `Color`) and opaque tokens (keys, buttons, layers, cursors, asset handles). JavaScript may keep these for as long as it likes.
- `slot.rs` - The lending machinery. `rquickjs` classes must be `'static`, but the engine hands each callback short-lived borrows, so the JS-side object holds only a `Slot`; Rust fills it just before calling in and the lease clears it on the way out.
- `context.rs`, `draw.rs` - The engine borrows: `ctx.window` / `time` / `input` / `assets`, and `draw`. Stashing one and using it later raises instead of dangling. The window is read-only inside `draw`, and `assets` is absent there, matching `DrawContext`.
- `module.rs` - The native `karna` module, and `scene.rs` the bridge to the `Scene` trait.
- `types/karna.d.ts` - Type definitions, for editor completion.

### Events

There are 2 main components to the event definitions, first, the sdl event wrappers, that are the boring part They just make it more easy to work with raw sdl event.
And then there are the User events, that are everything that the user requests to do, such as changing the target tick rate, changing target fps, changing a window's title, etc.

##### Event list

- 