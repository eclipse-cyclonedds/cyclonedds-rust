//! Operations that allow Cyclone to interact with Rust allocated data
//! structures. These are threaded through the [`Serdata`] type.

use std::io::Write;

use crate::cdr_bounds::{CdrBounds, CdrSize};
use crate::internal::ffi::InternalSample;
use crate::internal::key_hash::KeyHash;
use crate::internal::serdata::Serdata;
use crate::internal::sertype::Sertype;
use crate::sample::SampleOrKeyInner as SampleOrKey;

/// The size of the RTPS header in bytes.
pub const DDSI_RTPS_HEADER_SIZE: usize = 4;

/// This exists to ensure that initializing `cyclonedds_sys::ddsi_serdata_ops`
/// will work in const contexts even if more fields are added down the line to
/// Cyclone DDS.
pub(crate) const fn zeroed_serdata_ops() -> cyclonedds_sys::ddsi_serdata_ops {
    // SAFETY: `cyclonedds_sys::ddsi_serdata_ops` is repr(C) and consists solely of
    // function pointers and various integer types.
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
}

/// Compares the keys of two [`Serdata`] instances for equality.
///
/// ## Safety
/// The `lhs` and `rhs` must be non-null pointers to fully-initialized
/// [`Serdata`].
pub unsafe extern "C" fn eqkey<T>(
    lhs: *const cyclonedds_sys::ddsi_serdata,
    rhs: *const cyclonedds_sys::ddsi_serdata,
) -> bool
where
    T: crate::Topicable,
{
    let lhs = unsafe { &mut *(lhs as *mut Serdata<T>) };
    let rhs = unsafe { &mut *(rhs as *mut Serdata<T>) };

    lhs.key() == rhs.key()
}

/// Returns the serialized size (in bytes) of the sample contained in the given
/// `Serdata` + the size of the DDSI RTPS header.
///
/// ## Safety
/// The `serdata`  must be a non-null pointer to a fully-initialized
/// [`Serdata`].
pub unsafe extern "C" fn get_size<T>(serdata: *const cyclonedds_sys::ddsi_serdata) -> u32
where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };

    u32::try_from(
        serdata
            .serialized()
            .expect("unable to serialize data")
            .len(),
    )
    .expect("serialized data out of bounds")
}

fn deserialize_boxed<'de, T, BO>(bytes: &'de [u8]) -> Result<Box<T>, cdr_encoding::Error>
where
    T: serde::Deserialize<'de> + Default,
    BO: byteorder::ByteOrder,
{
    // NOTE: `extern "C"` tells the caller that this function cannot unwind,
    // even when `T::default()` is compiled in another codegen unit. Keeping
    // the call out of line lets the optimizer use the box as the return slot.
    #[inline(never)]
    extern "C" fn default_value<T: Default>() -> T {
        T::default()
    }

    let mut value = Box::write(Box::new_uninit(), default_value::<T>());
    let mut decoder = cdr_encoding::CdrDeserializer::<BO>::new(bytes);
    T::deserialize_in_place(&mut decoder, &mut value)?;
    Ok(value)
}

pub(crate) fn from_ser_buffer<T>(
    sertype: &crate::internal::sertype::Sertype<T>,
    kind: crate::internal::serdata::Kind,
    buffer: &[u8],
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    fn deserialize<T: crate::Topicable, BO: byteorder::ByteOrder>(
        kind: crate::internal::serdata::Kind,
        bytes: &[u8],
    ) -> Result<SampleOrKey<T>, cdr_encoding::Error> {
        match kind {
            crate::internal::serdata::Kind::Key => {
                deserialize_boxed::<T::Key, BO>(bytes).map(SampleOrKey::from_boxed_key)
            }
            crate::internal::serdata::Kind::Data => {
                deserialize_boxed::<T, BO>(bytes).map(SampleOrKey::from_boxed_sample)
            }
        }
    }

    let Some((header, bytes)) = buffer.split_at_checked(DDSI_RTPS_HEADER_SIZE) else {
        return std::ptr::null_mut();
    };
    let sample = match header {
        [0x0, 0x0, ..] => deserialize::<T, byteorder::BigEndian>(kind, bytes),
        [0x0, 0x1, ..] => deserialize::<T, byteorder::LittleEndian>(kind, bytes),
        _ => return std::ptr::null_mut(),
    };
    sample.map_or(std::ptr::null_mut(), |sample| {
        Box::into_raw(Box::new(Serdata::new(sertype, sample))).cast()
    })
}

fn copy_from_fragment(fragment_chain: &cyclonedds_sys::ddsi_rdata, size: usize) -> Option<Vec<u8>> {
    if fragment_chain.min != 0 {
        return None;
    }

    let mut buffer = vec![0; size];
    let mut offset = 0;

    let mut fragment = fragment_chain;
    loop {
        let min = fragment.min as usize;
        let maxp1 = fragment.maxp1 as usize;

        if offset < min {
            return None;
        }

        if maxp1 > offset {
            let number_of_bytes = maxp1 - offset;

            let buffer = buffer.get_mut(offset..offset + number_of_bytes)?;

            let src = unsafe {
                std::slice::from_raw_parts(
                    (fragment.rmsg as *const u8)
                        .add(std::mem::size_of::<cyclonedds_sys::ddsi_rmsg>())
                        .add(fragment.payload_zoff as usize)
                        .add(offset - min),
                    number_of_bytes,
                )
            };

            buffer.copy_from_slice(src);

            offset = maxp1;
        }

        if fragment.nextfrag.is_null() {
            break;
        }

        fragment = unsafe { &*fragment.nextfrag }
    }

    if offset == size { Some(buffer) } else { None }
}

///
/// ## Safety
pub unsafe extern "C" fn from_ser<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    kind: cyclonedds_sys::ddsi_serdata_kind,
    fragment_chain: *const cyclonedds_sys::ddsi_rdata,
    size: usize,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

    if fragment_chain.is_null() {
        return std::ptr::null_mut();
    }

    crate::internal::serdata::Kind::try_from(kind).map_or(std::ptr::null_mut(), |kind| {
        let fragment_chain = unsafe { &*fragment_chain };
        copy_from_fragment(fragment_chain, size).map_or(std::ptr::null_mut(), |buffer| {
            from_ser_buffer(sertype, kind, &buffer)
        })
    })
}

/// Construct a [`Serdata`] from a serialized `ddsrt_iovec_t` and return it as a
/// raw pointer.
///
/// ## Safety
/// - `serdata` must be a non-null pointer to a fully-initialized [`Serdata`].
/// - `containers` must point to a valid contiguous array of `ddsrt_iovec_t` structures of length
///   `containers_len`. Each `iov_base` must be a valid pointer to `iov_len` bytes of readable
///   memory.
/// - `size` must be less than or equal to the total number of bytes described by the
///   `ddsrt_iovec_t` array, or else out-of-bounds memory will be accessed when copying data.
/// - `T` must match the expected deserialized type, and the deserialization must not violate any
///   invariants assumed by the type (e.g., panic during construction or interior mutability
///   issues).
pub unsafe extern "C" fn from_ser_iov<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    kind: cyclonedds_sys::ddsi_serdata_kind,
    containers_len: cyclonedds_sys::ddsrt_msg_iovlen_t,
    containers: *const cyclonedds_sys::ddsrt_iovec_t,
    size: usize,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

    crate::internal::serdata::Kind::try_from(kind).map_or(std::ptr::null_mut(), |kind| {
        let mut buffer: Vec<u8> = Vec::with_capacity(size);

        // `ddsrt_msg_iovlen_t` is already a `usize` under Linux
        #[cfg(not(target_os = "linux"))]
        let Ok(containers_len) = usize::try_from(containers_len) else {
            return std::ptr::null_mut();
        };

        let containers = unsafe { std::slice::from_raw_parts(containers, containers_len) };

        let mut offset = 0;
        for container in containers {
            let container_iov_len = container.iov_len;

            // `ddsrt_iov_len_t` is a `usize` for every platform except Windows.
            #[cfg(target_os = "windows")]
            let Ok(container_iov_len) = usize::try_from(container_iov_len) else {
                return std::ptr::null_mut();
            };

            let len = if container_iov_len + offset > size {
                size - offset
            } else {
                container_iov_len
            };

            let container =
                unsafe { std::slice::from_raw_parts(container.iov_base as *const u8, len) };
            buffer.extend_from_slice(container);
            offset += len;
        }
        from_ser_buffer(sertype, kind, &buffer)
    })
}

pub(crate) fn from_keyhash_with_mode<T>(
    sertype: &Sertype<T>,
    keyhash: &cyclonedds_sys::ddsi_keyhash,
    force_md5: bool,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let max_possible_serialized_size = T::Key::max_serialized_cdr_size();

    if force_md5 || max_possible_serialized_size > CdrSize::Bounded(16) {
        // The key hash is based on MD5 and so can't be reconstructed into a key.
        std::ptr::null_mut()
    } else {
        // The key hash is just the big-endian CDR serialized form of the key.
        deserialize_boxed::<T::Key, byteorder::BigEndian>(&keyhash.value).map_or(
            std::ptr::null_mut(),
            |key| {
                let serdata = Box::new(Serdata::new(sertype, SampleOrKey::from_boxed_key(key)));
                Box::into_raw(serdata).cast()
            },
        )
    }
}

///
/// ## Safety
pub unsafe extern "C" fn from_keyhash<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    keyhash: *const cyclonedds_sys::ddsi_keyhash,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let force_md5 = T::FORCE_MD5_KEYHASH;
    let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };
    let keyhash = unsafe { &*keyhash };

    from_keyhash_with_mode::<T>(sertype, keyhash, force_md5)
}

/// Constructs a [`Serdata`] from a sample pointer, given a serialization kind.
///
/// ## Safety
/// `sertype` must be a valid, non-null pointer to a heap-allocated [`Sertype`].
pub unsafe extern "C" fn from_sample<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    kind: cyclonedds_sys::ddsi_serdata_kind,
    sample: *const std::ffi::c_void,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    if sertype.is_null() || sample.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: guaranteed by the caller.
    if unsafe { crate::internal::ffi::sertype_ops::representation::<T>(sertype) }
        == crate::internal::sertype::Representation::Raw
    {
        let Ok(kind) = crate::internal::serdata::Kind::try_from(kind) else {
            return std::ptr::null_mut();
        };

        let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };
        // SAFETY: guaranteed by the caller; a raw sample is a bare `T`.
        let sample = unsafe { &*sample.cast::<T>() };

        // A raw sample arrives either straight from the application or out of a
        // loan it filled in, and either way it has to be copied into the
        // `Serdata` because that outlives the call. This is the serializing
        // delivery path (network, or a local reader not using PSMX); the
        // zero-copy path short-circuits before a `Serdata` is ever built.
        let sample = match kind {
            crate::internal::serdata::Kind::Data => SampleOrKey::new_sample(sample.clone()),
            // The application representation of a key is a whole sample whose
            // key fields carry the value.
            crate::internal::serdata::Kind::Key => {
                SampleOrKey::new_key(crate::Topicable::as_key(sample))
            }
        };

        return Box::into_raw(Box::new(Serdata::new(sertype, sample))).cast();
    }

    let sample = unsafe { &*(sample.cast::<InternalSample<'_, T>>()) };
    match (crate::internal::serdata::Kind::try_from(kind), sample) {
        (Ok(crate::internal::serdata::Kind::Data), InternalSample::SampleRef(sample)) => {
            let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

            let sample = SampleOrKey::new_sample((*sample).clone());
            let serdata = Box::new(Serdata::new(sertype, sample));

            Box::into_raw(serdata).cast()
        }
        (Ok(crate::internal::serdata::Kind::Data), InternalSample::Sample(sample)) => {
            let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

            let sample = SampleOrKey::new_sample(sample.clone());
            let serdata = Box::new(Serdata::new(sertype, sample));

            Box::into_raw(serdata).cast()
        }
        (Ok(crate::internal::serdata::Kind::Key), InternalSample::KeyRef(key)) => {
            let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

            let key = SampleOrKey::new_key((*key).clone());
            let serdata = Box::new(Serdata::new(sertype, key));

            Box::into_raw(serdata).cast()
        }
        (Ok(crate::internal::serdata::Kind::Key), InternalSample::Key(key)) => {
            let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };

            let key = SampleOrKey::new_key(key.clone());
            let serdata = Box::new(Serdata::new(sertype, key));

            Box::into_raw(serdata).cast()
        }
        _ => std::ptr::null_mut(),
    }
}

/// TODO Unimplemented.
/// ## Safety
pub unsafe extern "C" fn to_ser<T>(
    serdata: *const cyclonedds_sys::ddsi_serdata,
    a: usize,
    b: usize,
    c: *mut std::ffi::c_void,
) {
    let args = (serdata, a, b, c);
    eprintln!(
        "serdata_ops::to_ser<{}>({args:?})",
        std::any::type_name::<T>()
    );
}

/// Write the serialized bytes of a sample to the provided
/// [`cyclonedds_sys::ddsrt_iovec_t`].
///
/// This increments the `serdata` reference counter.
///
/// ## Safety
/// - `serdata`  must be a non-null pointer to a fully-initialized [`Serdata`].
/// - `container` must be a non-null pointer to an [`cyclonedds_sys::ddsrt_iovec_t`].
pub unsafe extern "C" fn to_ser_ref<T>(
    serdata: *const cyclonedds_sys::ddsi_serdata,
    offset: usize,
    size: usize,
    container: *mut cyclonedds_sys::ddsrt_iovec_t,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };
    let container = unsafe { &mut *container };

    // Size is the size of the fragment so the total serialized data is
    // at least offset + size.
    let Some(end) = offset.checked_add(size) else {
        return std::ptr::null_mut();
    };

    // NOTE: on Windows, converting the fragment length to `ddsrt_iov_len_t`
    // can fail, so the closure must return an `Option` and use `and_then`.
    // On other platforms the conversion is compiled out, making this look like
    // `map` and triggering the `clippy::bind_instead_of_map` warning.
    #[allow(clippy::bind_instead_of_map)]
    serdata
        .serialized_with_size_hint(end)
        .ok()
        .and_then(|serialized| {
            // SAFETY: checked_add guarantees offset <= end, and the size hint
            // pads the serialized buffer to at least end bytes.
            let slice = serialized
                .get(offset..end)
                .expect("serialized buffer must contain the requested fragment");
            container.iov_base = slice.as_ptr() as *mut _;

            let iov_len = slice.len();
            // `ddsrt_iov_len_t` is a `usize` for every platform except Windows.
            #[cfg(target_os = "windows")]
            let iov_len = cyclonedds_sys::ddsrt_iov_len_t::try_from(iov_len).ok()?;
            container.iov_len = iov_len;
            Some(())
        })
        .map_or_default(|()| unsafe { cyclonedds_sys::ddsi_serdata_ref(&raw const serdata.inner) })
}

/// Relinquish the reference handed out by [`to_ser_ref`].
///
/// This decrements the `serdata` reference counter.
///
/// ## Safety
/// - `serdata`  must be a non-null pointer to a fully-initialized [`Serdata`].
pub unsafe extern "C" fn to_ser_unref<T>(
    serdata: *mut cyclonedds_sys::ddsi_serdata,
    _: *const cyclonedds_sys::ddsrt_iovec_t,
) where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata.cast::<Serdata<T>>()) };

    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
}

/// Copies a (deserialized) sample from a [`Serdata`] into a provided `sample`
/// pointer.
///
/// Returns `true` if a sample was present and copied, `false` otherwise.
///
/// ## Safety
/// - `sertype` must be a valid, non-null pointer to a heap-allocated [`Sertype`].
/// - `sample` must be a valid, non-null pointer that can hold a value of size `T`.
pub unsafe extern "C" fn to_sample<T>(
    serdata: *const cyclonedds_sys::ddsi_serdata,
    sample: *mut std::ffi::c_void,
    _buffer: *mut *mut std::ffi::c_void,
    _buffer_limit: *mut std::ffi::c_void,
) -> bool
where
    T: crate::Topicable,
{
    if sample.is_null() {
        false
    } else {
        let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };

        if serdata.representation == crate::internal::sertype::Representation::Raw {
            // SAFETY: the caller provides a buffer sized and aligned for the
            // topic's representation, which for this sertype is a bare `T`.
            unsafe { write_raw_sample(serdata, sample.cast::<T>()) };
            return true;
        }

        let sample = sample.cast::<InternalSample<'_, T>>();
        match serdata.kind() {
            crate::internal::serdata::Kind::Key => {
                let data = InternalSample::Key(serdata.key().clone());
                unsafe {
                    sample.write(data);
                }
                true
            }
            crate::internal::serdata::Kind::Data => {
                let data = InternalSample::Sample(serdata.sample().clone());
                unsafe {
                    sample.write(data);
                }
                true
            }
        }
    }
}

/// Write the sample a [`Serdata`] holds into a caller-provided buffer in the raw
/// representation.
///
/// A key-only serdata is expanded into a full sample through
/// [`Topicable::from_key`][crate::Topicable::from_key], since the raw
/// representation of a key is a whole sample whose key fields carry the value.
///
/// ## Safety
/// `destination` must be valid for writes of one `T` and aligned for `T`.
unsafe fn write_raw_sample<T>(serdata: &mut Serdata<T>, destination: *mut T)
where
    T: crate::Topicable,
{
    let sample = match serdata.kind() {
        crate::internal::serdata::Kind::Data => serdata.sample().clone(),
        crate::internal::serdata::Kind::Key => T::from_key(serdata.key()),
    };

    // SAFETY: guaranteed by the caller. `write` does not drop the previous
    // contents, which is correct: a memcpy-safe type owns nothing, and the buffer
    // may be uninitialized.
    unsafe { destination.write(sample) };
}

/// Create an untyped sertype based on provided typed [`Serdata`].
///
/// ## Safety
/// - `sertype` must be a valid, non-null pointer to a heap-allocated [`Sertype`].
pub unsafe extern "C" fn to_untyped<T>(
    serdata: *const cyclonedds_sys::ddsi_serdata,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };

    let sertype = unsafe { &mut *(serdata.inner.type_ as *mut Sertype<T>) };

    let mut untyped_serdata = Box::new(Serdata::new(
        sertype,
        SampleOrKey::new_key(serdata.sample.as_ref().key().clone()),
    ));
    untyped_serdata.inner.type_ = std::ptr::null_mut();

    Box::into_raw(untyped_serdata).cast()
}

///
/// ## Safety
pub unsafe extern "C" fn untyped_to_sample<T>(
    _sertype: *const cyclonedds_sys::ddsi_sertype,
    serdata: *const cyclonedds_sys::ddsi_serdata,
    sample: *mut std::ffi::c_void,
    _buffer: *mut *mut std::ffi::c_void,
    _buffer_limit: *mut std::ffi::c_void,
) -> bool
where
    T: crate::Topicable,
{
    if sample.is_null() {
        false
    } else {
        let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };

        if serdata.representation == crate::internal::sertype::Representation::Raw {
            // SAFETY: the caller provides a buffer sized and aligned for the
            // topic's representation, which for this sertype is a bare `T`.
            unsafe { write_raw_sample(serdata, sample.cast::<T>()) };
            return true;
        }

        let sample = sample.cast::<InternalSample<'_, T>>();
        match serdata.kind() {
            crate::internal::serdata::Kind::Data => unsafe {
                sample.write(InternalSample::Sample(serdata.sample().clone()));
            },
            crate::internal::serdata::Kind::Key => unsafe {
                sample.write(InternalSample::Key(serdata.key().clone()));
            },
        }
        true
    }
}

/// Deallocate a [`Serdata`].
///
/// ## Safety
/// - `serdata` must be a non-null pointer to a previously allocated [`Serdata<T>`].
/// - Cyclone DDS must call this only after the [`ddsi_serdata`][cyclonedds_sys::ddsi_serdata]
///   reference count has reached zero. After this call, no other references or serialized-data
///   loans may be used, and the pointer must not be passed to this function again.
pub unsafe extern "C" fn free<T>(serdata: *mut cyclonedds_sys::ddsi_serdata)
where
    T: crate::Topicable,
{
    let serdata = unsafe { Box::from_raw(serdata.cast::<Serdata<T>>()) };

    // A serdata built by `from_loaned_sample` holds a reference to the loan the
    // application published, and this is where it goes back. The C library's own
    // sertype releases it at the same point.
    let loan = serdata.inner.loan;
    if !loan.is_null() {
        // SAFETY: the reference was handed to us in `from_loaned_sample` and has
        // not been released since.
        unsafe { crate::internal::ffi::dds_loaned_sample_unref(loan) };
    }

    drop(serdata);
}

/// Writes the debug representation of a [`Serdata`] into the provided buffer.
///
/// This output relies on `T`’s [`std::fmt::Debug`] implementation to display
/// the sample held by the [`Serdata`]. At most `length` bytes are written. If
/// the formatted string is longer, the result is truncated. The buffer is
/// always null-terminated.
///
/// the return value is the number of bytes written to the buffer, **excluding**
/// the null terminator.
///
/// ## Safety
/// - `_sertype` must be either the [`Sertype<T>`] for `serdata` or the compatible type supplied by
///   Cyclone DDS when printing untyped serdata.
/// - `serdata` must be a non-null pointer to a fully-initialized [`Serdata<T>`].
/// - `buffer` must be non-null, valid for writes of `length` bytes, and `length` must be greater
///   than zero so the required trailing NULL terminator can be written.
pub unsafe extern "C" fn print<T>(
    _sertype: *const cyclonedds_sys::ddsi_sertype,
    serdata: *const cyclonedds_sys::ddsi_serdata,
    buffer: *mut std::ffi::c_char,
    length: usize,
) -> usize
where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };

    let buffer = unsafe { std::slice::from_raw_parts_mut(buffer.cast(), length) };
    let mut cursor = std::io::Cursor::new(&mut *buffer);

    // The formatting here is best-effort so ignore a potential error when
    // writing.
    let _ = write!(cursor, "{:#?}", *serdata);

    // Ensure that whatever was written is null-terminated.
    let written = cursor
        .position()
        .try_into()
        .unwrap_or(length)
        .min(length.saturating_sub(1));

    #[allow(clippy::indexing_slicing)]
    // `written` is guaranteed to be in bounds of the buffer because the
    // cursor written comes from was a cursor  into that self-same buffer.
    {
        buffer[written] = 0;
    }

    written
}

///
/// ## Safety
pub unsafe extern "C" fn get_keyhash<T>(
    serdata: *const cyclonedds_sys::ddsi_serdata,
    keyhash: *mut cyclonedds_sys::ddsi_keyhash,
    force_md5: bool,
) where
    T: crate::Topicable,
{
    let serdata = unsafe { &mut *(serdata as *mut Serdata<T>) };
    let keyhash = unsafe { &mut *keyhash };

    KeyHash::from_key::<T>(serdata.key(), force_md5)
        .inspect(|serdata_keyhash| keyhash.value.copy_from_slice(&serdata_keyhash.0));
}

/// How the payload of a PSMX loan is laid out in the loaned memory.
///
/// Distinct from [`Representation`][crate::internal::sertype::Representation],
/// which is how a *topic* passes samples across the FFI boundary: this describes
/// one loan that arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PayloadLayout {
    /// The application's own in-memory representation of the sample.
    ///
    /// Only meaningful for types Cyclone DDS considers memcpy-safe, which a
    /// Serde/CDR based [`Topicable`][crate::Topicable] is not.
    Raw,
    /// A CDR/XCDR1/XCDR2-serialized sample, *without* the four byte
    /// encapsulation header. The header is carried out-of-band in the
    /// `cdr_identifier` and `cdr_options` fields of the loan metadata.
    Serialized,
}

/// Classify the state of a PSMX loan into the serdata [`Kind`] it carries and
/// the [`PayloadLayout`] its payload uses.
///
/// Returns [`None`] for an uninitialized loan, which carries no payload at all,
/// and for any state the C library may add in the future.
///
/// This mirrors `loaned_sample_state_to_serdata_kind` in the C library's default
/// serdata implementation.
pub(crate) const fn classify_loan(
    state: cyclonedds_sys::dds_loaned_sample_state_t,
) -> Option<(crate::internal::serdata::Kind, PayloadLayout)> {
    use crate::internal::serdata::Kind;

    match state {
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_KEY => {
            Some((Kind::Key, PayloadLayout::Raw))
        }
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA => {
            Some((Kind::Data, PayloadLayout::Raw))
        }
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_KEY => {
            Some((Kind::Key, PayloadLayout::Serialized))
        }
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA => {
            Some((Kind::Data, PayloadLayout::Serialized))
        }
        // `DDS_LOANED_SAMPLE_STATE_UNITIALIZED` and anything the C library adds later.
        _ => None,
    }
}

/// Construct a [`Serdata`] from a sample the application built in memory the
/// writer loaned it, taking over ownership of the loan.
///
/// Cyclone DDS calls this when a sample published through
/// [`Writer::write_loaned`][crate::Writer::write_loaned] also needs a serdata —
/// that is, when the loan came from the heap rather than a PSMX interface,
/// because a serdata is what feeds the network and non-PSMX local readers. When
/// PSMX carries the sample instead, Cyclone DDS goes through [`from_sample`] and
/// this op is not involved.
///
/// It must also stay installed for a second reason: `dds_ensure_valid_psmx_instances`
/// in the C library refuses to assign any PSMX Instance to a *writer* whose
/// sertype leaves this op null, so nulling it out would disable shared-memory
/// publishing altogether.
///
/// On success the loan is retained by the returned [`Serdata`] and released when
/// that is freed, mirroring what the C library's own sertype does. Retaining it
/// rather than releasing it here keeps `sample` — which points into the loan —
/// valid for as long as anything can still reach it.
///
/// ## Safety
/// - `sertype` must be a non-null pointer to a fully-initialized [`Sertype<T>`].
/// - `sample` must be a non-null pointer to a fully-initialized sample in the representation
///   `sertype` declares.
/// - `loan` must be non-null, and ownership of one reference to it passes to this function on
///   success.
pub unsafe extern "C" fn from_loaned_sample<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    kind: cyclonedds_sys::ddsi_serdata_kind,
    sample: *const std::ffi::c_char,
    loan: *mut cyclonedds_sys::dds_loaned_sample,
    _will_require_cdr: bool,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    if loan.is_null() {
        return std::ptr::null_mut();
    }

    // The loaned memory holds a sample in the topic's own representation, which
    // is exactly what `from_sample` reads.
    //
    // SAFETY: guaranteed by the caller.
    let serdata = unsafe { from_sample::<T>(sertype, kind, sample.cast()) };

    if serdata.is_null() {
        // Ownership of the loan stays with the caller on failure.
        return std::ptr::null_mut();
    }

    // Take over the caller's reference. `free` drops it again.
    //
    // SAFETY: `from_sample` returned a non-null pointer to a freshly allocated
    // `Serdata<T>`, which starts with its `ddsi_serdata`.
    unsafe { &mut *serdata }.loan = loan;

    serdata
}

/// Construct a [`Serdata`] from a PSMX loan holding a sample in the raw
/// application representation.
///
/// This is the receiving half of zero-copy: no serialization was involved, so the
/// sample is read straight out of the loaned (typically shared) memory.
///
/// Only valid for a topic that opted into zero-copy, whose
/// [`FixedSizeTopic`][crate::FixedSizeTopic] contract is what makes reading bytes
/// another process wrote as a `T` sound. A raw loan arriving for any other topic
/// is rejected: the payload would be some other binding's memory layout, with
/// nothing valid to reinterpret it as.
///
/// The sample is copied out of the loan rather than borrowed from it, because the
/// samples this binding hands back from a read own their data. That is one
/// `memcpy` and no encoding or decoding, against a serialize plus deserialize for
/// an ordinary topic.
///
/// ## Safety
/// - `sertype` must be a non-null pointer to a fully-initialized [`Sertype<T>`].
/// - `loan` must be non-null with non-null `metadata`, and its `sample_ptr` valid for reads of
///   `size_of::<T>()` bytes.
unsafe fn from_psmx_raw<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    kind: crate::internal::serdata::Kind,
    loan: &cyclonedds_sys::dds_loaned_sample,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    // SAFETY: guaranteed by the caller.
    if unsafe { crate::internal::ffi::sertype_ops::representation::<T>(sertype) }
        != crate::internal::sertype::Representation::Raw
    {
        return std::ptr::null_mut();
    }

    let sample = loan.sample_ptr.cast::<T>();

    // A PSMX implementation is only obliged to align loans for the types IDL maps
    // to. Reading through a misaligned reference is undefined behaviour, so drop
    // the sample instead.
    if sample.is_null() || !sample.is_aligned() {
        return std::ptr::null_mut();
    }

    // A raw loan holds a sample in exactly the representation `from_sample`
    // reads for this sertype, so reuse it rather than restating the conversion.
    //
    // SAFETY: guaranteed by the caller, and checked above to be non-null and
    // aligned. Any bit pattern is a valid `T` by the `FixedSize` contract, which
    // the raw representation implies.
    unsafe { from_sample::<T>(sertype, kind.into(), sample.cast()) }
}

/// Construct a [`Serdata`] from a sample received over a PSMX (shared memory)
/// interface.
///
/// Cyclone DDS calls this from `dds_reader_store_loaned_sample` when a PSMX
/// Instance delivers a sample, in place of the RTPS receive path. Returning a
/// null pointer tells Cyclone DDS the sample could not be interpreted and makes
/// it drop the sample, which is the correct outcome for a payload this binding
/// cannot decode.
///
/// A [`PayloadLayout::Serialized`] payload is deserialized into an owned `T`.
/// This avoids the network stack and its copies but is not zero-copy: the writer
/// still encoded the sample and this decodes it.
///
/// A [`PayloadLayout::Raw`] payload is handled by [`from_psmx_raw`], which is
/// the receiving half of zero-copy and needs no decoding at all.
///
/// Either way the sample is copied out of the loan, so the returned [`Serdata`]
/// holds no reference to the loaned memory and leaves
/// [`ddsi_serdata::loan`][cyclonedds_sys::ddsi_serdata::loan] null. Cyclone DDS
/// is then free to drop the loan as soon as this returns.
///
/// ## Safety
/// - `sertype` must be a non-null pointer to a fully-initialized [`Sertype<T>`].
/// - `loan` must be a non-null pointer to a fully-initialized `dds_loaned_sample` whose
///   `metadata` is non-null, and whose `sample_ptr` is valid for reads of `metadata.sample_size`
///   bytes when the state is one of the serialized variants.
/// - The loan must remain valid for the duration of this call. Ownership is not taken.
pub unsafe extern "C" fn from_psmx<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    loan: *mut cyclonedds_sys::dds_loaned_sample,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    if sertype.is_null() || loan.is_null() {
        return std::ptr::null_mut();
    }

    let loan = unsafe { &*loan };

    if loan.metadata.is_null() {
        return std::ptr::null_mut();
    }

    let metadata = unsafe { &*loan.metadata };

    let Some((kind, payload)) = classify_loan(metadata.sample_state) else {
        return std::ptr::null_mut();
    };

    if payload == PayloadLayout::Raw {
        // SAFETY: guaranteed by the caller.
        return unsafe { from_psmx_raw::<T>(sertype, kind, loan) };
    }

    let Ok(sample_size) = usize::try_from(metadata.sample_size) else {
        return std::ptr::null_mut();
    };

    if loan.sample_ptr.is_null() && sample_size != 0 {
        return std::ptr::null_mut();
    }

    // The writing side splits the four byte encapsulation header off the
    // serialized sample and stores it in the metadata (see
    // `dds_writer_psmx_loan_from_serdata` in the C library), so reassemble it
    // here to get back the buffer the deserializer expects.
    //
    // Both fields hold the header bytes as loaded through a native `uint16_t`,
    // so writing them back out in native byte order reproduces the original
    // wire bytes on either endianness.
    let mut buffer = Vec::with_capacity(DDSI_RTPS_HEADER_SIZE + sample_size);
    buffer.extend_from_slice(&metadata.cdr_identifier.to_ne_bytes());
    buffer.extend_from_slice(&metadata.cdr_options.to_ne_bytes());

    if sample_size != 0 {
        // SAFETY: guaranteed by the caller, `sample_ptr` is valid for reads of
        // `sample_size` bytes.
        buffer.extend_from_slice(unsafe {
            std::slice::from_raw_parts(loan.sample_ptr.cast::<u8>(), sample_size)
        });
    }

    // NOTE: `sample_size` may include trailing alignment padding, indicated by
    // the low bits of `cdr_options`. Trailing bytes past the end of the encoded
    // sample are ignored by the deserializer, so there is nothing to trim.
    let sertype = unsafe { &mut *(sertype as *mut Sertype<T>) };
    let serdata = from_ser_buffer(sertype, kind, &buffer);

    if serdata.is_null() {
        return std::ptr::null_mut();
    }

    // Carry over the writer's timestamp and status info. Cyclone DDS reads
    // `statusinfo` off the returned serdata immediately after this call to look
    // up the writer, so this is not optional.
    //
    // SAFETY: `from_ser_buffer` returned a non-null pointer to a freshly
    // allocated `Serdata<T>`, which starts with its `ddsi_serdata`.
    let inner = unsafe { &mut *serdata };
    inner.timestamp.v = metadata.timestamp;
    inner.statusinfo = metadata.statusinfo;

    serdata
}
