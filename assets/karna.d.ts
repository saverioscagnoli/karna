// Type declarations for karna scripts.
//
// Reference it from a script with:
//   /// <reference path="karna.d.ts" />
// and annotate the scene with JSDoc (`/** @implements {Scene} */`, `@param
// {UpdateContext} ctx`) for completions. Keep in sync with
// libs/script/src/api.rs.

interface Vec2 {
  x: number;
  y: number;
}

/** An image handle from `ctx.assets.loadImage`. */
interface KarnaImage {
  readonly __brand: "KarnaImage";
}

type KeyName =
  | "A"
  | "B"
  | "C"
  | "D"
  | "E"
  | "F"
  | "G"
  | "H"
  | "I"
  | "J"
  | "K"
  | "L"
  | "M"
  | "N"
  | "O"
  | "P"
  | "Q"
  | "R"
  | "S"
  | "T"
  | "U"
  | "V"
  | "W"
  | "X"
  | "Y"
  | "Z"
  | "Num1"
  | "Num2"
  | "Num3"
  | "Num4"
  | "Num5"
  | "Num6"
  | "Num7"
  | "Num8"
  | "Num9"
  | "Num0"
  | "Return"
  | "Escape"
  | "Backspace"
  | "Tab"
  | "Space"
  | "Minus"
  | "Equals"
  | "LeftBracket"
  | "RightBracket"
  | "Backslash"
  | "NonUsHash"
  | "Semicolon"
  | "Apostrophe"
  | "Grave"
  | "Comma"
  | "Period"
  | "Slash"
  | "CapsLock"
  | "F1"
  | "F2"
  | "F3"
  | "F4"
  | "F5"
  | "F6"
  | "F7"
  | "F8"
  | "F9"
  | "F10"
  | "F11"
  | "F12"
  | "PrintScreen"
  | "ScrollLock"
  | "Pause"
  | "Insert"
  | "Home"
  | "PageUp"
  | "Delete"
  | "End"
  | "PageDown"
  | "Right"
  | "Left"
  | "Down"
  | "Up"
  | "NumLockClear"
  | "KpDivide"
  | "KpMultiply"
  | "KpMinus"
  | "KpPlus"
  | "KpEnter"
  | "Kp1"
  | "Kp2"
  | "Kp3"
  | "Kp4"
  | "Kp5"
  | "Kp6"
  | "Kp7"
  | "Kp8"
  | "Kp9"
  | "Kp0"
  | "KpPeriod"
  | "NonUsBackslash"
  | "Application"
  | "Power"
  | "KpEquals"
  | "F13"
  | "F14"
  | "F15"
  | "F16"
  | "F17"
  | "F18"
  | "F19"
  | "F20"
  | "F21"
  | "F22"
  | "F23"
  | "F24"
  | "Execute"
  | "Help"
  | "Menu"
  | "Select"
  | "Stop"
  | "Again"
  | "Undo"
  | "Cut"
  | "Copy"
  | "Paste"
  | "Find"
  | "Mute"
  | "VolumeUp"
  | "VolumeDown"
  | "KpComma"
  | "KpEqualsAs400"
  | "International1"
  | "International2"
  | "International3"
  | "International4"
  | "International5"
  | "International6"
  | "International7"
  | "International8"
  | "International9"
  | "Lang1"
  | "Lang2"
  | "Lang3"
  | "Lang4"
  | "Lang5"
  | "Lang6"
  | "Lang7"
  | "Lang8"
  | "Lang9"
  | "AltErase"
  | "SysReq"
  | "Cancel"
  | "Clear"
  | "Prior"
  | "Return2"
  | "Separator"
  | "Out"
  | "Oper"
  | "ClearAgain"
  | "CrSel"
  | "ExSel"
  | "Kp00"
  | "Kp000"
  | "ThousandsSeparator"
  | "DecimalSeparator"
  | "CurrencyUnit"
  | "CurrencySubUnit"
  | "KpLeftParen"
  | "KpRightParen"
  | "KpLeftBrace"
  | "KpRightBrace"
  | "KpTab"
  | "KpBackspace"
  | "KpA"
  | "KpB"
  | "KpC"
  | "KpD"
  | "KpE"
  | "KpF"
  | "KpXor"
  | "KpPower"
  | "KpPercent"
  | "KpLess"
  | "KpGreater"
  | "KpAmpersand"
  | "KpDblAmpersand"
  | "KpVerticalBar"
  | "KpDblVerticalBar"
  | "KpColon"
  | "KpHash"
  | "KpSpace"
  | "KpAt"
  | "KpExclam"
  | "KpMemStore"
  | "KpMemRecall"
  | "KpMemClear"
  | "KpMemAdd"
  | "KpMemSubtract"
  | "KpMemMultiply"
  | "KpMemDivide"
  | "KpPlusMinus"
  | "KpClear"
  | "KpClearEntry"
  | "KpBinary"
  | "KpOctal"
  | "KpDecimal"
  | "KpHexadecimal"
  | "LCtrl"
  | "LShift"
  | "LAlt"
  | "LGui"
  | "RCtrl"
  | "RShift"
  | "RAlt"
  | "RGui"
  | "Mode"
  | "Sleep"
  | "Wake"
  | "ChannelIncrement"
  | "ChannelDecrement"
  | "MediaPlay"
  | "MediaPause"
  | "MediaRecord"
  | "MediaFastForward"
  | "MediaRewind"
  | "MediaNextTrack"
  | "MediaPreviousTrack"
  | "MediaStop"
  | "MediaEject"
  | "MediaPlayPause"
  | "MediaSelect"
  | "AcNew"
  | "AcOpen"
  | "AcClose"
  | "AcExit"
  | "AcSave"
  | "AcPrint"
  | "AcProperties"
  | "AcSearch"
  | "AcHome"
  | "AcBack"
  | "AcForward"
  | "AcStop"
  | "AcRefresh"
  | "AcBookmarks"
  | "SoftLeft"
  | "SoftRight"
  | "Call"
  | "EndCall";

type MouseButtonName = "Left" | "Middle" | "Right" | "X1" | "X2";

/** A layer to draw on; later layers are drawn over earlier ones. */
type LayerName = "world" | "ui" | "debug";

interface Draw {
  /** Components are 0..1; alpha defaults to 1. */
  setColor(r: number, g: number, b: number, a?: number): void;
  setThickness(t: number): void;
  setLayer(layer: LayerName): void;

  line(x1: number, y1: number, x2: number, y2: number): void;
  rect(x: number, y: number, w: number, h: number): void;
  rectOutline(x: number, y: number, w: number, h: number): void;
  triangle(
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    x3: number,
    y3: number,
  ): void;
  circle(x: number, y: number, radius: number): void;
  circleOutline(x: number, y: number, radius: number): void;
  /** Drawn at its own size unless both `w` and `h` are given. */
  image(image: KarnaImage, x: number, y: number, w?: number, h?: number): void;
  print(text: string, x: number, y: number): void;
}

interface WindowApi {
  title(): string;
  setTitle(title: string): void;
  width(): number;
  height(): number;
  setSize(width: number, height: number): void;
  mouse(): Vec2;
  mouseDelta(): Vec2;
}

interface TimeApi {
  /** Seconds since the last frame. */
  delta(): number;
  /** Seconds per fixed update tick. */
  fixedDelta(): number;
  /** How far between two fixed ticks this frame is, 0..1. */
  alpha(): number;
  fps(): number;
  setTargetFps(fps: number): void;
  setTargetTps(tps: number): void;
}

interface InputApi {
  keyDown(key: number): boolean;
  keyPressed(key: number): boolean;
  keyReleased(key: number): boolean;
  mouseDown(button: number): boolean;
  mousePressed(button: number): boolean;
  mouseReleased(button: number): boolean;
  wheel(): Vec2;
  /** Text typed since the last frame. */
  text(): string;
}

interface AssetsApi {
  /** Path relative to the asset root. */
  loadImage(path: string): KarnaImage;
}

/**
 * Passed to `draw`. Like every context, it is only usable while the method it
 * was passed to runs; don't keep it around.
 */
interface DrawContext {
  window: WindowApi;
  time: TimeApi;
  input: InputApi;
}

/** Passed to `load`, `update`, `fixedUpdate` and `unload`. */
interface UpdateContext extends DrawContext {
  assets: AssetsApi;
}

type LoadContext = UpdateContext;

/**
 * What a script `export default`s: a class whose instances look like this, or
 * a plain object that does. Every method is optional.
 */
interface Scene {
  load?(ctx: LoadContext): void;
  update?(ctx: UpdateContext): void;
  fixedUpdate?(ctx: UpdateContext): void;
  draw?(ctx: DrawContext, draw: Draw): void;
  unload?(ctx: LoadContext): void;
}

/**
 * Window settings, read once before the window opens. Export one from the
 * entry script as `window`; unset fields keep the defaults.
 */
interface WindowConfig {
  title?: string;
  width?: number;
  height?: number;
  resizable?: boolean;
}

declare class KarnaWindowBuilder implements WindowConfig {
  title?: string;
  width?: number;
  height?: number;
  resizable?: boolean;

  withTitle(title: string): this;
  withSize(width: number, height: number): this;
  withResizable(resizable?: boolean): this;
}

declare const karna: {
  /** Logs its arguments, space separated, at info level. */
  log(...args: unknown[]): void;

  readonly Key: { readonly [K in KeyName]: number };
  readonly Mouse: { readonly [B in MouseButtonName]: number };

  /** Builds the `window` export of the entry script. */
  readonly WindowBuilder: typeof KarnaWindowBuilder;
};
