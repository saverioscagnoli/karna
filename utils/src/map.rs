use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;

pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<rustc_hash::FxHasher>>;
pub type FastHashSet<V> = HashSet<V, BuildHasherDefault<rustc_hash::FxHasher>>;
