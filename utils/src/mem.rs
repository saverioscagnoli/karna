use std::mem;

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe { ::core::slice::from_raw_parts(slice.as_ptr() as *const u8, mem::size_of_val(slice)) }
}
