use core::ffi;
use core::mem::MaybeUninit;

use alloc::string::String;
use sdl3_sys::SDL_EVENT_DID_ENTER_BACKGROUND;
use sdl3_sys::SDL_EVENT_DID_ENTER_FOREGROUND;
use sdl3_sys::SDL_EVENT_DROP_FILE;
use sdl3_sys::SDL_EVENT_DROP_TEXT;
use sdl3_sys::SDL_EVENT_FINGER_CANCELED;
use sdl3_sys::SDL_EVENT_FINGER_DOWN;
use sdl3_sys::SDL_EVENT_FINGER_MOTION;
use sdl3_sys::SDL_EVENT_FINGER_UP;
use sdl3_sys::SDL_EVENT_GAMEPAD_ADDED;
use sdl3_sys::SDL_EVENT_GAMEPAD_AXIS_MOTION;
use sdl3_sys::SDL_EVENT_GAMEPAD_BUTTON_DOWN;
use sdl3_sys::SDL_EVENT_GAMEPAD_BUTTON_UP;
use sdl3_sys::SDL_EVENT_GAMEPAD_REMOVED;
use sdl3_sys::SDL_EVENT_KEY_DOWN;
use sdl3_sys::SDL_EVENT_KEY_UP;
use sdl3_sys::SDL_EVENT_LOW_MEMORY;
use sdl3_sys::SDL_EVENT_MOUSE_BUTTON_DOWN;
use sdl3_sys::SDL_EVENT_MOUSE_BUTTON_UP;
use sdl3_sys::SDL_EVENT_MOUSE_MOTION;
use sdl3_sys::SDL_EVENT_MOUSE_WHEEL;
use sdl3_sys::SDL_EVENT_QUIT;
use sdl3_sys::SDL_EVENT_TERMINATING;
use sdl3_sys::SDL_EVENT_TEXT_EDITING;
use sdl3_sys::SDL_EVENT_TEXT_INPUT;
use sdl3_sys::SDL_EVENT_WILL_ENTER_BACKGROUND;
use sdl3_sys::SDL_EVENT_WILL_ENTER_FOREGROUND;
use sdl3_sys::SDL_EVENT_WINDOW_CLOSE_REQUESTED;
use sdl3_sys::SDL_EVENT_WINDOW_DESTROYED;
use sdl3_sys::SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED;
use sdl3_sys::SDL_EVENT_WINDOW_ENTER_FULLSCREEN;
use sdl3_sys::SDL_EVENT_WINDOW_EXPOSED;
use sdl3_sys::SDL_EVENT_WINDOW_FIRST;
use sdl3_sys::SDL_EVENT_WINDOW_FOCUS_GAINED;
use sdl3_sys::SDL_EVENT_WINDOW_FOCUS_LOST;
use sdl3_sys::SDL_EVENT_WINDOW_HIDDEN;
use sdl3_sys::SDL_EVENT_WINDOW_LAST;
use sdl3_sys::SDL_EVENT_WINDOW_LEAVE_FULLSCREEN;
use sdl3_sys::SDL_EVENT_WINDOW_MAXIMIZED;
use sdl3_sys::SDL_EVENT_WINDOW_MINIMIZED;
use sdl3_sys::SDL_EVENT_WINDOW_MOUSE_ENTER;
use sdl3_sys::SDL_EVENT_WINDOW_MOUSE_LEAVE;
use sdl3_sys::SDL_EVENT_WINDOW_MOVED;
use sdl3_sys::SDL_EVENT_WINDOW_OCCLUDED;
use sdl3_sys::SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED;
use sdl3_sys::SDL_EVENT_WINDOW_RESIZED;
use sdl3_sys::SDL_EVENT_WINDOW_RESTORED;
use sdl3_sys::SDL_EVENT_WINDOW_SAFE_AREA_CHANGED;
use sdl3_sys::SDL_EVENT_WINDOW_SHOWN;
use sdl3_sys::SDL_Event;
use sdl3_sys::SDL_EventType;
use sdl3_sys::SDL_KMOD_ALT;
use sdl3_sys::SDL_KMOD_CAPS;
use sdl3_sys::SDL_KMOD_CTRL;
use sdl3_sys::SDL_KMOD_GUI;
use sdl3_sys::SDL_KMOD_NUM;
use sdl3_sys::SDL_KMOD_SHIFT;
use sdl3_sys::SDL_MOUSEWHEEL_FLIPPED;
use sdl3_sys::SDL_PollEvent;
use traccia::trace;

use crate::events::Finger;
use crate::events::GamepadEvent;
use crate::events::KeyEvent;
use crate::events::Keycode;
use crate::events::Lifecycle;
use crate::events::Modifiers;
use crate::events::MouseButton;
use crate::events::MouseEvent;
use crate::events::SDLWindowEvent;
use crate::events::Scancode;
use crate::events::SdlEvent;
use crate::events::TextEvent;
use crate::events::TouchEvent;

const _: () = {
    assert!(Modifiers::SHIFT as u32 == SDL_KMOD_SHIFT);
    assert!(Modifiers::CTRL as u32 == SDL_KMOD_CTRL);
    assert!(Modifiers::ALT as u32 == SDL_KMOD_ALT);
    assert!(Modifiers::GUI as u32 == SDL_KMOD_GUI);
    assert!(Modifiers::CAPS as u32 == SDL_KMOD_CAPS);
    assert!(Modifiers::NUM as u32 == SDL_KMOD_NUM);
};

pub fn poll() -> Poll {
    Poll { _priv: () }
}

pub struct Poll {
    _priv: (),
}

impl Iterator for Poll {
    type Item = SdlEvent;

    fn next(&mut self) -> Option<SdlEvent> {
        loop {
            let mut raw = MaybeUninit::<SDL_Event>::uninit();

            // SAFETY: SDL_PollEvent either fully initialises the union and
            // returns true, or leaves it untouched and returns false.
            if !unsafe { SDL_PollEvent(raw.as_mut_ptr()) } {
                return None;
            }

            let raw = unsafe { raw.assume_init() };

            if let Some(event) = translate(&raw) {
                return Some(event);
            }
        }
    }
}

fn translate(raw: &SDL_Event) -> Option<SdlEvent> {
    let kind = unsafe { raw.type_ };

    match kind {
        SDL_EVENT_QUIT => Some(SdlEvent::Quit),

        // ---- lifecycle ---------------------------------------------------
        SDL_EVENT_TERMINATING => life(Lifecycle::Terminating),
        SDL_EVENT_LOW_MEMORY => life(Lifecycle::LowMemory),
        SDL_EVENT_WILL_ENTER_BACKGROUND => life(Lifecycle::WillEnterBackground),
        SDL_EVENT_DID_ENTER_BACKGROUND => life(Lifecycle::DidEnterBackground),
        SDL_EVENT_WILL_ENTER_FOREGROUND => life(Lifecycle::WillEnterForeground),
        SDL_EVENT_DID_ENTER_FOREGROUND => life(Lifecycle::DidEnterForeground),

        // ---- window ------------------------------------------------------
        _ if is_window(kind) => {
            // SAFETY: every SDL_EVENT_WINDOW_* writes the `window` member.
            let w = unsafe { raw.window };
            let wevent = window_event(kind, w.data1, w.data2)?;

            Some(SdlEvent::Window {
                window: w.windowID,
                wevent,
            })
        }

        // ---- keyboard ----------------------------------------------------
        SDL_EVENT_KEY_DOWN | SDL_EVENT_KEY_UP => {
            // SAFETY: both key events write the `key` member.
            let k = unsafe { raw.key };

            Some(SdlEvent::Key {
                window: k.windowID,
                kevent: KeyEvent {
                    scancode: Scancode(k.scancode),
                    keycode: Keycode(k.key),
                    modifiers: Modifiers(k.mod_),
                    pressed: k.down,
                    repeat: k.repeat,
                },
            })
        }

        // ---- mouse -------------------------------------------------------
        SDL_EVENT_MOUSE_MOTION => {
            // SAFETY: writes the `motion` member.
            let m = unsafe { raw.motion };

            Some(SdlEvent::Mouse {
                window: m.windowID,
                mevent: MouseEvent::Motion {
                    x: m.x,
                    y: m.y,
                    dx: m.xrel,
                    dy: m.yrel,
                },
            })
        }

        SDL_EVENT_MOUSE_BUTTON_DOWN | SDL_EVENT_MOUSE_BUTTON_UP => {
            // SAFETY: both button events write the `button` member.
            let b = unsafe { raw.button };

            Some(SdlEvent::Mouse {
                window: b.windowID,
                mevent: MouseEvent::Button {
                    button: MouseButton::from_raw(b.button),
                    pressed: b.down,
                    clicks: b.clicks,
                    x: b.x,
                    y: b.y,
                },
            })
        }

        SDL_EVENT_MOUSE_WHEEL => {
            // SAFETY: writes the `wheel` member.
            let w = unsafe { raw.wheel };

            // SDL reports FLIPPED for "natural" scrolling rather than negating
            // the deltas, leaving the correction to the app.
            let flipped = w.direction == SDL_MOUSEWHEEL_FLIPPED;
            let sign = if flipped { -1.0 } else { 1.0 };

            Some(SdlEvent::Mouse {
                window: w.windowID,
                mevent: MouseEvent::Wheel {
                    x: w.x * sign,
                    y: w.y * sign,
                    mouse_x: w.mouse_x,
                    mouse_y: w.mouse_y,
                },
            })
        }

        // ---- touch -------------------------------------------------------
        SDL_EVENT_FINGER_DOWN
        | SDL_EVENT_FINGER_UP
        | SDL_EVENT_FINGER_MOTION
        | SDL_EVENT_FINGER_CANCELED => {
            // SAFETY: every finger event writes the `tfinger` member.
            let f = unsafe { raw.tfinger };

            let finger = Finger {
                window: f.windowID,
                touch_id: f.touchID,
                finger_id: f.fingerID,
                x: f.x,
                y: f.y,
                dx: f.dx,
                dy: f.dy,
                pressure: f.pressure,
            };

            Some(SdlEvent::Touch(match kind {
                SDL_EVENT_FINGER_DOWN => TouchEvent::Down(finger),
                SDL_EVENT_FINGER_UP => TouchEvent::Up(finger),
                SDL_EVENT_FINGER_MOTION => TouchEvent::Motion(finger),
                _ => TouchEvent::Canceled(finger),
            }))
        }

        // ---- gamepad -----------------------------------------------------
        SDL_EVENT_GAMEPAD_ADDED => {
            // SAFETY: writes the `gdevice` member.
            let g = unsafe { raw.gdevice };
            Some(SdlEvent::Gamepad(GamepadEvent::Added { id: g.which }))
        }

        SDL_EVENT_GAMEPAD_REMOVED => {
            // SAFETY: writes the `gdevice` member.
            let g = unsafe { raw.gdevice };
            Some(SdlEvent::Gamepad(GamepadEvent::Removed { id: g.which }))
        }

        SDL_EVENT_GAMEPAD_BUTTON_DOWN | SDL_EVENT_GAMEPAD_BUTTON_UP => {
            // SAFETY: both write the `gbutton` member.
            let g = unsafe { raw.gbutton };

            Some(SdlEvent::Gamepad(GamepadEvent::Button {
                id: g.which,
                button: g.button,
                pressed: g.down,
            }))
        }

        SDL_EVENT_GAMEPAD_AXIS_MOTION => {
            // SAFETY: writes the `gaxis` member.
            let g = unsafe { raw.gaxis };

            Some(SdlEvent::Gamepad(GamepadEvent::Axis {
                id: g.which,
                axis: g.axis,
                // i16 is asymmetric: -32768..32767. Divide by 32767 and clamp
                // so a full-left stick reads exactly -1.0 rather than -1.000031.
                value: (g.value as f32 / 32767.0).clamp(-1.0, 1.0),
            }))
        }

        // ---- drag and drop -----------------------------------------------
        SDL_EVENT_DROP_FILE => {
            // SAFETY: writes the `drop` member.
            let d = unsafe { raw.drop };

            Some(SdlEvent::DropFile {
                window: d.windowID,
                path: String::from(unsafe { cstr(d.data) }?),
                x: d.x,
                y: d.y,
            })
        }

        SDL_EVENT_DROP_TEXT => {
            // SAFETY: writes the `drop` member.
            let d = unsafe { raw.drop };

            Some(SdlEvent::DropText {
                window: d.windowID,
                text: unsafe { cstr(d.data) }?,
            })
        }

        SDL_EVENT_TEXT_INPUT => {
            let e = unsafe { raw.text };

            Some(SdlEvent::Text {
                window: e.windowID,
                tevent: TextEvent::Input {
                    text: unsafe { owned(e.text) },
                },
            })
        }

        SDL_EVENT_TEXT_EDITING => {
            let e = unsafe { raw.edit };
            Some(SdlEvent::Text {
                window: e.windowID,
                tevent: TextEvent::Editing {
                    text: unsafe { owned(e.text) },
                    cursor: e.start,
                    len: e.length,
                },
            })
        }

        other => {
            trace!("Unmapped SDL event: {}", other);
            None
        }
    }
}

fn life(l: Lifecycle) -> Option<SdlEvent> {
    Some(SdlEvent::Lifecycle(l))
}

fn is_window(kind: SDL_EventType) -> bool {
    (SDL_EVENT_WINDOW_FIRST..=SDL_EVENT_WINDOW_LAST).contains(&kind)
}

unsafe fn owned(p: *const ffi::c_char) -> String {
    if p.is_null() {
        return String::new();
    }

    unsafe { ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

fn window_event(kind: SDL_EventType, data1: i32, data2: i32) -> Option<SDLWindowEvent> {
    let size = || (data1.max(0) as u32, data2.max(0) as u32);

    Some(match kind {
        SDL_EVENT_WINDOW_SHOWN => SDLWindowEvent::Shown,
        SDL_EVENT_WINDOW_HIDDEN => SDLWindowEvent::Hidden,
        SDL_EVENT_WINDOW_EXPOSED => SDLWindowEvent::Exposed,
        SDL_EVENT_WINDOW_MOVED => SDLWindowEvent::Moved { x: data1, y: data2 },
        SDL_EVENT_WINDOW_RESIZED => {
            let (width, height) = size();
            SDLWindowEvent::Resized { width, height }
        }
        SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
            let (width, height) = size();
            SDLWindowEvent::PixelSizeChanged { width, height }
        }
        SDL_EVENT_WINDOW_MINIMIZED => SDLWindowEvent::Minimized,
        SDL_EVENT_WINDOW_MAXIMIZED => SDLWindowEvent::Maximized,
        SDL_EVENT_WINDOW_RESTORED => SDLWindowEvent::Restored,
        SDL_EVENT_WINDOW_MOUSE_ENTER => SDLWindowEvent::MouseEnter,
        SDL_EVENT_WINDOW_MOUSE_LEAVE => SDLWindowEvent::MouseLeave,
        SDL_EVENT_WINDOW_FOCUS_GAINED => SDLWindowEvent::FocusGained,
        SDL_EVENT_WINDOW_FOCUS_LOST => SDLWindowEvent::FocusLost,
        SDL_EVENT_WINDOW_CLOSE_REQUESTED => SDLWindowEvent::CloseRequested,
        SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED => SDLWindowEvent::DisplayScaleChanged,
        SDL_EVENT_WINDOW_ENTER_FULLSCREEN => SDLWindowEvent::EnterFullscreen,
        SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => SDLWindowEvent::LeaveFullscreen,
        SDL_EVENT_WINDOW_OCCLUDED => SDLWindowEvent::Occluded,
        SDL_EVENT_WINDOW_SAFE_AREA_CHANGED => SDLWindowEvent::SafeAreaChanged,
        SDL_EVENT_WINDOW_DESTROYED => SDLWindowEvent::Destroyed,
        other => {
            trace!("Unmapped SDL window event: {}", other);
            return None;
        }
    })
}

/// # Safety
///
/// `ptr` must be null or a NUL-terminated string valid for the duration of
/// this call. SDL reuses these buffers, so the result is always a copy.
unsafe fn cstr(ptr: *const ffi::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { ffi::CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}
