use std::any::Any;
use std::fmt;
use std::mem;

use logging::warn;
use utils::FastHashMap;
use winit::keyboard::KeyCode;

use crate::context::ContextMut;
use crate::context::Draw;
use crate::context::SceneHandle;

pub type SceneBuilder = Box<dyn FnOnce(ContextMut, &mut SceneHandle) -> Box<dyn Scene> + Send>;

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: ContextMut, scene: &mut SceneHandle);
    fn loaded_with(&mut self, ctx: ContextMut, scene: &mut SceneHandle, user_data: Box<dyn Any>) {}
    fn fixed_update(&mut self, ctx: ContextMut, scene: &mut SceneHandle) {}
    fn update(&mut self, ctx: ContextMut, scene: &mut SceneHandle);
    fn draw(&mut self, ctx: ContextMut, draw: &mut Draw);

    // Events
    fn on_resize(&mut self, ctx: ContextMut, scene: &mut SceneHandle, size: math::Size<u32>) {}
    fn on_key_press(&mut self, ctx: ContextMut, scene: &mut SceneHandle, keys: &[KeyCode]) {}
    fn on_text_input(&mut self, ctx: ContextMut, scene: &mut SceneHandle, text: &str) {}
}

enum Slot {
    Pending(SceneBuilder),
    Building, // transient; only observable if a builder re-enters its own label
    Built(Box<dyn Scene>),
}

struct Entry {
    label: String,
    slot: Slot,
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entry({})", self.label)
    }
}

#[derive(Default)]
#[derive(Debug)]
pub struct Scenes {
    entries: Vec<Entry>,
    index: FastHashMap<String, usize>,
}

impl Scenes {
    pub fn index_of(&self, label: &str) -> Option<usize> {
        self.index.get(label).copied()
    }

    fn insert(&mut self, label: String, slot: Slot) -> usize {
        if let Some(&i) = self.index.get(&label) {
            warn!("scene {label:?} already registered; ignoring");
            return i;
        }

        let i = self.entries.len();
        self.index.insert(label.clone(), i);
        self.entries.push(Entry { label, slot });
        i
    }

    pub fn insert_builder<L: Into<String>>(&mut self, label: L, builder: SceneBuilder) -> usize {
        self.insert(label.into(), Slot::Pending(builder))
    }

    /// Registers an already-constructed scene. No factory needed: it has no
    /// handles to resolve, or the caller resolved them already.
    pub fn insert_built<L: Into<String>>(&mut self, label: L, scene: Box<dyn Scene>) -> usize {
        self.insert(label.into(), Slot::Built(scene))
    }

    /// Runs the builder if this entry is still pending. Returns `true` if a
    /// build actually happened.
    pub fn build(&mut self, i: usize, ctx: ContextMut, handle: &mut SceneHandle) -> bool {
        let entry = &mut self.entries[i];

        match mem::replace(&mut entry.slot, Slot::Building) {
            Slot::Pending(builder) => {
                let scene = builder(ctx, handle);
                self.entries[i].slot = Slot::Built(scene);
                true
            }

            Slot::Building => {
                warn!(
                    "scene {:?} activated from inside its own builder",
                    entry.label
                );
                false
            }

            built => {
                entry.slot = built;
                false
            }
        }
    }

    pub fn get_mut(&mut self, i: usize) -> &mut Box<dyn Scene> {
        match &mut self.entries[i].slot {
            Slot::Built(s) => s,
            _ => panic!("Scene not build yet"),
        }
    }
}

pub enum SceneCommand {
    Register {
        label: String,
        scene: Box<dyn Scene>,
    },
    Activate {
        label: String,
        user_data: Option<Box<dyn Any>>,
    },
    Deactivate {
        label: String,
    },
    Pause {
        label: String,
    },
    Resume {
        label: String,
    },
    TogglePause {
        label: String,
    },
}

pub struct ActiveScene {
    pub index: usize,
    pub paused: bool,
}

pub struct SceneManager {
    buffer: Vec<SceneCommand>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn register<L: Into<String>>(&mut self, label: L, scene: Box<dyn Scene>) {
        self.buffer.push(SceneCommand::Register {
            label: label.into(),
            scene,
        })
    }

    pub fn activate<L: Into<String>>(&mut self, label: L) {
        self.buffer.push(SceneCommand::Activate {
            label: label.into(),
            user_data: None,
        })
    }

    pub fn activate_with<L: Into<String>, D: Any>(&mut self, label: L, user_data: D) {
        self.buffer.push(SceneCommand::Activate {
            label: label.into(),
            user_data: Some(Box::new(user_data)),
        })
    }

    pub fn deactivate<L: Into<String>>(&mut self, label: L) {
        self.buffer.push(SceneCommand::Deactivate {
            label: label.into(),
        })
    }

    pub fn pause<L>(&mut self, label: L)
    where
        L: Into<String>,
    {
        self.buffer.push(SceneCommand::Pause {
            label: label.into(),
        });
    }

    pub fn resume<L>(&mut self, label: L)
    where
        L: Into<String>,
    {
        self.buffer.push(SceneCommand::Resume {
            label: label.into(),
        });
    }

    pub fn toggle_pause<L>(&mut self, label: L)
    where
        L: Into<String>,
    {
        self.buffer.push(SceneCommand::TogglePause {
            label: label.into(),
        })
    }

    pub(crate) fn drain_collect(&mut self) -> Vec<SceneCommand> {
        self.buffer.drain(..).collect()
    }
}
