// Type declarations for karna scripts.
//
// Reference it from a script with:
//   /// <reference path="karna.d.ts" />
// and annotate the scene with JSDoc (`/** @implements {Scene} */`) for
// completions. Keep in sync with libs/js/src/api.rs.
//
// Everything engine-side lives on the global `karna` object. It is usable
// while a scene method (or the scene constructor) runs; drawing goes through
// the `Draw` handle passed to `draw`.

// ---------------------------------------------------------------------------
// Basics
// ---------------------------------------------------------------------------

interface Vec2 {
  x: number;
  y: number;
}

interface Size {
  width: number;
  height: number;
}

type SizeLike = Size | [width: number, height: number];

declare class KarnaSize implements Size {
  constructor(width: number, height: number);
  width: number;
  height: number;
}

interface Vec3 {
  x: number;
  y: number;
  z: number;
}

/** Components are 0..1. */
interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

/** A color object, `[r, g, b, a?]`, or a hex string like `"#ff8800"`. */
type ColorLike =
  | Color
  | { r: number; g: number; b: number; a?: number }
  | [r: number, g: number, b: number, a?: number]
  | string;

declare class KarnaColor implements Color {
  constructor(r: number, g: number, b: number, a?: number);
  r: number;
  g: number;
  b: number;
  a: number;
}

/** An image handle from `karna.assets.loadImage`. */
interface KarnaImage {
  readonly __brand: "KarnaImage";
}

/** A sound handle from `karna.assets.loadAudio` / `loadAudioStream`. */
interface KarnaAudio {
  readonly __brand: "KarnaAudio";
}

/** A font handle from `karna.assets.loadFont`. */
interface KarnaFont {
  readonly __brand: "KarnaFont";
}

/** A playing sound, from `karna.audio.play`. */
interface KarnaVoice {
  readonly __brand: "KarnaVoice";
}

/** Raw file contents. */
type Bytes = ArrayBuffer | Uint8Array;

/** A scene name, as registered on the window. */
type SceneId = string;

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

type GamepadButtonName =
  | "South"
  | "East"
  | "West"
  | "North"
  | "Back"
  | "Guide"
  | "Start"
  | "LeftStick"
  | "RightStick"
  | "LeftShoulder"
  | "RightShoulder"
  | "DpadUp"
  | "DpadDown"
  | "DpadLeft"
  | "DpadRight"
  | "Misc1"
  | "RightPaddle1"
  | "LeftPaddle1"
  | "RightPaddle2"
  | "LeftPaddle2"
  | "Touchpad"
  | "Misc2"
  | "Misc3"
  | "Misc4"
  | "Misc5"
  | "Misc6";

type GamepadAxisName =
  | "LeftX"
  | "LeftY"
  | "RightX"
  | "RightY"
  | "LeftTrigger"
  | "RightTrigger";

type GamepadKind =
  | "unknown"
  | "standard"
  | "xbox360"
  | "xboxOne"
  | "ps3"
  | "ps4"
  | "ps5"
  | "switchPro"
  | "joyConLeft"
  | "joyConRight"
  | "joyConPair"
  | "gameCube";

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/** A layer to draw on; later layers are drawn over earlier ones. */
type LayerName = "world" | "ui" | "debug";

type TextAlign = "left" | "center" | "right" | "justified" | "end";

interface TextStyle {
  /** Defaults to the debug font. */
  font?: KarnaFont;
  /** Defaults to the font's own size. */
  size?: number;
  /** Multiple of the font size; defaults to 1.25. */
  lineHeight?: number;
  /** Wrap width in pixels; no wrapping when unset. */
  wrap?: number;
  align?: TextAlign;
  bold?: boolean;
  italic?: boolean;
}

/**
 * Passed to `draw`. Only usable while that call runs; don't keep it around.
 */
interface Draw {
  layer(): LayerName;
  setLayer(layer: LayerName): void;

  color(): Color;
  setColor(color: ColorLike): void;

  thickness(): number;
  setThickness(t: number): void;

  textStyle(): TextStyle;
  /** Used by `print` from now on; unset fields take their defaults. */
  setTextStyle(style: TextStyle): void;

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
  /** A filled convex polygon. */
  polygon(points: Vec2[]): void;
  circle(x: number, y: number, radius: number): void;
  circleOutline(x: number, y: number, radius: number): void;
  /** Drawn at its own size unless both `w` and `h` are given. */
  image(image: KarnaImage, x: number, y: number, w?: number, h?: number): void;
  print(text: string, x: number, y: number): void;
}

// ---------------------------------------------------------------------------
// karna.window
// ---------------------------------------------------------------------------

type PresentModeName = "Vsync" | "Mailbox" | "Immediate";

type FullscreenMode =
  | { readonly kind: "borderless" }
  | {
      readonly kind: "exclusive";
      readonly width: number;
      readonly height: number;
      readonly refreshRate: number;
    };

interface Monitor {
  id(): number;
  position(): Vec2;
  size(): Size;
  pixelDensity(): number;
  refreshRate(): number;
}

/** Setters throw during `draw`. */
interface WindowApi {
  title(): string;
  setTitle(title: string): void;

  /** Size in screen coordinates. */
  size(): Size;
  setSize(size: SizeLike): void;
  /** Size in pixels; differs from `size` on high density displays. */
  pixelSize(): Size;
  aspectRatio(): number;

  mousePosition(): Vec2;
  mouseDelta(): Vec2;

  opacity(): number;
  setOpacity(opacity: number): void;

  presentMode(): number;
  setPresentMode(mode: number): void;

  clearColor(): Color;
  setClearColor(color: ColorLike): void;

  isWindowed(): boolean;
  isMaximized(): boolean;
  isMinimized(): boolean;
  isFullscreen(): boolean;
  setWindowed(): void;
  setMaximized(): void;
  setMinimized(): void;
  /** Defaults to borderless. */
  setFullscreen(mode?: FullscreenMode): void;
  /** Back from maximized or minimized. */
  restore(): void;

  isHidden(): boolean;
  setHidden(hidden: boolean): void;
  isResizable(): boolean;
  setResizable(resizable: boolean): void;
  isDecorated(): boolean;
  setDecorated(decorated: boolean): void;
  isAlwaysOnTop(): boolean;
  setAlwaysOnTop(onTop: boolean): void;
  isFocusable(): boolean;
  setFocusable(focusable: boolean): void;
  isTransparent(): boolean;
  isHighPixelDensity(): boolean;

  isMouseGrabbed(): boolean;
  setMouseGrabbed(grabbed: boolean): void;
  isKeyboardGrabbed(): boolean;
  setKeyboardGrabbed(grabbed: boolean): void;
  /** Hides the cursor and reports only `mouseDelta`. */
  isRelativeMouse(): boolean;
  setRelativeMouse(relative: boolean): void;

  /** The monitor the window is mostly on. */
  monitor(): Monitor | undefined;
}

// ---------------------------------------------------------------------------
// karna.time
// ---------------------------------------------------------------------------

type FpsStrategy = "mean" | "smoothed";

/** Setters throw during `draw`. */
interface TimeApi {
  /** Seconds since the window opened. */
  elapsed(): number;
  /** Seconds since the last frame. */
  delta(): number;
  /** Seconds per fixed update tick. */
  fixedDelta(): number;
  /** How far between two fixed ticks this frame is, 0..1. */
  alpha(): number;
  fps(): number;

  setTargetFps(fps: number): void;
  setTargetTps(tps: number): void;
  setFpsStrategy(strategy: FpsStrategy): void;
}

// ---------------------------------------------------------------------------
// karna.input
// ---------------------------------------------------------------------------

interface Pad {
  id(): number;
  /** Player slot, if it has one. */
  slot(): number | undefined;
  name(): string;
  kind(): GamepadKind;

  /** `button` is a `karna.GamepadButton`. */
  down(button: number): boolean;
  pressed(button: number): boolean;
  released(button: number): boolean;

  /** `axis` is a `karna.GamepadAxis`; sticks are -1..1, triggers 0..1. */
  axis(axis: number): number;
  leftStick(): Vec2;
  rightStick(): Vec2;
  leftTrigger(): number;
  rightTrigger(): number;

  /** Intensities are 0..1; throws during `draw`. */
  rumble(intensity: number, seconds: number): void;
  rumbleMotors(low: number, high: number, seconds: number): void;
  rumbleTriggers(left: number, right: number, seconds: number): void;
  stopRumble(): void;
}

interface InputApi {
  /** `key` is a `karna.Key`. */
  keyDown(key: number): boolean;
  keyPressed(key: number): boolean;
  keyReleased(key: number): boolean;

  /** `button` is a `karna.Mouse` button. */
  mouseDown(button: number): boolean;
  mousePressed(button: number): boolean;
  mouseReleased(button: number): boolean;
  wheel(): Vec2;

  /** Text typed since the last frame. */
  text(): string;
  /** Text being composed by an input method, not committed yet. */
  preedit(): string;
  preeditCursor(): number;

  /** The gamepad in a player slot. */
  pad(slot: number): Pad | undefined;
  pads(): Pad[];
  /** `button` is a `karna.GamepadButton`, checked on every pad. */
  anyPadDown(button: number): boolean;
  anyPadPressed(button: number): boolean;
  anyPadReleased(button: number): boolean;
}

// ---------------------------------------------------------------------------
// karna.assets
// ---------------------------------------------------------------------------

/**
 * Loading is asynchronous: handles are usable right away and draw a
 * placeholder (or play silence) until the file is ready. Paths are relative
 * to the asset root. Throws during `draw`.
 */
interface AssetsApi {
  loadImage(path: string): KarnaImage;
  loadImageBytes(bytes: Bytes): KarnaImage;
  placeholderImage(): KarnaImage;

  /** Decoded fully up front; for short sounds. */
  loadAudio(path: string): KarnaAudio;
  loadAudioBytes(bytes: Bytes): KarnaAudio;
  /** Decoded while playing; for music. */
  loadAudioStream(path: string): KarnaAudio;
  loadAudioStreamBytes(bytes: Bytes): KarnaAudio;

  loadFont(path: string, size?: number): KarnaFont;
  loadFontBytes(bytes: Bytes, size?: number): KarnaFont;
  debugFont(): KarnaFont;
}

// ---------------------------------------------------------------------------
// karna.audio
// ---------------------------------------------------------------------------

interface PlayOptions {
  /** Defaults to false. */
  loop?: boolean;
  /** Defaults to 1. */
  gain?: number;
}

/** Throws during `draw`. */
interface AudioApi {
  play(audio: KarnaAudio, options?: PlayOptions): KarnaVoice;
  stop(voice: KarnaVoice): void;
  setGain(voice: KarnaVoice, gain: number): void;
}

// ---------------------------------------------------------------------------
// karna.scene
// ---------------------------------------------------------------------------

type Projection =
  | {
      kind: "orthographic";
      left: number;
      right: number;
      bottom: number;
      top: number;
      near: number;
      far: number;
    }
  | {
      kind: "perspective";
      /** Vertical field of view, in radians. */
      fov: number;
      aspectRatio: number;
      near: number;
      far: number;
    };

interface Camera {
  projection(): Projection;
  setProjection(projection: Projection): void;
  position(): Vec3;
  setPosition(x: number, y: number, z: number): void;
  translate(dx: number, dy: number, dz: number): void;
  target(): Vec3;
  setTarget(x: number, y: number, z: number): void;
}

/** Changes are applied after the current frame; they throw during `draw`. */
interface SceneApi {
  /** This scene's id. */
  id(): SceneId;
  /** Runs another scene alongside the active ones. */
  activate(scene: SceneId): void;
  deactivate(scene: SceneId): void;
  /** Replaces this scene with another. */
  change(to: SceneId): void;
  /** The camera a layer is drawn with. */
  camera(layer: LayerName): Camera;
}

// ---------------------------------------------------------------------------
// karna.monitors
// ---------------------------------------------------------------------------

interface MonitorsApi {
  all(): Monitor[];
  primary(): Monitor | undefined;
}

// ---------------------------------------------------------------------------
// Scenes
// ---------------------------------------------------------------------------

/**
 * What a script `export default`s: a class whose instances look like this, or
 * a plain object that does. Every method is optional.
 */
interface Scene {
  load?(): void;
  /** Runs at the fixed tick rate (`karna.time.setTargetTps`). */
  fixedUpdate?(): void;
  /** Runs once per frame. */
  update?(): void;
  draw?(draw: Draw): void;
  unload?(): void;
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

// ---------------------------------------------------------------------------
// Globals
// ---------------------------------------------------------------------------

declare const karna: {
  readonly window: WindowApi;
  readonly time: TimeApi;
  readonly input: InputApi;
  readonly assets: AssetsApi;
  readonly audio: AudioApi;
  readonly scene: SceneApi;
  readonly monitors: MonitorsApi;

  readonly Key: { readonly [K in KeyName]: number };
  readonly Mouse: { readonly [B in MouseButtonName]: number };
  readonly PresentMode: { readonly [M in PresentModeName]: number };
  readonly FullscreenMode: {
    readonly Borderless: FullscreenMode;
    Exclusive(width: number, height: number, refreshRate?: number): FullscreenMode;
  };
  readonly GamepadButton: { readonly [B in GamepadButtonName]: number };
  readonly GamepadAxis: { readonly [A in GamepadAxisName]: number };

  readonly Color: typeof KarnaColor;
  readonly Size: typeof KarnaSize;

  /** Builds the `window` export of the entry script. */
  readonly WindowBuilder: typeof KarnaWindowBuilder;
};

/** Arguments are joined with spaces and sent to the engine log. */
interface Console {
  log(...args: unknown[]): void;
  trace(...args: unknown[]): void;
  debug(...args: unknown[]): void;
  info(...args: unknown[]): void;
  warn(...args: unknown[]): void;
  error(...args: unknown[]): void;
}

declare var console: Console;
