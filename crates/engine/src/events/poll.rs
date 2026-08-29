//! SDL event translation.
//!
//! This is the *only* module in the engine permitted to name an SDL type. It
//! turns `SDL_Event` into [`SDLEvent`] and nothing else, so a platform that is
//! not SDL (a console backend) needs a sibling of this file rather than a
//! change to every call site. `tests/no_sdl_leak.rs` enforces that.
//!
//! # Safety
//!
//! `SDL_Event` is a C union. Reading a member other than the one SDL actually
//! wrote is undefined behaviour, so every read here is gated on `type_` — the
//! one field valid to read unconditionally, since every member begins with it.
//! Each `unsafe` block below is immediately preceded by the match arm that
//! justifies it.

use std::mem::MaybeUninit;
use std::path::PathBuf;

use logging::trace;
use sdl3::SDL_Event;
use sdl3::SDL_EventType;
use sdl3::SDL_PollEvent;

use crate::events::sdl::Finger;
use crate::events::sdl::GamepadEvent;
use crate::events::sdl::KeyEvent;
use crate::events::sdl::Keycode;
use crate::events::sdl::Lifecycle;
use crate::events::sdl::Modifiers;
use crate::events::sdl::MouseButton;
use crate::events::sdl::MouseEvent;
use crate::events::sdl::SDLEvent;
use crate::events::sdl::SDLWindowEvent;
use crate::events::sdl::Scancode;
use crate::events::sdl::TextEvent;
use crate::events::sdl::TouchEvent;
use crate::events::sdl::WindowId;

/// `Modifiers` hardcodes SDL's KMOD bits so that `event.rs` stays free of SDL.
/// This pins them to the headers: if SDL renumbers a modifier, this fails to
/// compile instead of silently reporting the wrong key.
const _: () = {
    assert!(Modifiers::SHIFT as u32 == sdl3::SDL_KMOD_SHIFT);
    assert!(Modifiers::CTRL as u32 == sdl3::SDL_KMOD_CTRL);
    assert!(Modifiers::ALT as u32 == sdl3::SDL_KMOD_ALT);
    assert!(Modifiers::GUI as u32 == sdl3::SDL_KMOD_GUI);
    assert!(Modifiers::CAPS as u32 == sdl3::SDL_KMOD_CAPS);
    assert!(Modifiers::NUM as u32 == sdl3::SDL_KMOD_NUM);
};

/// Drains the SDL event queue, yielding translated events.
///
/// Must be called on the thread that initialised the video subsystem.
///
/// Unmapped SDL events are skipped rather than ending the iteration — SDL
/// posts plenty the engine does not model (sensors, cameras, audio hotplug),
/// and stopping at the first one would strand every later event in the queue
/// until the next frame.
pub fn poll() -> Poll {
    Poll { _priv: () }
}

pub struct Poll {
    _priv: (),
}

impl Iterator for Poll {
    type Item = SDLEvent;

    fn next(&mut self) -> Option<SDLEvent> {
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

/// Returns `None` for events the engine does not model.
fn translate(raw: &SDL_Event) -> Option<SDLEvent> {
    // SAFETY: `type_` overlaps the first field of every union member, so it is
    // always valid to read regardless of which member SDL wrote.
    let kind = SDL_EventType(unsafe { raw.type_ });

    match kind {
        SDL_EventType::SDL_EVENT_QUIT => Some(SDLEvent::Quit),

        // ---- lifecycle ---------------------------------------------------
        SDL_EventType::SDL_EVENT_TERMINATING => life(Lifecycle::Terminating),
        SDL_EventType::SDL_EVENT_LOW_MEMORY => life(Lifecycle::LowMemory),
        SDL_EventType::SDL_EVENT_WILL_ENTER_BACKGROUND => life(Lifecycle::WillEnterBackground),
        SDL_EventType::SDL_EVENT_DID_ENTER_BACKGROUND => life(Lifecycle::DidEnterBackground),
        SDL_EventType::SDL_EVENT_WILL_ENTER_FOREGROUND => life(Lifecycle::WillEnterForeground),
        SDL_EventType::SDL_EVENT_DID_ENTER_FOREGROUND => life(Lifecycle::DidEnterForeground),

        // ---- window ------------------------------------------------------
        _ if is_window(kind) => {
            // SAFETY: every SDL_EVENT_WINDOW_* writes the `window` member.
            let w = unsafe { raw.window };
            let wevent = window_event(kind, w.data1, w.data2)?;

            Some(SDLEvent::Window {
                window: WindowId(w.windowID),
                wevent,
            })
        }

        // ---- keyboard ----------------------------------------------------
        SDL_EventType::SDL_EVENT_KEY_DOWN | SDL_EventType::SDL_EVENT_KEY_UP => {
            // SAFETY: both key events write the `key` member.
            let k = unsafe { raw.key };

            Some(SDLEvent::Key {
                window: WindowId(k.windowID),
                kevent: KeyEvent {
                    scancode: Scancode(k.scancode.0),
                    keycode: Keycode(k.key),
                    modifiers: Modifiers(k.mod_),
                    pressed: k.down,
                    repeat: k.repeat,
                },
            })
        }

        // ---- mouse -------------------------------------------------------
        SDL_EventType::SDL_EVENT_MOUSE_MOTION => {
            // SAFETY: writes the `motion` member.
            let m = unsafe { raw.motion };

            Some(SDLEvent::Mouse {
                window: WindowId(m.windowID),
                mevent: MouseEvent::Motion {
                    x: m.x,
                    y: m.y,
                    dx: m.xrel,
                    dy: m.yrel,
                },
            })
        }

        SDL_EventType::SDL_EVENT_MOUSE_BUTTON_DOWN | SDL_EventType::SDL_EVENT_MOUSE_BUTTON_UP => {
            // SAFETY: both button events write the `button` member.
            let b = unsafe { raw.button };

            Some(SDLEvent::Mouse {
                window: WindowId(b.windowID),
                mevent: MouseEvent::Button {
                    button: MouseButton::from_raw(b.button),
                    pressed: b.down,
                    clicks: b.clicks,
                    x: b.x,
                    y: b.y,
                },
            })
        }

        SDL_EventType::SDL_EVENT_MOUSE_WHEEL => {
            // SAFETY: writes the `wheel` member.
            let w = unsafe { raw.wheel };

            // SDL reports FLIPPED for "natural" scrolling rather than negating
            // the deltas, leaving the correction to the app.
            let flipped = w.direction == sdl3::SDL_MouseWheelDirection::SDL_MOUSEWHEEL_FLIPPED;
            let sign = if flipped { -1.0 } else { 1.0 };

            Some(SDLEvent::Mouse {
                window: WindowId(w.windowID),
                mevent: MouseEvent::Wheel {
                    x: w.x * sign,
                    y: w.y * sign,
                    mouse_x: w.mouse_x,
                    mouse_y: w.mouse_y,
                },
            })
        }

        // ---- touch -------------------------------------------------------
        SDL_EventType::SDL_EVENT_FINGER_DOWN
        | SDL_EventType::SDL_EVENT_FINGER_UP
        | SDL_EventType::SDL_EVENT_FINGER_MOTION
        | SDL_EventType::SDL_EVENT_FINGER_CANCELED => {
            // SAFETY: every finger event writes the `tfinger` member.
            let f = unsafe { raw.tfinger };

            let finger = Finger {
                window: WindowId(f.windowID),
                touch_id: f.touchID,
                finger_id: f.fingerID,
                x: f.x,
                y: f.y,
                dx: f.dx,
                dy: f.dy,
                pressure: f.pressure,
            };

            Some(SDLEvent::Touch(match kind {
                SDL_EventType::SDL_EVENT_FINGER_DOWN => TouchEvent::Down(finger),
                SDL_EventType::SDL_EVENT_FINGER_UP => TouchEvent::Up(finger),
                SDL_EventType::SDL_EVENT_FINGER_MOTION => TouchEvent::Motion(finger),
                _ => TouchEvent::Canceled(finger),
            }))
        }

        // ---- gamepad -----------------------------------------------------
        SDL_EventType::SDL_EVENT_GAMEPAD_ADDED => {
            // SAFETY: writes the `gdevice` member.
            let g = unsafe { raw.gdevice };
            Some(SDLEvent::Gamepad(GamepadEvent::Added { id: g.which }))
        }

        SDL_EventType::SDL_EVENT_GAMEPAD_REMOVED => {
            // SAFETY: writes the `gdevice` member.
            let g = unsafe { raw.gdevice };
            Some(SDLEvent::Gamepad(GamepadEvent::Removed { id: g.which }))
        }

        SDL_EventType::SDL_EVENT_GAMEPAD_BUTTON_DOWN
        | SDL_EventType::SDL_EVENT_GAMEPAD_BUTTON_UP => {
            // SAFETY: both write the `gbutton` member.
            let g = unsafe { raw.gbutton };

            Some(SDLEvent::Gamepad(GamepadEvent::Button {
                id: g.which,
                button: g.button,
                pressed: g.down,
            }))
        }

        SDL_EventType::SDL_EVENT_GAMEPAD_AXIS_MOTION => {
            // SAFETY: writes the `gaxis` member.
            let g = unsafe { raw.gaxis };

            Some(SDLEvent::Gamepad(GamepadEvent::Axis {
                id: g.which,
                axis: g.axis,
                // i16 is asymmetric: -32768..32767. Divide by 32767 and clamp
                // so a full-left stick reads exactly -1.0 rather than -1.000031.
                value: (g.value as f32 / 32767.0).clamp(-1.0, 1.0),
            }))
        }

        // ---- drag and drop -----------------------------------------------
        SDL_EventType::SDL_EVENT_DROP_FILE => {
            // SAFETY: writes the `drop` member.
            let d = unsafe { raw.drop };

            Some(SDLEvent::DropFile {
                window: WindowId(d.windowID),
                path: PathBuf::from(unsafe { cstr(d.data) }?),
                x: d.x,
                y: d.y,
            })
        }

        SDL_EventType::SDL_EVENT_DROP_TEXT => {
            // SAFETY: writes the `drop` member.
            let d = unsafe { raw.drop };

            Some(SDLEvent::DropText {
                window: WindowId(d.windowID),
                text: unsafe { cstr(d.data) }?,
            })
        }

        SDL_EventType::SDL_EVENT_TEXT_INPUT => {
            let e = unsafe { raw.text };

            Some(SDLEvent::Text {
                window: WindowId(e.windowID),
                tevent: TextEvent::Input {
                    text: unsafe { owned(e.text) },
                },
            })
        }

        SDL_EventType::SDL_EVENT_TEXT_EDITING => {
            let e = unsafe { raw.edit };
            Some(SDLEvent::Text {
                window: WindowId(e.windowID),
                tevent: TextEvent::Editing {
                    text: unsafe { owned(e.text) },
                    cursor: e.start,
                    len: e.length,
                },
            })
        }

        other => {
            trace!("Unmapped SDL event: {}", other.0);
            None
        }
    }
}

fn life(l: Lifecycle) -> Option<SDLEvent> {
    Some(SDLEvent::Lifecycle(l))
}

fn is_window(kind: SDL_EventType) -> bool {
    (SDL_EventType::SDL_EVENT_WINDOW_FIRST.0..=SDL_EventType::SDL_EVENT_WINDOW_LAST.0)
        .contains(&kind.0)
}

unsafe fn owned(p: *const std::ffi::c_char) -> String {
    if p.is_null() {
        return String::new();
    }

    unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

fn window_event(kind: SDL_EventType, data1: i32, data2: i32) -> Option<SDLWindowEvent> {
    let size = || (data1.max(0) as u32, data2.max(0) as u32);

    Some(match kind {
        SDL_EventType::SDL_EVENT_WINDOW_SHOWN => SDLWindowEvent::Shown,
        SDL_EventType::SDL_EVENT_WINDOW_HIDDEN => SDLWindowEvent::Hidden,
        SDL_EventType::SDL_EVENT_WINDOW_EXPOSED => SDLWindowEvent::Exposed,
        SDL_EventType::SDL_EVENT_WINDOW_MOVED => SDLWindowEvent::Moved { x: data1, y: data2 },
        SDL_EventType::SDL_EVENT_WINDOW_RESIZED => {
            let (width, height) = size();
            SDLWindowEvent::Resized { width, height }
        }
        SDL_EventType::SDL_EVENT_WINDOW_PIXEL_SIZE_CHANGED => {
            let (width, height) = size();
            SDLWindowEvent::PixelSizeChanged { width, height }
        }
        SDL_EventType::SDL_EVENT_WINDOW_MINIMIZED => SDLWindowEvent::Minimized,
        SDL_EventType::SDL_EVENT_WINDOW_MAXIMIZED => SDLWindowEvent::Maximized,
        SDL_EventType::SDL_EVENT_WINDOW_RESTORED => SDLWindowEvent::Restored,
        SDL_EventType::SDL_EVENT_WINDOW_MOUSE_ENTER => SDLWindowEvent::MouseEnter,
        SDL_EventType::SDL_EVENT_WINDOW_MOUSE_LEAVE => SDLWindowEvent::MouseLeave,
        SDL_EventType::SDL_EVENT_WINDOW_FOCUS_GAINED => SDLWindowEvent::FocusGained,
        SDL_EventType::SDL_EVENT_WINDOW_FOCUS_LOST => SDLWindowEvent::FocusLost,
        SDL_EventType::SDL_EVENT_WINDOW_CLOSE_REQUESTED => SDLWindowEvent::CloseRequested,
        SDL_EventType::SDL_EVENT_WINDOW_DISPLAY_SCALE_CHANGED => {
            SDLWindowEvent::DisplayScaleChanged
        }
        SDL_EventType::SDL_EVENT_WINDOW_ENTER_FULLSCREEN => SDLWindowEvent::EnterFullscreen,
        SDL_EventType::SDL_EVENT_WINDOW_LEAVE_FULLSCREEN => SDLWindowEvent::LeaveFullscreen,
        SDL_EventType::SDL_EVENT_WINDOW_OCCLUDED => SDLWindowEvent::Occluded,
        SDL_EventType::SDL_EVENT_WINDOW_SAFE_AREA_CHANGED => SDLWindowEvent::SafeAreaChanged,
        SDL_EventType::SDL_EVENT_WINDOW_DESTROYED => SDLWindowEvent::Destroyed,
        other => {
            trace!("Unmapped SDL window event: {}", other.0);
            return None;
        }
    })
}

/// # Safety
///
/// `ptr` must be null or a NUL-terminated string valid for the duration of
/// this call. SDL reuses these buffers, so the result is always a copy.
unsafe fn cstr(ptr: *const std::ffi::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { std::ffi::CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}
