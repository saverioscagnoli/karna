pub use rand::random_range as rng;

#[inline]
pub fn pick<T>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        None
    } else {
        Some(&items[rng(0..items.len())])
    }
}

#[inline]
pub fn pick_mut<T>(items: &mut [T]) -> Option<&mut T> {
    if items.is_empty() {
        None
    } else {
        Some(&mut items[rng(0..items.len())])
    }
}

#[inline]
pub fn flip(chance: u8) -> bool {
    rng(0..100) < chance
}

#[inline]
pub fn flip_coin() -> bool {
    flip(50)
}
