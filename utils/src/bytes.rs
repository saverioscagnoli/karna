use std::mem;

pub fn hash_bytes(data: &[u8]) -> u64 {
    use std::hash::Hash;
    use std::hash::Hasher;

    let mut h = std::collections::hash_map::DefaultHasher::new();

    data.hash(&mut h);
    h.finish()
}

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe { ::core::slice::from_raw_parts(slice.as_ptr() as *const u8, mem::size_of_val(slice)) }
}
