pub trait BitSet: Default + Copy {
    type Item: Copy;

    fn insert(&mut self, item: Self::Item);
    fn remove(&mut self, item: Self::Item);
    fn contains(&self, item: Self::Item) -> bool;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
}
