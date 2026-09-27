macro_rules! sdl_enum {
    ($name:ident: $raw:ty { $($variant:ident => $sdl:ident),* $(,)? }) => {
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$(Self::$variant,)*];

            pub const fn from_raw(raw: $raw) -> Option<Self> {
                match raw {
                    $(::sdl3_sys::$sdl => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub const fn raw(self) -> $raw {
                match self {
                    $(Self::$variant => ::sdl3_sys::$sdl,)*
                }
            }
        }
    };
}
