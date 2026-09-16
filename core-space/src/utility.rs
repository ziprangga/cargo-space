pub use rustc_hash::FxHashMap as HashMap;
pub use rustc_hash::FxHashSet as HashSet;

pub type IndexMap<K, V> = indexmap::IndexMap<K, V, rustc_hash::FxBuildHasher>;
pub type IndexSet<V> = indexmap::IndexSet<V, rustc_hash::FxBuildHasher>;

pub fn index_map<K, V>() -> IndexMap<K, V> {
    IndexMap::with_hasher(rustc_hash::FxBuildHasher::default())
}
