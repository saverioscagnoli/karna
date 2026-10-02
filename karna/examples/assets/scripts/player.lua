local Key = karna.Key

local SPEED = 300

local Player = {}
Player.__index = Player

function Player.new()
    return setmetatable({ x = 100, y = 100, size = 40 }, Player)
end

function Player:update()
    local step = SPEED * karna.time.delta()
    local down = karna.input.key_down

    if down(Key.W) or down(Key.Up) then self.y = self.y - step end
    if down(Key.S) or down(Key.Down) then self.y = self.y + step end
    if down(Key.A) or down(Key.Left) then self.x = self.x - step end
    if down(Key.D) or down(Key.Right) then self.x = self.x + step end
end

function Player:draw(g)
    g.set_color({ 0.2, 0.8, 1 })
    g.rect(self.x, self.y, self.size, self.size)
end

return Player
