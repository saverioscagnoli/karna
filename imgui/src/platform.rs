use std::ffi::CString;

use dear_imgui_sys::*;
use sdl3::{
    event::{Event, WindowEvent},
    keyboard::Scancode,
    mouse::MouseButton,
};

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Capture {
    pub mouse: bool,
    pub keyboard: bool,
}

pub fn handle_event(io: *mut ImGuiIO, event: &Event) -> Capture {
    unsafe {
        match event {
            Event::MouseMotion { x, y, .. } => {
                ImGuiIO_AddMousePosEvent(io, *x, *y);
            }

            Event::MouseButtonDown { mouse_btn, .. } => {
                if let Some(b) = mouse_button(*mouse_btn) {
                    ImGuiIO_AddMouseButtonEvent(io, b, true);
                }
            }

            Event::MouseButtonUp { mouse_btn, .. } => {
                if let Some(b) = mouse_button(*mouse_btn) {
                    ImGuiIO_AddMouseButtonEvent(io, b, false);
                }
            }

            Event::MouseWheel { x, y, .. } => {
                ImGuiIO_AddMouseWheelEvent(io, *x, *y);
            }

            Event::KeyDown {
                scancode: Some(c),
                keymod,
                ..
            } => {
                add_modifiers(io, *keymod);
                ImGuiIO_AddKeyEvent(io, key(*c), true);
            }

            Event::KeyUp {
                scancode: Some(c),
                keymod,
                ..
            } => {
                add_modifiers(io, *keymod);
                ImGuiIO_AddKeyEvent(io, key(*c), false);
            }

            Event::TextInput { text, .. } => {
                if let Ok(c) = CString::new(text.as_str()) {
                    ImGuiIO_AddInputCharactersUTF8(io, c.as_ptr());
                }
            }

            Event::Window { win_event, .. } => match win_event {
                WindowEvent::FocusGained => ImGuiIO_AddFocusEvent(io, true),
                WindowEvent::FocusLost => ImGuiIO_AddFocusEvent(io, false),
                _ => {}
            },

            _ => {}
        }

        Capture {
            mouse: (*io).WantCaptureMouse,
            keyboard: (*io).WantCaptureKeyboard,
        }
    }
}

pub fn new_frame(io: *mut ImGuiIO, logical: math::Size<u32>, pixel: math::Size<u32>, delta: f32) {
    unsafe {
        (*io).DisplaySize = ImVec2_c {
            x: logical.width as f32,
            y: logical.height as f32,
        };

        (*io).DisplayFramebufferScale = ImVec2_c {
            x: pixel.width as f32 / logical.width.max(1) as f32,
            y: pixel.height as f32 / logical.height.max(1) as f32,
        };

        (*io).DeltaTime = delta.max(1.0 / 1000.0);

        igNewFrame();
    }
}

fn mouse_button(btn: MouseButton) -> Option<ImGuiMouseButton> {
    match btn {
        MouseButton::Left => Some(ImGuiMouseButton_Left),
        MouseButton::Right => Some(ImGuiMouseButton_Right),
        MouseButton::Middle => Some(ImGuiMouseButton_Middle),
        _ => None,
    }
}

unsafe fn add_modifiers(io: *mut ImGuiIO, keymod: sdl3::keyboard::Mod) {
    use sdl3::keyboard::Mod;

    unsafe {
        let ctrl = keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD);
        let shift = keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD);
        let alt = keymod.intersects(Mod::LALTMOD | Mod::RALTMOD);
        let sup = keymod.intersects(Mod::LGUIMOD | Mod::RGUIMOD);

        ImGuiIO_AddKeyEvent(io, ImGuiMod_Ctrl, ctrl);
        ImGuiIO_AddKeyEvent(io, ImGuiMod_Shift, shift);
        ImGuiIO_AddKeyEvent(io, ImGuiMod_Alt, alt);
        ImGuiIO_AddKeyEvent(io, ImGuiMod_Super, sup);
    }
}

fn key(code: Scancode) -> ImGuiKey {
    match code {
        Scancode::Tab => ImGuiKey_Tab,
        Scancode::Left => ImGuiKey_LeftArrow,
        Scancode::Right => ImGuiKey_RightArrow,
        Scancode::Up => ImGuiKey_UpArrow,
        Scancode::Down => ImGuiKey_DownArrow,
        Scancode::PageUp => ImGuiKey_PageUp,
        Scancode::PageDown => ImGuiKey_PageDown,
        Scancode::Home => ImGuiKey_Home,
        Scancode::End => ImGuiKey_End,
        Scancode::Insert => ImGuiKey_Insert,
        Scancode::Delete => ImGuiKey_Delete,
        Scancode::Backspace => ImGuiKey_Backspace,
        Scancode::Space => ImGuiKey_Space,
        Scancode::Return => ImGuiKey_Enter,
        Scancode::Escape => ImGuiKey_Escape,
        Scancode::KpEnter => ImGuiKey_KeypadEnter,

        Scancode::LCtrl => ImGuiKey_LeftCtrl,
        Scancode::LShift => ImGuiKey_LeftShift,
        Scancode::LAlt => ImGuiKey_LeftAlt,
        Scancode::LGui => ImGuiKey_LeftSuper,
        Scancode::RCtrl => ImGuiKey_RightCtrl,
        Scancode::RShift => ImGuiKey_RightShift,
        Scancode::RAlt => ImGuiKey_RightAlt,
        Scancode::RGui => ImGuiKey_RightSuper,

        c if (Scancode::A as i32..=Scancode::Z as i32).contains(&(c as i32)) => {
            ImGuiKey_A + (c as i32 - Scancode::A as i32)
        }

        c if (Scancode::_1 as i32..=Scancode::_9 as i32).contains(&(c as i32)) => {
            ImGuiKey_1 + (c as i32 - Scancode::_1 as i32)
        }

        Scancode::_0 => ImGuiKey_0,

        _ => ImGuiKey_None,
    }
}
