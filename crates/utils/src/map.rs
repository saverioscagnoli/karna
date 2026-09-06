use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

pub type FastHasher = rustc_hash::FxHasher;
pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;
pub type FastHashSet<V> = HashSet<V, BuildHasherDefault<FastHasher>>;
