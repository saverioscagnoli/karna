---@meta karna

---@class Vec2
---@field x number
---@field y number

---@class Size
---@field width integer
---@field height integer

---@alias SizeLike Size | integer[]

---@class Color
---@field r number
---@field g number
---@field b number
---@field a number

---@alias ColorLike Color | number[] | string

---@class KarnaImage

---@class KarnaAudio

---@class KarnaVoice

---@class PlayOptions
---@field loop? boolean
---@field gain? number

---@class FullscreenModeBorderless
---@field kind "borderless"

---@class FullscreenModeExclusive
---@field kind "exclusive"
---@field width integer
---@field height integer
---@field refresh_rate number

---@alias FullscreenMode FullscreenModeBorderless | FullscreenModeExclusive

---@alias Layer "world" | "ui" | "debug"

---@class WindowBuilder
---@field title? string
---@field width? integer
---@field height? integer
---@field resizable? boolean
local WindowBuilder = {}

---@param title string
---@return WindowBuilder
function WindowBuilder:with_title(title) end

---@param width integer
---@param height integer
---@return WindowBuilder
function WindowBuilder:with_size(width, height) end

---@param resizable? boolean
---@return WindowBuilder
function WindowBuilder:with_resizable(resizable) end

---@class Graphics
local Graphics = {}

---@return Color
function Graphics.color() end

---@param color ColorLike
function Graphics.set_color(color) end

---@param thickness number
function Graphics.set_thickness(thickness) end

---@param layer Layer
function Graphics.set_layer(layer) end

---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
function Graphics.line(x1, y1, x2, y2) end

---@param x number
---@param y number
---@param w number
---@param h number
function Graphics.rect(x, y, w, h) end

---@param x number
---@param y number
---@param w number
---@param h number
function Graphics.rect_outline(x, y, w, h) end

---@param x1 number
---@param y1 number
---@param x2 number
---@param y2 number
---@param x3 number
---@param y3 number
function Graphics.triangle(x1, y1, x2, y2, x3, y3) end

---@param x number
---@param y number
---@param r number
function Graphics.circle(x, y, r) end

---@param x number
---@param y number
---@param r number
function Graphics.circle_outline(x, y, r) end

---@param text string
---@param x number
---@param y number
function Graphics.print(text, x, y) end

---@param image KarnaImage
---@param x number
---@param y number
---@param w? number
---@param h? number
function Graphics.image(image, x, y, w, h) end

---@class Scene
---@field window? WindowBuilder
---@field new? fun(self: Scene): Scene
---@field load? fun(self: Scene)
---@field update? fun(self: Scene)
---@field fixed_update? fun(self: Scene)
---@field draw? fun(self: Scene, g: Graphics)
---@field unload? fun(self: Scene)

karna = {}

---@return WindowBuilder
function karna.WindowBuilder() end

---@param r number
---@param g number
---@param b number
---@param a? number
---@return Color
function karna.Color(r, g, b, a) end

---@param width integer
---@param height integer
---@return Size
function karna.Size(width, height) end

karna.FullscreenMode = {
    ---@type FullscreenModeBorderless
    Borderless = { kind = "borderless" },
}

---@param width integer
---@param height integer
---@param refresh_rate? number
---@return FullscreenModeExclusive
function karna.FullscreenMode.Exclusive(width, height, refresh_rate) end

karna.window = {}

---@return string
function karna.window.title() end

---@return Size
function karna.window.size() end

---@return Size
function karna.window.pixel_size() end

---@return number
function karna.window.aspect_ratio() end

---@return Vec2
function karna.window.mouse_position() end

---@return Vec2
function karna.window.mouse_delta() end

---@return karna.PresentMode
function karna.window.present_mode() end

---@return boolean
function karna.window.is_windowed() end

---@return boolean
function karna.window.is_maximized() end

---@return boolean
function karna.window.is_minimized() end

---@return boolean
function karna.window.is_fullscreen() end

---@return boolean
function karna.window.is_hidden() end

---@return boolean
function karna.window.is_resizable() end

---@return boolean
function karna.window.is_decorated() end

---@return boolean
function karna.window.is_always_on_top() end

---@return boolean
function karna.window.is_utility() end

---@return boolean
function karna.window.is_transparent() end

---@return boolean
function karna.window.is_focusable() end

---@return boolean
function karna.window.is_high_pixel_density() end

---@return boolean
function karna.window.is_mouse_grabbed() end

---@return boolean
function karna.window.is_keyboard_grabbed() end

---@return boolean
function karna.window.is_relative_mouse() end

---@return number
function karna.window.opacity() end

---@return Color
function karna.window.clear_color() end

---@param title string
function karna.window.set_title(title) end

---@param size SizeLike
function karna.window.set_size(size) end

function karna.window.set_windowed() end

function karna.window.set_maximized() end

function karna.window.set_minimized() end

---@param mode? FullscreenMode
function karna.window.set_fullscreen(mode) end

---@param hidden boolean
function karna.window.set_hidden(hidden) end

---@param resizable boolean
function karna.window.set_resizable(resizable) end

---@param decorated boolean
function karna.window.set_decorated(decorated) end

---@param on_top boolean
function karna.window.set_always_on_top(on_top) end

---@param focusable boolean
function karna.window.set_focusable(focusable) end

---@param grabbed boolean
function karna.window.set_mouse_grabbed(grabbed) end

---@param grabbed boolean
function karna.window.set_keyboard_grabbed(grabbed) end

---@param relative boolean
function karna.window.set_relative_mouse(relative) end

---@param opacity number
function karna.window.set_opacity(opacity) end

---@param mode karna.PresentMode
function karna.window.set_present_mode(mode) end

---@param color ColorLike
function karna.window.set_clear_color(color) end

function karna.window.restore() end

karna.time = {}

---@return number
function karna.time.delta() end

---@return number
function karna.time.fixed_delta() end

---@return number
function karna.time.alpha() end

---@return number
function karna.time.fps() end

---@param fps integer
function karna.time.set_target_fps(fps) end

---@param tps integer
function karna.time.set_target_tps(tps) end

karna.input = {}

---@param key karna.Key
---@return boolean
function karna.input.key_down(key) end

---@param key karna.Key
---@return boolean
function karna.input.key_pressed(key) end

---@param key karna.Key
---@return boolean
function karna.input.key_released(key) end

---@param button karna.Mouse
---@return boolean
function karna.input.mouse_down(button) end

---@param button karna.Mouse
---@return boolean
function karna.input.mouse_pressed(button) end

---@param button karna.Mouse
---@return boolean
function karna.input.mouse_released(button) end

---@return string
function karna.input.text() end

---@return Vec2
function karna.input.wheel() end

karna.assets = {}

---@param path string
---@return KarnaImage
function karna.assets.load_image(path) end

---@param path string
---@return KarnaAudio
function karna.assets.load_audio(path) end

karna.audio = {}

---@param audio KarnaAudio
---@param options? PlayOptions
---@return KarnaVoice
function karna.audio.play(audio, options) end

---@param voice KarnaVoice
function karna.audio.stop(voice) end

---@param voice KarnaVoice
---@param gain number
function karna.audio.set_gain(voice, gain) end

karna.log = {}

---@param ... any
function karna.log.trace(...) end

---@param ... any
function karna.log.debug(...) end

---@param ... any
function karna.log.info(...) end

---@param ... any
function karna.log.warn(...) end

---@param ... any
function karna.log.error(...) end

---@enum karna.Mouse
karna.Mouse = {
    Left = 0,
    Middle = 1,
    Right = 2,
    X1 = 3,
    X2 = 4,
}

---@enum karna.PresentMode
karna.PresentMode = {
    Vsync = 0,
    Mailbox = 1,
    Immediate = 2,
}

---@enum karna.Key
karna.Key = {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
    J = 9,
    K = 10,
    L = 11,
    M = 12,
    N = 13,
    O = 14,
    P = 15,
    Q = 16,
    R = 17,
    S = 18,
    T = 19,
    U = 20,
    V = 21,
    W = 22,
    X = 23,
    Y = 24,
    Z = 25,
    Num1 = 26,
    Num2 = 27,
    Num3 = 28,
    Num4 = 29,
    Num5 = 30,
    Num6 = 31,
    Num7 = 32,
    Num8 = 33,
    Num9 = 34,
    Num0 = 35,
    Return = 36,
    Escape = 37,
    Backspace = 38,
    Tab = 39,
    Space = 40,
    Minus = 41,
    Equals = 42,
    LeftBracket = 43,
    RightBracket = 44,
    Backslash = 45,
    NonUsHash = 46,
    Semicolon = 47,
    Apostrophe = 48,
    Grave = 49,
    Comma = 50,
    Period = 51,
    Slash = 52,
    CapsLock = 53,
    F1 = 54,
    F2 = 55,
    F3 = 56,
    F4 = 57,
    F5 = 58,
    F6 = 59,
    F7 = 60,
    F8 = 61,
    F9 = 62,
    F10 = 63,
    F11 = 64,
    F12 = 65,
    PrintScreen = 66,
    ScrollLock = 67,
    Pause = 68,
    Insert = 69,
    Home = 70,
    PageUp = 71,
    Delete = 72,
    End = 73,
    PageDown = 74,
    Right = 75,
    Left = 76,
    Down = 77,
    Up = 78,
    NumLockClear = 79,
    KpDivide = 80,
    KpMultiply = 81,
    KpMinus = 82,
    KpPlus = 83,
    KpEnter = 84,
    Kp1 = 85,
    Kp2 = 86,
    Kp3 = 87,
    Kp4 = 88,
    Kp5 = 89,
    Kp6 = 90,
    Kp7 = 91,
    Kp8 = 92,
    Kp9 = 93,
    Kp0 = 94,
    KpPeriod = 95,
    NonUsBackslash = 96,
    Application = 97,
    Power = 98,
    KpEquals = 99,
    F13 = 100,
    F14 = 101,
    F15 = 102,
    F16 = 103,
    F17 = 104,
    F18 = 105,
    F19 = 106,
    F20 = 107,
    F21 = 108,
    F22 = 109,
    F23 = 110,
    F24 = 111,
    Execute = 112,
    Help = 113,
    Menu = 114,
    Select = 115,
    Stop = 116,
    Again = 117,
    Undo = 118,
    Cut = 119,
    Copy = 120,
    Paste = 121,
    Find = 122,
    Mute = 123,
    VolumeUp = 124,
    VolumeDown = 125,
    KpComma = 126,
    KpEqualsAs400 = 127,
    International1 = 128,
    International2 = 129,
    International3 = 130,
    International4 = 131,
    International5 = 132,
    International6 = 133,
    International7 = 134,
    International8 = 135,
    International9 = 136,
    Lang1 = 137,
    Lang2 = 138,
    Lang3 = 139,
    Lang4 = 140,
    Lang5 = 141,
    Lang6 = 142,
    Lang7 = 143,
    Lang8 = 144,
    Lang9 = 145,
    AltErase = 146,
    SysReq = 147,
    Cancel = 148,
    Clear = 149,
    Prior = 150,
    Return2 = 151,
    Separator = 152,
    Out = 153,
    Oper = 154,
    ClearAgain = 155,
    CrSel = 156,
    ExSel = 157,
    Kp00 = 158,
    Kp000 = 159,
    ThousandsSeparator = 160,
    DecimalSeparator = 161,
    CurrencyUnit = 162,
    CurrencySubUnit = 163,
    KpLeftParen = 164,
    KpRightParen = 165,
    KpLeftBrace = 166,
    KpRightBrace = 167,
    KpTab = 168,
    KpBackspace = 169,
    KpA = 170,
    KpB = 171,
    KpC = 172,
    KpD = 173,
    KpE = 174,
    KpF = 175,
    KpXor = 176,
    KpPower = 177,
    KpPercent = 178,
    KpLess = 179,
    KpGreater = 180,
    KpAmpersand = 181,
    KpDblAmpersand = 182,
    KpVerticalBar = 183,
    KpDblVerticalBar = 184,
    KpColon = 185,
    KpHash = 186,
    KpSpace = 187,
    KpAt = 188,
    KpExclam = 189,
    KpMemStore = 190,
    KpMemRecall = 191,
    KpMemClear = 192,
    KpMemAdd = 193,
    KpMemSubtract = 194,
    KpMemMultiply = 195,
    KpMemDivide = 196,
    KpPlusMinus = 197,
    KpClear = 198,
    KpClearEntry = 199,
    KpBinary = 200,
    KpOctal = 201,
    KpDecimal = 202,
    KpHexadecimal = 203,
    LCtrl = 204,
    LShift = 205,
    LAlt = 206,
    LGui = 207,
    RCtrl = 208,
    RShift = 209,
    RAlt = 210,
    RGui = 211,
    Mode = 212,
    Sleep = 213,
    Wake = 214,
    ChannelIncrement = 215,
    ChannelDecrement = 216,
    MediaPlay = 217,
    MediaPause = 218,
    MediaRecord = 219,
    MediaFastForward = 220,
    MediaRewind = 221,
    MediaNextTrack = 222,
    MediaPreviousTrack = 223,
    MediaStop = 224,
    MediaEject = 225,
    MediaPlayPause = 226,
    MediaSelect = 227,
    AcNew = 228,
    AcOpen = 229,
    AcClose = 230,
    AcExit = 231,
    AcSave = 232,
    AcPrint = 233,
    AcProperties = 234,
    AcSearch = 235,
    AcHome = 236,
    AcBack = 237,
    AcForward = 238,
    AcStop = 239,
    AcRefresh = 240,
    AcBookmarks = 241,
    SoftLeft = 242,
    SoftRight = 243,
    Call = 244,
    EndCall = 245,
}
