-- arena.lua -- a hypothetical karna game, written against hypothetical bindings.
--
-- The shape mirrors the Rust `Scene` trait 1:1: a table with `load`, `update`,
-- `fixed_update` and `draw`, returned from the module. The host registers it
-- with `WindowBuilder::with_scene`, and `self` is the scene struct.

local karna = require("karna")

local vec2  = karna.vec2
local color = karna.color
local key   = karna.key
local layer = karna.layer

local PLAYER_SIZE   = 34.0
local COIN_SIZE     = 16.0
local HUNTER_SIZE   = 26.0
local ACCEL         = 1600.0
local DAMPING       = 0.86
local HUNTER_SPEED  = 130.0
local SPAWN_EVERY   = 1.1

local PALETTE = {
    bg      = color.hex("#11111b"),
    player  = color.hex("#89b4fa"),
    coin    = color.hex("#f9e2af"),
    hunter  = color.hex("#f38ba8"),
    ui      = color.hex("#cdd6f4"),
    debug   = color.rgba(0.0, 1.0, 0.4, 0.6),
}

-- axis-aligned overlap test; every entity is a { pos, size } pair
local function overlaps(a, b)
    return a.pos.x < b.pos.x + b.size.width
       and b.pos.x < a.pos.x + a.size.width
       and a.pos.y < b.pos.y + b.size.height
       and b.pos.y < a.pos.y + a.size.height
end

local function center(e)
    return e.pos + vec2(e.size.width, e.size.height) * 0.5
end

-- four thin rects, since the renderer only knows how to fill quads
local function outline(draw, e, thickness)
    local x, y = e.pos.x, e.pos.y
    local w, h = e.size.width, e.size.height

    draw:rect(x, y, w, thickness)
    draw:rect(x, y + h - thickness, w, thickness)
    draw:rect(x, y, thickness, h)
    draw:rect(x + w - thickness, y, thickness, h)
end

local Arena = {}

function Arena:load(ctx, scene)
    ctx.time:set_target_fps(144)
    ctx.time:set_target_tps(60)
    ctx.window:set_clear_color(PALETTE.bg)

    local view = ctx.window:size()

    self.player = {
        pos  = vec2(view.width * 0.5, view.height * 0.5),
        vel  = vec2.zero(),
        size = karna.size(PLAYER_SIZE, PLAYER_SIZE),
    }

    self.coins       = {}
    self.hunters     = {}
    self.spawn_timer = 0.0
    self.score       = 0
    self.lives       = 3
    self.over        = false
    self.show_debug  = false
end

function Arena:spawn(view)
    local margin = 40.0

    local slot = {
        pos  = vec2(
            margin + math.random() * (view.width  - margin * 2.0),
            margin + math.random() * (view.height - margin * 2.0)
        ),
        size = karna.size(COIN_SIZE, COIN_SIZE),
    }


    -- every fourth pickup is a hunter instead
    if #self.coins > 0 and math.random() < 0.25 then
        slot.size = karna.size(HUNTER_SIZE, HUNTER_SIZE)
        table.insert(self.hunters, slot)
    else
        table.insert(self.coins, slot)
    end
end

-- Input and anything frame-rate dependent lives here. `key_pressed` is
-- edge-scoped by the engine, so it reports the same edge exactly once per
-- frame here and once per tick in `fixed_update`.
function Arena:update(ctx, scene)
    if ctx.input:key_pressed(key.F1) then
        self.show_debug = not self.show_debug
    end

    if self.over then
        if ctx.input:key_pressed(key.Space) then
            self:load(ctx, scene)
        end
        return
    end

    local vel = self.player.vel
    local dt  = ctx.time:delta()

    if ctx.input:key_down(key.W) then vel.y = vel.y - ACCEL * dt end
    if ctx.input:key_down(key.S) then vel.y = vel.y + ACCEL * dt end
    if ctx.input:key_down(key.A) then vel.x = vel.x - ACCEL * dt end
    if ctx.input:key_down(key.D) then vel.x = vel.x + ACCEL * dt end

    -- no text renderer yet, so the window title is the HUD
    ctx.window:set_title(string.format(
        "arena -- score %d -- lives %d -- %.0f fps",
        self.score, self.lives, ctx.time:fps()
    ))
end

-- Simulation runs on the fixed clock so collisions stay deterministic
-- regardless of how fast the window is presenting.
function Arena:fixed_update(ctx, scene)
    if self.over then return end

    local dt   = ctx.time:fixed_delta()
    local view = ctx.window:size()
    local p    = self.player

    p.vel = p.vel * DAMPING
    if p.vel:length() < 1.0 then
        p.vel = vec2.zero()
    end

    p.pos = p.pos + p.vel * dt
    p.pos = vec2(
        math.min(math.max(p.pos.x, 0.0), view.width  - p.size.width),
        math.min(math.max(p.pos.y, 0.0), view.height - p.size.height)
    )

    self.spawn_timer = self.spawn_timer + dt
    if self.spawn_timer >= SPAWN_EVERY then
        self.spawn_timer = self.spawn_timer - SPAWN_EVERY
        self:spawn(view)
    end

    for i = #self.coins, 1, -1 do
        if overlaps(p, self.coins[i]) then
            table.remove(self.coins, i)
            self.score = self.score + 1
        end
    end

    local target = center(p)

    for i = #self.hunters, 1, -1 do
        local h  = self.hunters[i]
        local to = target - center(h)

        if to:length() > 0.001 then
            h.pos = h.pos + to:normalize() * (HUNTER_SPEED * dt)
        end

        if overlaps(p, h) then
            table.remove(self.hunters, i)
            self.lives = self.lives - 1
            self.over  = self.lives <= 0
        end
    end
end

function Arena:draw(ctx, draw)
    local view = draw:viewport()

    -- world layer: camera-transformed gameplay geometry
    draw:set_layer(layer.WORLD)

    draw:set_color(PALETTE.coin)
    for _, c in ipairs(self.coins) do
        draw:rect_v(c.pos, c.size)
    end

    draw:set_color(PALETTE.hunter)
    for _, h in ipairs(self.hunters) do
        draw:rect_v(h.pos, h.size)
    end

    draw:set_color(self.over and PALETTE.hunter or PALETTE.player)
    draw:rect_v(self.player.pos, self.player.size)

    -- ui layer: screen space, never touched by the world camera
    draw:set_layer(layer.UI)
    draw:set_color(PALETTE.ui)

    for i = 1, self.lives do
        draw:rect(12.0 + (i - 1) * 20.0, 12.0, 14.0, 14.0)
    end

    draw:rect(12.0, 36.0, math.min(self.score * 8.0, view.width - 24.0), 4.0)

    -- debug layer: hitboxes and the velocity vector, toggled with F1
    if self.show_debug then
        draw:set_layer(layer.DEBUG)
        draw:set_color(PALETTE.debug)

        outline(draw, self.player, 1.0)
        for _, h in ipairs(self.hunters) do outline(draw, h, 1.0) end
        for _, c in ipairs(self.coins)   do outline(draw, c, 1.0) end

        local from = center(self.player)
        local tip  = from + self.player.vel * 0.15

        draw:rect(math.min(from.x, tip.x), from.y - 1.0, math.abs(tip.x - from.x), 2.0)
        draw:rect(from.x - 1.0, math.min(from.y, tip.y), 2.0, math.abs(tip.y - from.y))
    end
end

return Arena
