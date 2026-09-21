use core::ops::Index;
use core::ops::IndexMut;

use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use utils::fnv1a;

use crate::render::ImmediateVertex;

#[derive(Hash)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Layer(u64);

impl Layer {
    pub const WORLD: Self = Self::new_label("world");
    pub const UI: Self = Self::new_label("ui");
    pub const DEBUG: Self = Self::new_label("debug");

    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn new_label(label: &'static str) -> Self {
        Self(fnv1a(label.as_bytes()))
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::WORLD
    }
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct LayerData {
    pub vertices: Vec<ImmediateVertex>,
    pub indices: Vec<u32>,
}

pub struct LayerMap<T> {
    world: T,
    ui: T,
    debug: T,
    other: HashMap<Layer, T>,
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
            other: HashMap::default(),
            order: Vec::new(),
        }
    }

    pub fn get(&self, layer: Layer) -> Option<&T> {
        match layer {
            Layer::WORLD => Some(&self.world),
            Layer::UI => Some(&self.ui),
            Layer::DEBUG => Some(&self.debug),
            l => self.other.get(&l),
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

    /// Every layer's value, built-ins included, in no particular order.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.world, &mut self.ui, &mut self.debug]
            .into_iter()
            .chain(self.other.values_mut())
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
