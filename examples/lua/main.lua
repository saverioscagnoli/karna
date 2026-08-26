local karna = require("karna")

local vec2 = karna.vec2
local key = karna.key

---@type karna.Scene
local LuaDemoScene = {};
local position = vec2(10, 10);

function LuaDemoScene:load(ctx, scene)
    ctx.time:set_target_fps(120)
    ctx.window:set_title("Lua demo")
end

function LuaDemoScene:update(ctx, scene)
    local dt = ctx.time:delta()

    if ctx.input:key_down(key.W) then
        position.y = position.y - 500 * dt
    end

    if ctx.input:key_down(key.A) then
        position.x = position.x - 500 * dt
    end

    if ctx.input:key_down(key.S) then
        position.y = position.y + 500 * dt
    end

    if ctx.input:key_down(key.D) then
        position.x = position.x + 500 * dt
    end
end

function LuaDemoScene:fixed_update(ctx, scene)
    print(position)
end

function LuaDemoScene:draw(ctx, draw)
    draw:rect(position.x, position.y, 50, 50)
end

return LuaDemoScene
