use sdl3_sys::SDL_GetPerformanceCounter;
use sdl3_sys::SDL_rand_bits_r;
use sdl3_sys::SDL_rand_r;
use sdl3_sys::SDL_randf_r;

#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl Rng {
    pub fn new() -> Self {
        Self {
            state: unsafe { SDL_GetPerformanceCounter() },
        }
    }

    pub fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline]
    pub fn state(&self) -> u64 {
        self.state
    }

    #[inline]
    pub fn set_state(&mut self, state: u64) {
        self.state = state;
    }

    #[inline]
    fn splitmix64(mut z: u64) -> u64 {
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn fork(&mut self) -> Rng {
        let hi = unsafe { SDL_rand_bits_r(&mut self.state) } as u64;
        let lo = unsafe { SDL_rand_bits_r(&mut self.state) } as u64;
        Rng::seeded(Self::splitmix64((hi << 32) | lo))
    }

    /// [0, n)
    #[inline]
    pub fn below(&mut self, n: u32) -> u32 {
        debug_assert!(n > 0 && n <= i32::MAX as u32);
        unsafe { SDL_rand_r(&mut self.state, n as i32) as u32 }
    }

    /// [low, high]
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        debug_assert!(high >= low);
        let span = (high as i64 - low as i64 + 1) as u32; // fits for spans <= i32::MAX
        (low as i64 + self.below(span) as i64) as i32
    }

    /// [0.0, 1.0)
    #[inline]
    pub fn float(&mut self) -> f32 {
        unsafe { SDL_randf_r(&mut self.state) }
    }

    /// approximately [low, high)
    #[inline]
    pub fn float_range(&mut self, low: f32, high: f32) -> f32 {
        debug_assert!(high >= low);
        low + self.float() * (high - low)
    }

    #[inline]
    pub fn chance(&mut self, p: f32) -> bool {
        self.float() < p
    }

    #[inline]
    pub fn coin_flip(&mut self) -> bool {
        self.below(2) == 0
    }

    /// [1, sides]
    #[inline]
    pub fn die(&mut self, sides: u32) -> u32 {
        self.below(sides) + 1
    }

    #[inline]
    pub fn dice(&mut self, count: u32, sides: u32) -> u32 {
        debug_assert!(count.checked_mul(sides).is_some());
        (0..count).map(|_| self.die(sides)).sum()
    }

    #[inline]
    pub fn d4(&mut self) -> u32 {
        self.die(4)
    }

    #[inline]
    pub fn d6(&mut self) -> u32 {
        self.die(6)
    }

    #[inline]
    pub fn d8(&mut self) -> u32 {
        self.die(8)
    }

    #[inline]
    pub fn d10(&mut self) -> u32 {
        self.die(10)
    }

    #[inline]
    pub fn d12(&mut self) -> u32 {
        self.die(12)
    }

    #[inline]
    pub fn d20(&mut self) -> u32 {
        self.die(20)
    }

    #[inline]
    pub fn d100(&mut self) -> u32 {
        self.die(100)
    }

    #[inline]
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        debug_assert!(!items.is_empty());
        &items[self.below(items.len() as u32) as usize]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            items.swap(i, j);
        }
    }

    pub fn weighted_index(&mut self, weights: &[u32]) -> usize {
        let total = weights.iter().sum::<u32>();
        debug_assert!(total > 0);

        let mut roll = self.below(total);

        for (i, &w) in weights.iter().enumerate() {
            if roll < w {
                return i;
            }

            roll -= w;
        }

        unreachable!()
    }
}
