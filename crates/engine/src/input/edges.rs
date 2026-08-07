use utils::BitSet;

#[derive(Clone, Copy)]
pub struct Edges<S> {
    down: S,
    press_frame: S,
    press_tick: S,
    rel_frame: S,
    rel_tick: S,
}

impl<S: Default> Default for Edges<S> {
    fn default() -> Self {
        Self {
            down: S::default(),
            press_frame: S::default(),
            press_tick: S::default(),
            rel_frame: S::default(),
            rel_tick: S::default(),
        }
    }
}

impl<S: BitSet> Edges<S> {
    pub fn press(&mut self, t: S::Item) {
        self.down.insert(t);
        self.press_frame.insert(t);
        self.press_tick.insert(t);
    }

    pub fn release(&mut self, t: S::Item) {
        self.down.remove(t);
        self.rel_frame.insert(t);
        self.rel_tick.insert(t);
    }

    pub fn held(&self, t: S::Item) -> bool {
        self.down.contains(t)
    }

    pub fn just_pressed_frame(&self, t: S::Item) -> bool {
        self.press_frame.contains(t)
    }

    pub fn just_pressed_tick(&self, t: S::Item) -> bool {
        self.press_tick.contains(t)
    }

    pub fn just_released_frame(&self, t: S::Item) -> bool {
        self.rel_frame.contains(t)
    }

    pub fn just_released_tick(&self, t: S::Item) -> bool {
        self.rel_tick.contains(t)
    }

    pub fn roll_frame(&mut self) {
        self.press_frame.clear();
        self.rel_frame.clear();
    }

    pub fn roll_tick(&mut self) {
        self.press_tick.clear();
        self.rel_tick.clear();
    }

    pub fn clear_all(&mut self) {
        self.down.clear();
        self.press_frame.clear();
        self.press_tick.clear();
        self.rel_frame.clear();
        self.rel_tick.clear();
    }
}
