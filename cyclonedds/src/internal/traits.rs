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

pub trait CdrHeader {
    fn cdr_header() -> [u8; 4];
}

impl CdrHeader for byteorder::LittleEndian {
    fn cdr_header() -> [u8; 4] {
        [0x00, 0x01, 0x00, 0x00]
    }
}

impl CdrHeader for byteorder::BigEndian {
    fn cdr_header() -> [u8; 4] {
        [0x00, 0x00, 0x00, 0x00]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdr_header_configurations() {
        // NOTE: this is a trivial test for now, but will be expanded as the
        // additional CDR options are added.
        let expected = [0x00, 0x01, 0x00, 0x00];
        let actual = byteorder::LittleEndian::cdr_header();
        assert_eq!(expected, actual);

        let expected = [0x00, 0x00, 0x00, 0x00];
        let actual = byteorder::BigEndian::cdr_header();
        assert_eq!(expected, actual);
    }
}
