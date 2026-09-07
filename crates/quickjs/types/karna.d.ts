/**
 * Type definitions for the karna JavaScript bindings.
 *
 * Only plain data on an owned value is a property -- `v.x`, `color.r`. Anything
 * that reaches into the engine or computes on access is a method, so its cost is
 * visible at the call site.
 *
 * A scene module default-exports a {@link Scene}. Point your editor at this
 * file to get completion for it -- see `examples/js/jsconfig.json`.
 */
declare module "karna" {
    /** A 2D vector. Every operator is a method; only `x`/`y` and `set` mutate. */
    export class Vec2 {
        constructor(x: number, y: number);

        x: number;
        y: number;

        static zero(): Vec2;
        static one(): Vec2;
        static splat(v: number): Vec2;
        /** The unit vector pointing `angle` radians from `+x`. */
        static fromAngle(angle: number): Vec2;

        set(x: number, y: number): void;
        clone(): Vec2;

        add(other: Vec2): Vec2;
        sub(other: Vec2): Vec2;
        /** Componentwise product; see {@link scale} for the scalar one. */
        mul(other: Vec2): Vec2;
        div(other: Vec2): Vec2;
        scale(factor: number): Vec2;
        neg(): Vec2;
        eq(other: Vec2): boolean;

        length(): number;
        lengthSq(): number;
        normalize(): Vec2;
        perp(): Vec2;
        angle(): number;
        rotate(angle: number): Vec2;
        dot(other: Vec2): number;
        distance(other: Vec2): number;
        lerp(other: Vec2, t: number): Vec2;
    }

    export class Size {
        constructor(width: number, height: number);

        width: number;
        height: number;

        static zero(): Size;
        static square(side: number): Size;

        clone(): Size;
        area(): number;
        aspectRatio(): number;
        scale(factor: number): Size;
        eq(other: Size): boolean;
    }

    /** Linear RGBA, each component in `0..1`. */
    export class Color {
        constructor(r: number, g: number, b: number, a?: number);

        r: number;
        g: number;
        b: number;
        a: number;

        static rgb(r: number, g: number, b: number): Color;
        static rgba(r: number, g: number, b: number, a: number): Color;
        /** `Color.hex("#89b4fa")`, `Color.hex("#89b4faff")` or `Color.hex(0x89b4fa)`. */
        static hex(value: string | number): Color;

        static readonly RED: Color;
        static readonly GREEN: Color;
        static readonly BLUE: Color;
        static readonly WHITE: Color;
        static readonly BLACK: Color;
        static readonly YELLOW: Color;
        static readonly CYAN: Color;
        static readonly MAGENTA: Color;
        static readonly GRAY: Color;
        static readonly ORANGE: Color;
        static readonly PURPLE: Color;
        static readonly BROWN: Color;
        static readonly PINK: Color;

        clone(): Color;
        withAlpha(a: number): Color;
        eq(other: Color): boolean;
    }

    /** Anywhere a color is expected, a hex string or number will do. */
    export type ColorLike = Color | string | number;

    export function vec2(x: number, y: number): Vec2;
    export function size(width: number, height: number): Size;
    export function color(r: number, g: number, b: number, a?: number): Color;

    /** An opaque token. Reading an unknown member of a namespace throws. */
    interface Token {
        /** Formatted on each call rather than stored, hence a method. */
        name(): string;
        eq(other: this): boolean;
    }

    export interface KeyToken extends Token {}
    export interface ButtonToken extends Token {}
    export interface LayerToken extends Token {}
    export interface CursorToken extends Token {}

    /** A handle to an image; loading is asynchronous, drawing is not. */
    export interface ImageHandle extends Token {}
    export interface FontHandle extends Token {}

    /** Every key the engine models, named as in `Key.W`, `Key.Space`. */
    export const Key: Readonly<Record<string, KeyToken>>;
    export const Button: Readonly<{
        Left: ButtonToken;
        Middle: ButtonToken;
        Right: ButtonToken;
        X1: ButtonToken;
        X2: ButtonToken;
    }>;

    /** Layers are drawn in this order, each with its own camera. */
    export const Layer: Readonly<{
        WORLD: LayerToken;
        UI: LayerToken;
        DEBUG: LayerToken;
    }>;

    export const Cursor: Readonly<
        Record<string, CursorToken> & {
            custom(
                image: ImageHandle,
                hotspotX?: number,
                hotspotY?: number,
            ): CursorToken;
        }
    >;

    /** Text that has already been shaped and laid out. */
    export interface Text {
        size(): Size;
        width(): number;
        height(): number;
        content(): string;
    }

    export interface TextStyle {
        font?: FontHandle;
        /** Defaults to 16; setting it also resets `lineHeight` to `size * 1.25`. */
        size?: number;
        lineHeight?: number;
        /** Wrap width in pixels. Omit for no wrapping. */
        wrap?: number;
        align?: "left" | "right" | "center" | "justified" | "end";
    }

    /** One run of text in a call that mixes styles. */
    export interface TextSpan {
        text: string;
        color?: ColorLike;
        font?: FontHandle;
        bold?: boolean;
        italic?: boolean;
    }

    export interface Window {
        title(): string;
        size(): Size;
        width(): number;
        height(): number;
        resizable(): boolean;
        mousePosition(): Vec2;
        /** Movement accumulated since the last frame, in pixels. */
        mouseDelta(): Vec2;

        setTitle(title: string): void;
        setSize(width: number, height: number): void;
        setResizable(value: boolean): void;
        setCursor(cursor: CursorToken): void;
        setPresentMode(mode: "vsync" | "immediate" | "mailbox"): void;

        loadScene(name: string): void;
        unloadScene(name: string): void;
        activateScene(name: string): void;
        deactivateScene(name: string): void;

        startTextInput(): void;
        stopTextInput(): void;
        setTextInputArea(
            x: number,
            y: number,
            width: number,
            height: number,
            cursor?: number,
        ): void;
        clearTextInputArea(): void;
    }

    export interface Time {
        /** Seconds since the previous frame. */
        delta(): number;
        /** The fixed timestep, for use in `fixedUpdate`. */
        fixedDelta(): number;
        fps(): number;
        /** How far this frame sits between two fixed ticks, in `0..1`. */
        alpha(): number;
        /** Wall-clock seconds the last frame took. */
        frame(): number;

        setTargetFps(target: number): void;
        setTargetTps(target: number): void;
    }

    export interface Input {
        /** True for as long as the key is held. */
        keyDown(key: KeyToken): boolean;
        /** True only on the callback where the key went down. */
        keyPressed(key: KeyToken): boolean;
        keyReleased(key: KeyToken): boolean;

        mouseDown(button: ButtonToken): boolean;
        mousePressed(button: ButtonToken): boolean;
        mouseReleased(button: ButtonToken): boolean;

        mouseWheel(): Vec2;
        /** Text committed since the last frame, once text input is started. */
        text(): string;
        /** The IME's in-progress composition, not yet committed. */
        preedit(): string;
        preeditCursor(): number;
    }

    export interface Assets {
        /**
         * Queues `path` (relative to the asset root) and returns its handle
         * immediately. Drawing it before the load finishes draws the
         * placeholder, so there is no need to wait.
         */
        loadImage(path: string): ImageHandle;
        imageSize(image: ImageHandle): Size;
        isImagePending(image: ImageHandle): boolean;
        isImageReady(image: ImageHandle): boolean;
        placeholderImage(): ImageHandle;

        loadFont(path: string): FontHandle;
        /** Looks the font up by family name through the system's fonts. */
        systemFont(name: string): FontHandle;
        fontFamily(font: FontHandle): string | null;
    }

    export interface Draw {
        /** What subsequent calls paint with. */
        color(): Color;
        setColor(color: ColorLike): void;
        /** Which layer subsequent calls land in. */
        layer(): LayerToken;
        setLayer(layer: LayerToken): void;
        /** The size of the drawable area, in pixels. */
        viewport(): Size;

        rect(x: number, y: number, width: number, height: number): void;

        /** Draws `image` at its natural size, tinted by the current color. */
        image(image: ImageHandle, x: number, y: number): void;
        imageSized(
            image: ImageHandle,
            x: number,
            y: number,
            width: number,
            height: number,
        ): void;
        imageSize(image: ImageHandle): Size;

        /**
         * Lays out and draws `content`, returning the size it occupied.
         *
         * Pass an array of spans to mix styles within one run.
         */
        print(
            content: string | TextSpan[],
            style: TextStyle | undefined,
            x: number,
            y: number,
        ): Size;

        /**
         * Lays `content` out without drawing it.
         *
         * Layout is the expensive half of drawing text, so text that does not
         * change every frame is worth laying out once and drawing with
         * {@link text}.
         */
        layout(content: string | TextSpan[], style?: TextStyle): Text;
        text(text: Text, x: number, y: number): void;
    }

    /**
     * The context handed to every callback.
     *
     * It is one object, refilled before each call, so it is valid only for the
     * duration of that call -- using it from anywhere else throws. `assets` is
     * absent during `draw`, and the window is read-only there.
     */
    export interface Context {
        readonly window: Window;
        readonly time: Time;
        readonly input: Input;
        readonly assets: Assets;
    }

    /**
     * What a scene module default-exports.
     *
     * Every method is optional, and each is called with the scene object as
     * `this`, so state can live on `this` between calls.
     */
    export interface Scene {
        load?(ctx: Context): void;
        update?(ctx: Context): void;
        /** Runs at the fixed tick rate; use `ctx.time.fixedDelta`. */
        fixedUpdate?(ctx: Context): void;
        draw?(ctx: Context, draw: Draw): void;
        unload?(ctx: Context): void;

        [key: string]: unknown;
    }
}

/** Routed into the engine's logger, tagged with the script's path. */
declare const console: {
    log(...args: unknown[]): void;
    info(...args: unknown[]): void;
    warn(...args: unknown[]): void;
    error(...args: unknown[]): void;
    debug(...args: unknown[]): void;
    trace(...args: unknown[]): void;
};
