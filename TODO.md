## Done

- [x] Sprite sheets and animated sprites
- [x] RNG
- [x] Draw state stack: push / pop, translate / rotate / scale
- [x] Offscreen rendering (canvases)
- [x] `Aabb` (2D and 3D) with intersection and penetration
- [x] Math test coverage


## Collision

- [x] Use `Aabb3` in `Frustum::intersects_aabb`
- [x] Replace the private `Bounds` in `renderer.rs` with `Aabb3`
- [ ] Move-and-collide helper (resolve x and y separately)
- [ ] Broadphase: uniform grid for 2D
- [ ] Raycast against `Aabb`
- [ ] Swept AABB for fast movers
- [ ] 3D narrowphase shapes: sphere, capsule

## Scripting

- [ ] JS: push / pop, translate / rotate / scale
- [ ] Lua: push / pop, translate / rotate / scale
- [ ] JS and Lua: canvases
- [ ] JS and Lua: `Aabb`

## Gameplay

- [ ] Tweening and easing
- [ ] Timers and cooldowns
- [ ] Free images, fonts and audio (`remove_*`), shrink atlas pages
- [ ] Tilemaps, with a Tiled loader on the JSON parser
- [ ] Particles
- [ ] UI widgets
- [ ] Save data through an SDL pref path
- [ ] Hot reload of assets and scripts

## Rendering

- [ ] Per-canvas sampler (nearest / linear)
- [ ] Screen-sized canvas that follows window resizes
- [ ] Custom shaders and post-processing
- [ ] Point and spot lights
- [ ] Shadows
- [ ] glTF or OBJ model import
- [ ] MSAA

## Infrastructure

- [ ] Tests for `Affine2` in `draw.rs`
- [ ] Tests for `LayerData::record` command merging
- [ ] More engine tests overall
