---@meta
--- Type definitions for the karna Lua API.
---
--- These are LuaCATS annotations for lua-language-server. Nothing here runs;
--- the file exists so editors can offer completion, signatures and hover docs.
--- Point `workspace.library` at this directory (see `.luarc.json`).
---
--- Each `@class` carries a `@rust` marker naming the Rust type it mirrors.
--- `tests/definitions.rs` parses those markers and fails the build if a bound
--- method is missing here, or documented here but not bound.

---------------------------------------------------------------------------
-- Values
---------------------------------------------------------------------------

---@class karna.Vec2 @rust LuaVec2
---@field x number
---@field y number
---@operator add(karna.Vec2): karna.Vec2
---@operator sub(karna.Vec2): karna.Vec2
---@operator mul(karna.Vec2|number): karna.Vec2
---@operator div(karna.Vec2|number): karna.Vec2
---@operator unm: karna.Vec2
local Vec2 = {}

---@return number
function Vec2:length() end

--- Squared length. Prefer this over `length()` when only comparing distances.
---@return number
function Vec2:length_sq() end

---@return karna.Vec2
function Vec2:normalize() end

--- Perpendicular vector, rotated 90 degrees counter-clockwise.
---@return karna.Vec2
function Vec2:perp() end

---@return number @radians
function Vec2:angle() end

---@param angle number @radians
---@return karna.Vec2
function Vec2:rotate(angle) end

---@param other karna.Vec2
---@return number
function Vec2:dot(other) end

---@param other karna.Vec2
---@return number
function Vec2:distance(other) end

---@param other karna.Vec2
---@param t number @0..1
---@return karna.Vec2
function Vec2:lerp(other, t) end

---@return number x, number y
function Vec2:unpack() end

--- Vectors are userdata, so assignment aliases rather than copies. Use this
--- when you need an independent value.
---@return karna.Vec2
function Vec2:clone() end

--- Mutates in place.
---@param x number
---@param y number
function Vec2:set(x, y) end

---@class karna.Size @rust LuaSize
---@field width number
---@field height number
local Size = {}

---@return number
function Size:area() end

---@return number
function Size:aspect_ratio() end

---@param factor number
---@return karna.Size
function Size:scale(factor) end

---@return number width, number height
function Size:unpack() end

---@return karna.Size
function Size:clone() end

---@class karna.Color @rust LuaColor
---@field r number @0..1
---@field g number @0..1
---@field b number @0..1
---@field a number @0..1
local Color = {}

---@return number r, number g, number b, number a
function Color:unpack() end

---@return karna.Color
function Color:clone() end

---@param a number @0..1
---@return karna.Color
function Color:with_alpha(a) end

---@class karna.Key      @rust LuaKey
---@class karna.Button   @rust LuaButton
---@class karna.Layer    @rust LuaLayer
---@class karna.Image    @rust LuaImage

---------------------------------------------------------------------------
-- Engine services, borrowed for the duration of one callback
---------------------------------------------------------------------------

---@class karna.Window @rust WindowHandle
local Window = {}

---@return string
function Window:title() end

---@return karna.Size
function Window:size() end

---@return number
function Window:width() end

---@return number
function Window:height() end

---@return karna.Vec2
function Window:mouse_position() end

--- Movement since the last frame. Zeroed each frame.
---@return karna.Vec2
function Window:mouse_delta() end

---@return karna.Color
function Window:clear_color() end

---@param title string
function Window:set_title(title) end

---@param width integer
---@param height integer
function Window:set_size(width, height) end

---@param color karna.Color|string @color, or a "#rrggbb" / "#rrggbbaa" string
function Window:set_clear_color(color) end

--- The image may still be loading; the cursor is applied once it decodes.
---@param image karna.Image
---@param hotspot_x? integer @default 0
---@param hotspot_y? integer @default 0
function Window:set_custom_cursor(image, hotspot_x, hotspot_y) end

---@class karna.Time @rust Time
local Time = {}

--- Seconds since the previous frame. Use in `update`.
---@return number
function Time:delta() end

--- Fixed timestep, constant across ticks. Use in `fixed_update`.
---@return number
function Time:fixed_delta() end

---@return number
function Time:fps() end

--- Interpolation factor between the last two fixed ticks, 0..1.
---@return number
function Time:alpha() end

--- Average frame time in seconds.
---@return number
function Time:frame() end

---@param fps integer
function Time:set_target_fps(fps) end

---@param tps integer
function Time:set_target_tps(tps) end

---@class karna.Input @rust Input
local Input = {}

--- True for as long as the key is held.
---@param key karna.Key
---@return boolean
function Input:key_down(key) end

--- True once per edge — once per frame in `update`, once per tick in
--- `fixed_update`.
---@param key karna.Key
---@return boolean
function Input:key_pressed(key) end

---@param key karna.Key
---@return boolean
function Input:key_released(key) end

---@param button karna.Button
---@return boolean
function Input:mouse_down(button) end

---@param button karna.Button
---@return boolean
function Input:mouse_pressed(button) end

---@param button karna.Button
---@return boolean
function Input:mouse_released(button) end

---@return karna.Vec2
function Input:mouse_wheel() end

---@class karna.Assets @rust AssetServer
local Assets = {}

--- Loads asynchronously; the handle is valid immediately but the pixels may
--- not be ready. Paths resolve against the app's asset root.
---@param path string
---@return karna.Image
function Assets:load_image(path) end

---@param image karna.Image
---@return boolean
function Assets:is_image_pending(image) end

---@param image karna.Image
---@return karna.Size
function Assets:image_size(image) end

---@return karna.Image
function Assets:placeholder_image() end

---------------------------------------------------------------------------
-- Drawing
---------------------------------------------------------------------------

---@class karna.Draw @rust DrawRef
local Draw = {}

---@return karna.Size
function Draw:viewport() end

---@return karna.Color
function Draw:color() end

---@return karna.Layer
function Draw:layer() end

---@param color karna.Color|string
function Draw:set_color(color) end

--- Subsequent draws go to this layer. Unknown layers are ignored with a log.
---@param layer karna.Layer
function Draw:set_layer(layer) end

---@param x number
---@param y number
---@param w number
---@param h number
function Draw:rect(x, y, w, h) end

---@param pos karna.Vec2
---@param size karna.Size
function Draw:rect_v(pos, size) end

---@param image karna.Image
---@param x number
---@param y number
function Draw:image(image, x, y) end

--- Camera handle for the scene's layers. No methods yet.
---@class karna.SceneView @rust SceneViewRef

---------------------------------------------------------------------------
-- Contexts and the scene contract
---------------------------------------------------------------------------

---@class karna.Context
---@field window karna.Window
---@field time   karna.Time
---@field input  karna.Input
---@field assets karna.Assets

--- Implement this table and return it from your scene file.
---@class karna.Scene
---@field load          fun(self: self, ctx: karna.Context, scene: karna.SceneView)
---@field update        fun(self: self, ctx: karna.Context, scene: karna.SceneView)
---@field fixed_update? fun(self: self, ctx: karna.Context, scene: karna.SceneView)
---@field draw          fun(self: self, ctx: karna.Context, draw: karna.Draw)

---------------------------------------------------------------------------
-- The module
---------------------------------------------------------------------------

---@class karna.vec2ctor
---@overload fun(x: number, y: number): karna.Vec2
local vec2ctor = {}
---@return karna.Vec2
function vec2ctor.zero() end
---@return karna.Vec2
function vec2ctor.one() end
---@param v number
---@return karna.Vec2
function vec2ctor.splat(v) end
---@param angle number @radians
---@return karna.Vec2
function vec2ctor.from_angle(angle) end

---@class karna.sizector
---@overload fun(width: number, height: number): karna.Size
local sizector = {}
---@return karna.Size
function sizector.zero() end
---@param s number
---@return karna.Size
function sizector.square(s) end

---@class karna.colorctor
---@overload fun(r: number, g: number, b: number, a?: number): karna.Color
---@field RED karna.Color
---@field GREEN karna.Color
---@field BLUE karna.Color
---@field WHITE karna.Color
---@field BLACK karna.Color
---@field YELLOW karna.Color
---@field CYAN karna.Color
---@field MAGENTA karna.Color
---@field GRAY karna.Color
---@field ORANGE karna.Color
---@field PURPLE karna.Color
---@field BROWN karna.Color
---@field PINK karna.Color
local colorctor = {}
---@param r number
---@param g number
---@param b number
---@return karna.Color
function colorctor.rgb(r, g, b) end
---@param r number
---@param g number
---@param b number
---@param a number
---@return karna.Color
function colorctor.rgba(r, g, b, a) end
--- Accepts `"#rgb"`, `"#rrggbb"`, `"#rrggbbaa"` or an integer like `0x89b4fa`.
---@param v string|integer
---@return karna.Color
function colorctor.hex(v) end

---@class karna
---@field vec2   karna.vec2ctor
---@field size   karna.sizector
---@field color  karna.colorctor
---@field key    table<string, karna.Key>    @indexing an unknown name raises
---@field button table<string, karna.Button>
---@field layer  table<string, karna.Layer>  @WORLD, UI, DEBUG
local karna = {}

return karna
