local Player = require("player")

local Key, Mouse = karna.Key, karna.Mouse

--@class Demo : Scene
local Demo = {}
Demo.__index = Demo

function Demo:new()
    return setmetatable({ player = Player.new(), trail = {} }, Demo)
end

function Demo:load()
    self.pcb = karna.assets.load_image("assets/pcb.png")
    self.osu = karna.assets.load_audio("assets/osu-hit-sound.wav")

    local size = karna.window.size()
    print("script loaded, window is", size.width, "x", size.height)
end

function Demo:update()
    self.player:update()

    if karna.input.mouse_pressed(Mouse.Left) then
        table.insert(self.trail, karna.window.mouse_position())
        if #self.trail > 32 then table.remove(self.trail, 1) end
    end

    if karna.input.key_pressed(Key.Space) then self.trail = {} end

    if karna.input.key_pressed(Key.Y) then
        karna.audio.play(self.osu)
    end
end

function Demo:draw(g)
    if self.pcb then
        g.image(self.pcb, karna.window.size().width - 266, 10, 256, 256)
    end

    self.player:draw(g)

    g.set_color({ 1, 0.3, 0.8 })

    for _, p in ipairs(self.trail) do g.circle(p.x, p.y, 6) end

    g.set_color({ 1, 1, 1 })
    g.print(("fps: %d"):format(math.floor(karna.time.fps() + 0.5)), 10, 10)
    g.print("WASD / arrows to move, click to drop dots, space to clear", 10, 30)
    g.print(("dt: %s"):format(karna.time.delta()), 10, 50)
end

return Demo
