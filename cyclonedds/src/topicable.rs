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
    serde::ser::Serialize
    + serde::de::DeserializeOwned
    + std::default::Default
    + std::clone::Clone
    + std::fmt::Debug
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
        + std::default::Default
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

/// A type whose in-memory representation may be shared verbatim with another
/// process.
///
/// This is the building block for zero-copy transfer. It composes: a struct is
/// [`FixedSize`] when it is `#[repr(C)]` and every one of its fields is, which is
/// what [`derive(FixedSize)`](cyclonedds_macros::FixedSize) checks. Deriving it is
/// the only way to get it without writing `unsafe`.
///
/// A [`Topicable`] that is also [`FixedSize`] is automatically a
/// [`FixedSizeTopic`], which is what
/// [`ZeroCopy`](crate::representation::ZeroCopy) representation
/// requires.
///
/// # What is implemented
///
/// The integer and floating-point types that IDL maps to — [`u8`], [`u16`],
/// [`u32`], [`u64`], [`i8`], [`i16`], [`i32`], [`i64`], [`f32`], [`f64`] — and
/// arrays of them.
///
/// Deliberately **not** implemented, because these have bit patterns that are not
/// valid values and the bytes may come from another process which this binding
/// cannot police:
///
/// - [`bool`], where anything other than 0 or 1 is invalid. Use [`u8`].
/// - [`char`], which is a Unicode scalar value, not IDL's one-byte `char`. Use [`u8`].
/// - enums, including [`Option`] and [`Result`], where only listed discriminants are valid.
/// - [`u128`] and [`i128`], which have no IDL equivalent and are 16-byte aligned.
/// - [`usize`] and [`isize`], whose width differs between peers.
///
/// # Safety
///
/// Cyclone DDS will `memcpy` `size_of::<Self>()` bytes of a value into memory
/// that other processes read, and will present memory written by other processes
/// to be read as a `Self`. Implementing this by hand asserts all of the
/// following.
///
/// - `Self` is `#[repr(C)]`, so its layout is stable and matches what a C compiler would
///   produce for the equivalent IDL type. A `repr(Rust)` type has no layout guarantee at all.
/// - `Self` contains no pointers, references, or indirection of any kind. The [`Copy`] supertrait
///   rules out `String`, `Vec` and `Box` for you, but not a raw pointer field.
/// - Every bit pattern of `size_of::<Self>()` bytes is a valid `Self`, including all zeroes.
/// - `Self` has no padding bytes. Padding is not an unsoundness in itself, but it would be sent
///   uninitialized, so it leaks whatever happened to be in those bytes.
/// - `align_of::<Self>()` is at most 8, since a PSMX implementation is only required to align
///   loaned memory for the types IDL maps to.
///
/// Prefer the derive, which checks every one of those except the first half of
/// the second (raw pointers) mechanically.
///
/// # Examples
///
/// ```
/// use cyclonedds::FixedSize;
///
/// #[derive(FixedSize, Clone, Copy)]
/// #[repr(C)]
/// struct Position {
///     x: f64,
///     y: f64,
/// }
///
/// // Composes: a `repr(C)` struct of `FixedSize` fields is itself `FixedSize`.
/// #[derive(FixedSize, Clone, Copy)]
/// #[repr(C)]
/// struct Pose {
///     position: Position,
///     heading: f64,
///     tag: [u8; 8],
/// }
/// ```
///
/// # Rejected at compile time
///
/// Padding, because those bytes would travel uninitialized. A `u8` followed by a
/// `u64` leaves a seven byte hole:
///
/// ```compile_fail
/// use cyclonedds::FixedSize;
///
/// #[derive(FixedSize, Clone, Copy)]
/// #[repr(C)]
/// struct Padded {
///     small: u8,
///     large: u64,
/// }
/// ```
///
/// A field whose type is not itself shareable — here a [`bool`], for which most
/// bit patterns are invalid:
///
/// ```compile_fail
/// use cyclonedds::FixedSize;
///
/// #[derive(FixedSize, Clone, Copy)]
/// #[repr(C)]
/// struct Flagged {
///     flag: bool,
/// }
/// ```
///
/// And an over-aligned type, which a PSMX implementation need not align a loan
/// for:
///
/// ```compile_fail
/// use cyclonedds::FixedSize;
///
/// #[derive(FixedSize, Clone, Copy)]
/// #[repr(C, align(16))]
/// struct OverAligned {
///     value: u64,
/// }
/// ```
#[allow(
    unsafe_code,
    reason = "implementors promise a layout contract the compiler cannot fully check"
)]
pub unsafe trait FixedSize: Copy {}

/// Implement [`FixedSize`] for types whose every bit pattern is a valid value and
/// whose layout is fixed by the platform ABI.
macro_rules! fixed_size {
    ($($type:ty),* $(,)?) => {
        $(
            #[allow(
                unsafe_code,
                reason = "a primitive with no invalid bit patterns and a stable ABI layout"
            )]
            // SAFETY: an integer or floating-point primitive is `repr`-stable, holds
            // no indirection, has no padding, is valid for every bit pattern, and is
            // aligned to at most its own size, which here is at most 8.
            unsafe impl FixedSize for $type {}
        )*
    };
}

fixed_size!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[allow(
    unsafe_code,
    reason = "an array adds no layout of its own beyond that of its element type"
)]
// SAFETY: `[T; N]` is `N` contiguous `T` with no padding between elements and none
// at the end, and it inherits `T`'s alignment. So it holds whenever it holds for
// `T`.
unsafe impl<T, const N: usize> FixedSize for [T; N] where T: FixedSize {}

/// A [`Topicable`] whose in-memory representation may be shared verbatim with
/// other processes, enabling zero-copy transfer.
///
/// You never implement this, and it is safe precisely because of that: it holds
/// automatically for any [`Topicable`] that is also [`FixedSize`], and the blanket
/// implementation means it cannot be asserted any other way. Every guarantee it
/// relies on belongs to [`FixedSize`], which is the unsafe half. Put
/// [`derive(FixedSize)`](cyclonedds_macros::FixedSize) on the type and this
/// follows.
///
/// Implementing it changes nothing on its own. Opt a topic in with
/// [`ZeroCopy`](crate::representation::ZeroCopy) representation,
/// then borrow buffers with [`Writer::loan`](crate::Writer::loan).
///
/// # Examples
///
/// ```
/// use cyclonedds::{FixedSize, Topicable};
///
/// #[derive(Topicable, FixedSize, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default)]
/// #[repr(C)]
/// struct Reading {
///     #[dds(key)]
///     sensor_id: u32,
///     sequence: u32,
///     value: f64,
/// }
///
/// fn assert_zero_copy_capable<T: cyclonedds::FixedSizeTopic>() {}
/// assert_zero_copy_capable::<Reading>();
/// ```
pub trait FixedSizeTopic: Topicable + FixedSize {}

impl<T> FixedSizeTopic for T where T: Topicable + FixedSize {}

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
