use std::hash::Hasher;

pub trait Hash32 {
    fn hash32(&self) -> u32;
}

impl<T: std::hash::Hash> Hash32 for T {
    fn hash32(&self) -> u32 {
        // Prepare the 32-bit hash by running the default hasher which produces a
        // 64-bit output and then combining the high and low ends of the hash via
        // xor to produce a 32-bit output.
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        // Do a 64-bit fold of both halves of the hash.
        //
        // NOTE: that the truncation to 32 bits is intentional and because of the
        // fold no entropy should be lost.
        #[allow(clippy::cast_possible_truncation)]
        let hash: u32 = (hash ^ (hash >> 32)) as u32;
        hash
    }
}
