use std::ops::Index;
use std::ops::IndexMut;

use utils::FastHashMap;
use utils::fnv1a;

use crate::render::geometry::ImmediateGeometry;

#[derive(Default)]
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

pub enum LayerData {
    ThreeDimensional {
        // Mesh slotmap will go here
        immediate: ImmediateGeometry,
    },
    TwoDimensional {
        immediate: ImmediateGeometry,
    },
}

impl LayerData {
    pub fn immediate(&self) -> &ImmediateGeometry {
        match self {
            Self::ThreeDimensional { immediate } | Self::TwoDimensional { immediate } => immediate,
        }
    }

    pub fn immediate_mut(&mut self) -> &mut ImmediateGeometry {
        match self {
            Self::ThreeDimensional { immediate } | Self::TwoDimensional { immediate } => immediate,
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

impl<T> Index<Layer> for LayerMap<T> {
    type Output = T;

    fn index(&self, layer: Layer) -> &Self::Output {
        match layer {
            Layer::WORLD => &self.world,
            Layer::UI => &self.ui,
            Layer::DEBUG => &self.debug,
            l => self.other.get(&l).expect("Failed to index layermap"),
        }
    }
}

impl<T> IndexMut<Layer> for LayerMap<T> {
    fn index_mut(&mut self, layer: Layer) -> &mut Self::Output {
        match layer {
            Layer::WORLD => &mut self.world,
            Layer::UI => &mut self.ui,
            Layer::DEBUG => &mut self.debug,
            l => self.other.get_mut(&l).expect("Failed to index layermap"),
        }
    }
}

impl<T> LayerMap<T> {
    pub fn new(world: T, ui: T, debug: T) -> Self {
        Self {
            world,
            ui,
            debug,
            other: FastHashMap::default(),
            order: vec![Layer::WORLD, Layer::UI, Layer::DEBUG],
        }
    }

    pub fn contains(&self, layer: Layer) -> bool {
        matches!(layer, Layer::WORLD | Layer::UI | Layer::DEBUG) || self.other.contains_key(&layer)
    }

    pub fn insert(&mut self, layer: Layer, value: T) {
        if !self.contains(layer) {
            self.order.push(layer);
        }

        self.other.insert(layer, value);
    }

    pub fn order(&self) -> &[Layer] {
        &self.order
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn layer_at(&self, index: usize) -> Layer {
        self.order[index]
    }
}
