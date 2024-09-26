use rand::{distributions::uniform::SampleUniform, Rng};

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
/// let i = rng(0, 10);
/// let f = rng(0.0, 10.0);
/// ```
pub fn rng<T>(min: T, max: T) -> T
where
    T: SampleUniform + Copy + PartialOrd,
{
    let mut rand = rand::thread_rng();
    rand.gen_range(min..max)
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
    let index = rng(0, items.len());
    &items[index]
}
