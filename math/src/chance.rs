//! This module provides utilities for generating random numbers and making random choices.

use rand::distr::uniform::{SampleRange, SampleUniform};

/// Return a random value in the range `range`.
/// The type will be inferred from the range.
///
/// # Example
///
/// ```no_run
/// let x: f32 = rng(0.0..1.0); // -> 0.0 <= x < 1.0
/// ```
pub fn rng<T: SampleUniform, R: SampleRange<T>>(range: R) -> T {
    rand::random_range(range)
}

/// Return a bool with a probability `p` of being true.
///
/// The value of `p` must be between 0 and 100.
/// If `p` is not provided, it defaults to 50.
///
/// The name comes from the concept of 50/50 chance. (`coin_flip(None)` -> 50% chance of being true)
pub fn coin_flip(chance: u8) -> bool {
    rng(0..100) < chance
}

/// Pick a random element from a slice or a vector.
/// The element will be borrowed.
pub trait Pick<T> {
    /// Picks a random element from the container.
    /// Returns the index of the element and a reference to it.
    fn pick(&self) -> &T;
}

/// Pick a random element from a slice or a vector.
/// The element will be borrowed mutably.
pub trait PickMut<T> {
    /// Picks a random element from the container.
    fn pick_mut(&mut self) -> &mut T;
}

/// Implement pick for all elements that can be converted to a slice of T.
impl<T, C> Pick<T> for C
where
    C: AsRef<[T]>,
{
    fn pick(&self) -> &T {
        let slice = self.as_ref();
        let index = rng(0..slice.len());
        &slice[index]
    }
}

/// Implement pick_mut for all elements that can be converted to a slice of T.
impl<T, C> PickMut<T> for C
where
    C: AsMut<[T]>,
{
    fn pick_mut(&mut self) -> &mut T {
        let slice = self.as_mut();
        let index = rng(0..slice.len());
        &mut slice[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rng() {
        for _ in 0..100 {
            let x: f32 = rng(0.0..1.0);
            assert!(0.0 <= x && x < 1.0);
        }
    }

    #[test]
    fn test_coin_flip() {
        assert_ne!(coin_flip(0), coin_flip(100));
    }

    #[test]
    fn test_pick() {
        let list = [1, 2, 3, 4, 5];
        let picked = list.pick();

        assert!(*picked >= 1 && *picked <= 5);
    }

    #[test]
    fn test_pick_mut() {
        let mut list = [1, 2, 3, 4, 5];
        let picked = list.pick_mut();

        assert!(*picked >= 1 && *picked <= 5);
    }
}
