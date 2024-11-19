#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LoopFlag {
    Accelerated,
    VSync,
}
