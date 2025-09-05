/// A type that can be used as a DDS topic payload.
///
/// Implement this trait to register a type as a DDS topic payload. The derive
/// macro [`Topicable`](cyclonedds_macros::Topicable) handles the common case;
/// implement manually when you need control over the key type or type name.
///
/// # Keys
///
/// Every [`Topicable`] type has an associated [`Key`](Topicable::Key) type that
/// uniquely identifies an instance. For unkeyed topics, use any zero-sized type
/// as the key ([`()`](primitive@unit) being the straightforward choice), and
/// all samples will be treated as belonging to a single instance.
///
/// If using the [`derive`](cyclonedds_macros::Topicable) macro, a hidden type
/// gets generated which contains just the fields marked as a key fields. To
/// access this type see the [`Key` type alias](crate::topicable::Key).
///
/// # Examples
///
/// ```
/// #[derive(
///     cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Default, Clone, Debug,
/// )]
/// struct Temperature {
///     #[dds(key)]
///     sensor_id: u32,
///     value: f32,
/// }
/// ```
///
/// Manual implementation:
///
/// ```
/// #[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
/// struct Temperature {
///     sensor_id: u32,
///     value: f32,
/// }
///
/// impl cyclonedds::Topicable for Temperature {
///     type Key = u32;
///
///     fn from_key(key: &u32) -> Self {
///         Self {
///             sensor_id: *key,
///             value: 0.0,
///         }
///     }
///
///     fn as_key(&self) -> u32 {
///         self.sensor_id
///     }
/// }
/// ```
pub trait Topicable:
    serde::ser::Serialize + serde::de::DeserializeOwned + std::clone::Clone + std::fmt::Debug
{
    /// The key type that uniquely identifies an instance of this topic.
    ///
    /// For unkeyed topics, use any zero-sized type as the key
    /// ([`()`](primitive@unit) being the straightforward choice), and all
    /// samples will be treated as belonging to a single instance.
    ///
    /// The key type must implement [`CdrBounds`](crate::cdr_bounds::CdrBounds)
    /// to provide serialization size information required by DDS when
    /// computing keyhashes.
    type Key: serde::ser::Serialize
        + serde::de::DeserializeOwned
        + std::clone::Clone
        + std::fmt::Debug
        + std::cmp::PartialEq
        + std::hash::Hash
        + crate::cdr_bounds::CdrBounds;

    /// Whether this type has a meaningful key.
    ///
    /// `false` when `Self::Key` is `()`, in which case all samples belong to a
    /// single instance. Derived from the size of `Self::Key` at compile time.
    ///
    /// This can technically be overridden, but doing so will produce
    /// unintuitive behavior. The default derived value should be correct in
    /// virtually all cases.
    #[doc(hidden)]
    const IS_KEYED: bool = std::mem::size_of::<Self::Key>() != 0;

    /// Forces MD5 keyhash generation regardless of key size.
    ///
    /// By default, the big-endian CDR serialization of the key is used as the
    /// keyhash when the maximum serialized key size is 16 bytes or fewer, and
    /// MD5 otherwise. Set this to `true` to force MD5 unconditionally.
    const FORCE_MD5_KEYHASH: bool = false;

    /// Constructs a default instance of `Self` from a key.
    ///
    /// Used to materialize a full sample from a key-only notification. Fields
    /// not part of the key should be set to their default values.
    fn from_key(key: &Self::Key) -> Self;

    /// Extracts the key from this instance.
    fn as_key(&self) -> Self::Key;

    /// Returns the DDS type name for this type.
    ///
    /// Defaults to the Rust type name as it would appear within the crate.
    /// Override when interoperating with an existing system whose IDL type
    /// names differ from the Rust type names.
    #[must_use]
    fn dds_type_name() -> impl AsRef<str> {
        let full_type_path = std::any::type_name::<Self>();

        // Strip out the leading module if it exists or leave it as the full
        // type path otherwise.
        full_type_path
            .split_once("::")
            .map_or(full_type_path, |(_, type_path)| type_path)
    }
}

/// Evaluates to the [`Key`](Topicable::Key) type associated with the
/// [`Topicable`] type `T`.
///
/// # Examples
/// ```
/// use cyclonedds::{Key, Topicable};
///
/// #[derive(Topicable, serde::Serialize, serde::Deserialize, Default, Clone, Debug)]
/// struct Data {
///     #[dds(key)]
///     pub x: i32,
///     #[dds(key)]
///     pub y: i32,
///     pub message: String,
/// }
///
/// let data = Data {
///     x: 1,
///     y: 2,
///     ..Data::default()
/// };
///
/// // Access the `Key` of `Data` via the type alias.
/// let key = Key::<Data> { x: 1, y: 2 };
///
/// // The keys are equal.
/// assert_eq!(key, data.as_key());
/// ```
pub type Key<T> = <T as Topicable>::Key;
