//! Common utilities for unit tests.

pub mod domain {
    //! Utilities for creating domains in tests.

    /// Atomic counter used to generate unique domain IDs across a test thread.
    static UNIQUE_DOMAIN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    /// Upper bound on a valid domain ID.
    ///
    /// NOTE: this was determined by incrementing it until it started failing
    /// and so this value is not necessarily stable (though for all practical
    /// purposes it probably is).
    const MAX_DOMAIN_BOUNDS: u32 = 233;

    /// Returns a unique domain ID (unique across a thread).
    ///
    /// The IDs are incremented sequentially and are wrapped within the
    /// [0..[`MAX_DOMAIN_BOUNDS`]].
    pub fn unique_id() -> u32 {
        let counter = UNIQUE_DOMAIN.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        counter % MAX_DOMAIN_BOUNDS
    }
}

pub mod topic {
    //! Utilities for creating topics in tests.

    use uuid::Uuid;

    /// A topic type laid out so Cyclone DDS may move it as raw memory.
    ///
    /// `#[repr(C)]`, 16 bytes with no padding, valid for every bit pattern, 8-byte
    /// aligned.
    #[derive(
        serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash,
    )]
    #[repr(C)]
    pub struct RawData {
        pub id: u32,
        pub sequence: u32,
        pub payload: [u8; 8],
    }

    // NOTE: hand-written rather than derived because `derive(FixedSize)` emits
    // `::cyclonedds::` paths, which do not resolve from within this crate. The derive
    // itself is exercised from `tests/psmx.rs`, which is a separate crate.
    //
    // SAFETY: `repr(C)`, only integers and a byte array, so no padding, no
    // indirection, valid for every bit pattern, and 4-byte aligned.
    #[allow(
        unsafe_code,
        reason = "the memcpy-safe contract cannot be validated by the compiler"
    )]
    unsafe impl crate::topicable::FixedSize for RawData {}

    impl crate::Topicable for RawData {
        type Key = u32;

        fn from_key(key: &Self::Key) -> Self {
            Self {
                id: *key,
                ..Default::default()
            }
        }

        fn as_key(&self) -> Self::Key {
            self.id
        }
    }

    /// A minimal [`crate::Topicable`] type used as a test topic payload.
    ///
    /// The composite key `(x, y)` uniquely identifies each instance.
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default)]
    pub struct Data {
        /// First component of the key.
        pub x: u32,
        /// Second component of the key.
        pub y: i32,
        /// Arbitrary string payload.
        pub message: String,
    }

    impl crate::Topicable for Data {
        type Key = (u32, i32);

        fn from_key(key: &Self::Key) -> Self {
            Self {
                x: key.0,
                y: key.1,
                ..Default::default()
            }
        }

        fn as_key(&self) -> Self::Key {
            (self.x, self.y)
        }
    }

    /// Returns a unique topic name backed by a random UUID v4 (to prevent topic
    /// collisions between concurrently running tests).
    pub fn unique_name() -> String {
        let uuid = Uuid::new_v4();
        uuid.to_string()
    }
}
