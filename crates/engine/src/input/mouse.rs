use utils::BitSet;

macro_rules! buttons {
    ($($name:ident => $btn:ident),* $(,)?) => {
        #[repr(u8)]
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum MouseButton {
            $($name = sdl3::$btn as u8,)*
        }

        impl MouseButton {
            /// Map a raw SDL button index to a `MouseButton`.
            ///
            /// Returns `None` for buttons this engine does not model.
            pub fn from_index(raw: u8) -> Option<Self> {
                match raw as u32 {
                    $(sdl3::$btn => Some(Self::$name),)*
                    _ => None,
                }
            }

            /// Every button this engine models, in index order.
            pub const ALL: &'static [MouseButton] = &[$(Self::$name,)*];
        }
    };
}

buttons! {
    Left   => SDL_BUTTON_LEFT,
    Middle => SDL_BUTTON_MIDDLE,
    Right  => SDL_BUTTON_RIGHT,
    X1     => SDL_BUTTON_X1,
    X2     => SDL_BUTTON_X2,
}

impl MouseButton {
    pub fn to_index(self) -> u8 {
        self as u8
    }

    const fn mask(self) -> u8 {
        1u8 << (self as u8 - 1)
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseSet(u8);

impl BitSet for MouseSet {
    type Item = MouseButton;

    fn insert(&mut self, btn: MouseButton) {
        self.0 |= btn.mask();
    }

    fn remove(&mut self, btn: MouseButton) {
        self.0 &= !btn.mask();
    }

    fn contains(&self, btn: MouseButton) -> bool {
        self.0 & btn.mask() != 0
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}
