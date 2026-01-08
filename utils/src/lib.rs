mod lazy;
mod macros;
mod storage;
mod structs;
mod timer;

// === RE-EXPORTS ===
pub use lazy::*;
pub use storage::*;
pub use structs::*;
pub use timer::*;

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * ::core::mem::size_of::<T>(),
        )
    }
}
