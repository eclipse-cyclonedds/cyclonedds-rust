//! Common utilities for integration tests.

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

    /// A minimal [`cyclonedds::Topicable`] type used as a test topic payload.
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

    impl cyclonedds::Topicable for Data {
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
    #[must_use]
    pub fn unique_name() -> String {
        let uuid = Uuid::new_v4();
        uuid.to_string()
    }
}
