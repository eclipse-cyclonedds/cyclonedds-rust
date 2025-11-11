use md5::Digest;

use crate::cdr_bounds::{CdrBounds, CdrSize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyHash(pub(crate) [u8; 16]);

impl KeyHash {
    pub fn from_key<T>(key: &T::Key, force_md5: bool) -> Option<KeyHash>
    where
        T: crate::Topicable,
    {
        cdr_encoding::to_vec::<_, byteorder::BigEndian>(&key)
            .ok()
            .and_then(|mut serialized| {
                let max_possible_serialized_size = T::Key::max_serialized_cdr_size();

                let key_hash = if force_md5 || max_possible_serialized_size > CdrSize::Bounded(16) {
                    // The key hash should be computed via MD5.
                    let mut hasher = md5::Md5::new();
                    hasher.update(serialized);
                    let hash = hasher.finalize();
                    hash.into()
                } else {
                    // The CDR serialized form fits and can be used as the key hash but
                    // it must be padded to 16 bytes and those padding bytes must be zeroed.
                    serialized.resize(16.max(serialized.len()), 0);
                    // TODO could this possibly fail?
                    serialized.try_into().ok()?
                };

                Some(KeyHash(key_hash))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Topicable;

    #[test]
    fn test_keyhash() {
        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq)]
        struct Data {
            x: i32,
        }

        #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Hash, Default)]
        struct DataKey {
            x: Vec<i32>,
        }

        static MOCKED_MAX_SERIALIZED_CDR_SIZE: std::sync::Mutex<usize> = std::sync::Mutex::new(0);

        impl CdrBounds for DataKey {
            fn max_serialized_cdr_size() -> CdrSize {
                let size = *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap();
                CdrSize::Bounded(size)
            }

            fn alignment() -> usize {
                8
            }
        }

        impl crate::Topicable for Data {
            type Key = DataKey;

            fn from_key(key: &Self::Key) -> Self {
                Self {
                    x: key.x.iter().copied().next().unwrap_or_default(),
                }
            }

            fn as_key(&self) -> Self::Key {
                DataKey { x: vec![self.x] }
            }
        }

        let sample = Data { x: 11 };
        let key = sample.as_key();

        // Deliberately miscalculated.
        *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = DataKey::alignment();

        // Check that the computed keyhash from the vec is the Big Endian CDR encoded form of the
        // vec.
        let cdr_key_hash = KeyHash::from_key::<Data>(&key, false).unwrap();

        let deserialized_key = cdr_encoding::from_bytes::<_, byteorder::BigEndian>(&cdr_key_hash.0)
            .unwrap()
            .0;
        assert_eq!(key, deserialized_key);
        assert_eq!(sample, Data::from_key(&deserialized_key));

        let key = DataKey { x: vec![0; 32] };
        // Check that since the serialized form would be over the 16-byte bound
        // the result was None.
        let cdr_key_hash = KeyHash::from_key::<Data>(&key, false);
        assert_eq!(cdr_key_hash, None);

        // Check that even with the invalid serialization limit the keyhash under md5 still
        // succeeds.
        let md5_key_hash_01 = KeyHash::from_key::<Data>(&key, true).unwrap();

        *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = 4 * 32 + 4;

        // Check that with a serialization limit over the bound the keyhash
        // without forced md5 still succeeds.
        let md5_key_hash_02 = KeyHash::from_key::<Data>(&key, false).unwrap();

        // Check that the results of both of these are the same md5 keyhash.
        assert_eq!(md5_key_hash_01, md5_key_hash_02);
    }
}
