use crate::context::{Context, Flags};
use karna_log::{info, KarnaError};
use karna_math::size::Size;
use karna_traits::Scene;
use sdl3::{event::Event, EventPump};
use std::time::Instant;

pub struct App {
    window_data: (String, Size<u32>),
}

impl App {
    pub fn window<T: Into<String>, S: Into<Size<u32>>>(title: T, size: S) -> Self {
        let title: String = title.into();
        let size: Size<u32> = size.into();

        Self {
            window_data: (title, size),
        }
    }

    pub fn run<S: Scene<Context>>(&mut self, mut first_scene: S) {
        let (title, size) = self.window_data.clone();

        let mut context = Context::init(title, size, Some(&[Flags::default()]));
        let mut events = context
            .sdl
            .event_pump()
            .map_err(|e| KarnaError::Sdl("Retrieving the event pump".to_string(), e.to_string()))
            .unwrap();

        first_scene.load(&mut context);

        let mut t0 = Instant::now();

        while !context.should_close {
            let t1 = Instant::now();
            let dt = t1.duration_since(t0).as_secs_f32();

            t0 = t1;

            App::handle_events(&mut events, &mut context);

            // Upodate the time interface before updating the scene,
            // So that the time interface is up-to-date.
            context.time.update(dt);

            first_scene.update(&mut context);
            first_scene.draw(&mut context);

            context.window.swap_buffers();
        }

        info!("App was terminated.");
    }

    fn handle_events(events: &mut EventPump, context: &mut Context) {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => context.should_close = true,

                _ => {}
            }
        }
    }
}
