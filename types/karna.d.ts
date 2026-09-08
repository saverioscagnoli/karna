/**
 * Type definitions for the karna JavaScript bindings.
 *
 * Only plain data on an owned value is a property -- `v.x`, `color.r`. Anything
 * that reaches into the engine or computes on access is a method, so its cost
 * is visible at the call site.
 *
 * The entry module default-exports either a {@link Scene} or an {@link App}.
 * Point your editor at this file to get completion for it -- see
 * `examples/js/jsconfig.json`.
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

        toString(): string;
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

        toString(): string;
    }

    /** Straight (non-premultiplied) RGBA, each component in `0..1`. */
    export class Color {
        constructor(r: number, g: number, b: number, a?: number);

        r: number;
        g: number;
        b: number;
        a: number;

        static rgb(r: number, g: number, b: number): Color;
        static rgba(r: number, g: number, b: number, a: number): Color;
        /** `Color.hex("#89b4fa")`, `"#8bf"`, `"#89b4faff"` or `0x89b4fa`. */
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
        static readonly TRANSPARENT: Color;

        clone(): Color;
        withAlpha(a: number): Color;
        eq(other: ColorLike): boolean;

        toString(): string;
    }

    /** Anywhere a color is expected, a hex string or number will do. */
    export type ColorLike = Color | string | number;

    export function vec2(x: number, y: number): Vec2;
    export function size(width: number, height: number): Size;
    export function color(r: number, g: number, b: number, a?: number): Color;

    /** An opaque token. Passing anything else where one is expected throws. */
    interface Token {
        /** Formatted on each call rather than stored, hence a method. */
        name(): string;
        eq(other: this): boolean;
        toString(): string;
    }

    export interface KeyToken extends Token {}
    export interface ButtonToken extends Token {}
    export interface CursorToken extends Token {}

    /**
     * Every key the engine models, named after SDL's own scancode names with
     * the spaces removed: `Key.W`, `Key.Space`, `Key.LeftShift`, `Key.Escape`.
     * The number row is `Key.Num1` .. `Key.Num0`, since `Key.1` will not parse.
     */
    export const Key: Readonly<Record<string, KeyToken>>;

    export const Button: Readonly<{
        Left: ButtonToken;
        Middle: ButtonToken;
        Right: ButtonToken;
        X1: ButtonToken;
        X2: ButtonToken;
    }>;

    export const Cursor: Readonly<{
        DEFAULT: CursorToken;
        POINTER: CursorToken;
        TEXT: CursorToken;
        WAIT: CursorToken;
        CROSSHAIR: CursorToken;
        PROGRESS: CursorToken;
        MOVE: CursorToken;
        NOT_ALLOWED: CursorToken;
        EW_RESIZE: CursorToken;
        NS_RESIZE: CursorToken;
        NESW_RESIZE: CursorToken;
        NWSE_RESIZE: CursorToken;
    }>;

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

        /**
         * Scene transitions are queued and applied between frames, so calling
         * one does not interrupt the scene that called it. Activating a scene
         * that has never been loaded runs its `load` first.
         *
         * Naming a scene that was not declared throws immediately.
         */
        loadScene(name: string): void;
        unloadScene(name: string): void;
        activateScene(name: string): void;
        deactivateScene(name: string): void;

        /** Ends the run loop after the current frame. */
        quit(): void;
    }

    export interface Time {
        /** Seconds since the previous frame. */
        delta(): number;
        /** The fixed timestep, for use in `fixedUpdate`. */
        fixedDelta(): number;
        fps(): number;
        /** How far this frame sits between two fixed ticks, in `0..1`. */
        alpha(): number;
        /** Seconds of fixed-step time since the run began. */
        elapsed(): number;

        setTargetTps(target: number): void;
    }

    export interface Input {
        /** True for as long as the key is held. */
        keyDown(key: KeyToken): boolean;
        /** True only on the frame the key went down; auto-repeat does not count. */
        keyPressed(key: KeyToken): boolean;
        keyReleased(key: KeyToken): boolean;

        mouseDown(button: ButtonToken): boolean;
        mousePressed(button: ButtonToken): boolean;
        mouseReleased(button: ButtonToken): boolean;

        mouseWheel(): Vec2;
    }

    export interface Draw {
        /** What subsequent calls paint with. */
        color(): Color;
        setColor(color: ColorLike): void;
        /** The size of the drawable area, in pixels. */
        viewport(): Size;

        rect(x: number, y: number, width: number, height: number): void;
    }

    /**
     * The context handed to every callback.
     *
     * It is one object, refilled before each call, so it is valid only for the
     * duration of that call -- using it from a microtask or a stashed reference
     * throws. Inside `draw` the window is read-only: `setTitle` and the scene
     * transitions raise there, since the frame is already being painted.
     */
    export interface Context {
        readonly window: Window;
        readonly time: Time;
        readonly input: Input;
    }

    /**
     * What a scene is.
     *
     * Every method is optional, and each is called with the scene object as
     * `this`, so state can live on `this` between calls. A method that throws is
     * logged with its JS stack, and the scene is switched off rather than
     * allowed to throw again on every frame.
     */
    export interface Scene {
        load?(ctx: Context): void;
        update?(ctx: Context): void;
        /** Runs at the fixed tick rate; use `ctx.time.fixedDelta()`. */
        fixedUpdate?(ctx: Context): void;
        draw?(ctx: Context, draw: Draw): void;
        unload?(ctx: Context): void;

        [key: string]: unknown;
    }

    /**
     * The other thing the entry module may default-export: a window plus a set
     * of named scenes. A default export with lifecycle methods and no `scenes`
     * is taken as a single scene named "main".
     */
    export interface App {
        title?: string;
        width?: number;
        height?: number;
        resizable?: boolean;

        scenes: Record<string, Scene>;
        /** Which scene starts active. Defaults to the first one declared. */
        scene?: string;
    }
}

/**
 * Routed into the engine's logger, tagged with the entry script's name.
 *
 * Note that there is no `setTimeout` and no `fetch`: a game paces itself from
 * `update`, and a timer firing between frames has no context to act on.
 * Promises and microtasks do work -- they are drained after each callback.
 */
declare const console: {
    log(...args: unknown[]): void;
    info(...args: unknown[]): void;
    warn(...args: unknown[]): void;
    error(...args: unknown[]): void;
    debug(...args: unknown[]): void;
    trace(...args: unknown[]): void;
};
