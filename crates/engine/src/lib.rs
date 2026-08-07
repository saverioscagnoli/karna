mod builder;
mod config;
mod err;
mod input;
mod render;
mod scene;
mod window;

use std::any::Any;
use std::marker::PhantomData;
use std::mem;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use logging::debug;
use logging::error;
use logging::info;
use sdl3::SDL_Event;
use sdl3::SDL_EventType;
use sdl3::SDL_INIT_VIDEO;
use sdl3::SDL_Init;
use sdl3::SDL_PollEvent;
use sdl3::SDL_Quit;
use utils::FastHashMap;

use crate::builder::WindowBuilder;
use crate::err::SDL_LastError;
use crate::input::Input;
use crate::input::keys::Key;
use crate::input::mouse::MouseButton;
use crate::render::draw::Draw;
use crate::window::WindowId;
use crate::window::context::UserContext;
use crate::window::platform::PlatformWindow;
use crate::window::state::SceneSlot;
use crate::window::state::WindowState;

static SDL_ACTIVE: AtomicBool = AtomicBool::new(false);

struct WindowEntry {
    platform: PlatformWindow,
    state: WindowState,
}

pub struct App {
    requested_at_creation: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowEntry>,
    should_quit: bool,

    input: Input,

    /// Makes app !Send and !Sync, pinning SDL
    /// to the thread that created it
    _sdl: PhantomData<*const ()>,
}

impl App {
    pub fn new() -> Result<Self, String> {
        if SDL_ACTIVE.swap(true, Ordering::AcqRel) {
            return Err("SDL is already initialized.".into());
        }

        let success = unsafe { SDL_Init(SDL_INIT_VIDEO) };

        if !success {
            SDL_ACTIVE.store(false, Ordering::Release);
            return Err(SDL_LastError());
        }

        debug!("SDL v{} initialized.", sdl3::compiled_version_string());

        Ok(Self {
            requested_at_creation: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            input: Input::default(),
            _sdl: PhantomData,
        })
    }

    fn spawn_window(&mut self, builder: WindowBuilder) {
        #[rustfmt::skip]
        let WindowBuilder { title, size, scenes, active_scenes } = builder;

        let window = PlatformWindow::new(&title, size);
        let scenes = scenes
            .into_iter()
            .map(|(id, scene)| (id, SceneSlot::Unloaded(scene)))
            .collect::<FastHashMap<_, _>>();

        let state = WindowState {
            ctx: UserContext {},
            draw: Draw::new(),
            scenes,
            active_scenes,
        };

        self.windows.insert(
            window.id(),
            WindowEntry {
                platform: window,
                state,
            },
        );
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            error!("Received close request for a closed window.");
            return;
        };

        if self.windows.is_empty() {
            self.should_quit = true;
        }

        info!(
            "Closing window (id: {}, title: {})",
            entry.platform.id(),
            entry.platform.title()
        );
    }

    fn quit(&mut self) {
        debug!("Quit signal received");
        let ids = self.windows.keys().copied().collect::<Vec<_>>();

        for id in ids {
            self.close_window(id);
        }
    }

    fn handle_sdl_event(&mut self, event: SDL_Event) {
        match SDL_EventType(unsafe { event.type_ }) {
            SDL_EventType::SDL_EVENT_QUIT => self.quit(),

            SDL_EventType::SDL_EVENT_WINDOW_CLOSE_REQUESTED => {
                let window_id = unsafe { event.window.windowID };
                self.close_window(window_id);
            }

            SDL_EventType::SDL_EVENT_WINDOW_FOCUS_GAINED => {
                let window_id = unsafe { event.window.windowID };
                self.input.focused = Some(window_id);
                debug!("Window '{}' gained focus.", window_id);
            }

            SDL_EventType::SDL_EVENT_WINDOW_FOCUS_LOST => {
                let window_id = unsafe { event.window.windowID };

                if self.input.focused == Some(window_id) {
                    self.input.focused = None;
                    self.input.keys.clear_all();
                    self.input.mouse.clear_all();
                    debug!("Window '{}' lost focus.", window_id);
                }
            }

            SDL_EventType::SDL_EVENT_KEY_DOWN => {
                if !unsafe { event.key.repeat } {
                    let window_id = unsafe { event.key.windowID };
                    let scancode = unsafe { event.key.scancode };

                    if self.input.focused == Some(window_id)
                        && let Some(k) = Key::from_scancode(scancode)
                    {
                        self.input.keys.press(k);
                    }
                }
            }

            SDL_EventType::SDL_EVENT_KEY_UP => {
                let scancode = unsafe { event.key.scancode };

                if let Some(k) = Key::from_scancode(scancode) {
                    self.input.keys.release(k);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_BUTTON_DOWN => {
                let window_id = unsafe { event.button.windowID };
                let btn = unsafe { event.button.button };

                if self.input.focused == Some(window_id)
                    && let Some(btn) = MouseButton::from_index(btn)
                {
                    self.input.mouse.press(btn);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_BUTTON_UP => {
                let btn = unsafe { event.button.button };

                if let Some(btn) = MouseButton::from_index(btn) {
                    self.input.mouse.release(btn);
                }
            }

            SDL_EventType::SDL_EVENT_MOUSE_WHEEL => {
                let x = unsafe { event.wheel.x };
                let y = unsafe { event.wheel.y };

                self.input.m_wheel += math::Vector2::new(x, y);
            }

            event => {}
        }
    }

    pub fn run(mut self) {
        for builder in mem::take(&mut self.requested_at_creation) {
            self.spawn_window(builder);
        }

        let mut event: MaybeUninit<SDL_Event> = MaybeUninit::uninit();

        while !self.should_quit {
            while unsafe { SDL_PollEvent(event.as_mut_ptr()) } {
                let event = unsafe { event.assume_init() };
                self.handle_sdl_event(event);
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { SDL_Quit() };
        SDL_ACTIVE.store(false, Ordering::Release);
    }
}
