use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Batch {
    pub indices: Range<u32>,
    /// Which page of the atlas to bind to?
    /// If None, then use a white 1x1 texture
    pub page: Option<usize>,
}
