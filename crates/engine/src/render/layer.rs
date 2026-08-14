use std::ops::Index;
use std::ops::IndexMut;

use utils::FastHashMap;
use utils::fnv1a;

use crate::Camera;
use crate::render::geometry::ImmediateGeometry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer(u64);

impl Layer {
    pub const WORLD: Self = Self::new_label("world");
    pub const UI: Self = Self::new_label("ui");
    pub const DEBUG: Self = Self::new_label("debug");

    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_label(label: &'static str) -> Self {
        Self(fnv1a(label.as_bytes()))
    }
}

pub enum LayerKind {
    Mesh {
        // Mesh slotmap will go here...
        immediate: ImmediateGeometry,
    },
    Content {
        immediate: ImmediateGeometry,
    },
}

impl LayerKind {
    #[inline]
    pub fn immediate(&self) -> &ImmediateGeometry {
        match self {
            Self::Mesh { immediate } | Self::Content { immediate } => immediate,
        }
    }

    #[inline]
    pub fn immediate_mut(&mut self) -> &mut ImmediateGeometry {
        match self {
            Self::Mesh { immediate } | Self::Content { immediate } => immediate,
        }
    }
}

pub struct LayerMap<T> {
    world: T,
    ui: T,
    debug: T,
    other: FastHashMap<Layer, T>,
    order: Vec<Layer>,
}

impl<T> LayerMap<T> {
    pub fn new(world: T, ui: T, debug: T) -> Self {
        Self {
            world,
            ui,
            debug,
            other: FastHashMap::default(),
            order: Vec::new(),
        }
    }
}

impl<T> Index<Layer> for LayerMap<T> {
    type Output = T;

    fn index(&self, layer: Layer) -> &Self::Output {
        match layer {
            Layer::WORLD => &self.world,
            Layer::UI => &self.ui,
            Layer::DEBUG => &self.debug,
            l => self.other.get(&l).expect("Failed to get layer"),
        }
    }
}

impl<T> IndexMut<Layer> for LayerMap<T> {
    fn index_mut(&mut self, layer: Layer) -> &mut Self::Output {
        match layer {
            Layer::WORLD => &mut self.world,
            Layer::UI => &mut self.ui,
            Layer::DEBUG => &mut self.debug,
            l => self.other.get_mut(&l).expect("Failed to get layer"),
        }
    }
}
