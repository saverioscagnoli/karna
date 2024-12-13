use rand::distributions::uniform::SampleUniform;
use rand::Rng;
use std::ops::RangeBounds;

/// Generate a random number between min and max.
/// The number type is automatically inferred.
/// So, for example, if you call `let i = rng(0, 10); arr[i]`, the return type will be `usize`.
/// If you call `rng(0.0, 10.0)`, the return type will be `f32`.
///
/// # Examples
///
/// ```no_run
/// use karna::math::rng;
///
/// let i = rng(0..=10);
/// let f = rng(0.0..=10.0);
/// ```

pub fn rng<T, R>(range: R) -> T
where
    T: SampleUniform + Copy + PartialOrd,
    R: RangeBounds<T>,
{
    let mut rand = rand::thread_rng();

    match (range.start_bound(), range.end_bound()) {
        (std::ops::Bound::Included(&start), std::ops::Bound::Included(&end)) => {
            rand.gen_range(start..=end)
        }
        (std::ops::Bound::Included(&start), std::ops::Bound::Excluded(&end)) => {
            rand.gen_range(start..end)
        }
        _ => panic!("Unsupported range bounds"),
    }
}

/// Flip a coin with a certain chance of success.
pub fn coin_flip(chance: u32) -> bool {
    rng(0..=100) < chance
}

/// Pick a random item from a slice.
/// The return type is a reference to the item.
///
/// # Examples
///
/// ```no_run
/// use karna::math::pick;
///
/// let foods = vec!["apple", "banana", "carrot"];
/// let food = pick(&foods);
///
/// println!("You should eat a {}", food);
/// ```
pub fn pick<T>(items: &[T]) -> &T {
    &items[rng(0..items.len())]
}

/// Pick a random mutable item from a slice.
/// The return type is a mutable reference to the item.
///
/// # Examples
///
/// ```no_run
/// use karna::math::pick_mut;
///
/// let mut foods = vec!["apple", "banana", "carrot"];
/// let food = pick_mut(&mut foods);
///
/// println!("You should eat a {}", food);
///
/// *food = "donut";
///
/// println!("Food changed to {}", food);
pub fn pick_mut<T>(items: &mut [T]) -> &mut T {
    &mut items[rng(0..items.len())]
}
