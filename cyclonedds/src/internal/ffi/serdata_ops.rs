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

pub(crate) fn from_ser_buffer<T>(
    sertype: &crate::internal::sertype::Sertype<T>,
    kind: crate::internal::serdata::Kind,
    buffer: &[u8],
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    fn deserialize<'a, T: serde::Deserialize<'a>>(
        buffer: &[u8],
    ) -> Result<(T, usize), cdr_encoding::Error> {
        match buffer.split_at_checked(DDSI_RTPS_HEADER_SIZE) {
            Some((header, bytes)) => match header {
                [0x0, 0x0, ..] => cdr_encoding::from_bytes::<T, byteorder::BigEndian>(bytes),
                [0x0, 0x1, ..] => cdr_encoding::from_bytes::<T, byteorder::LittleEndian>(bytes),
                _ => Err(cdr_encoding::Error::Message(format!(
                    "could not determine endianness from CDR header: {header:?}"
                ))),
            },
            None => Err(cdr_encoding::Error::Message(format!(
                "deserialization failed: byteslice too short to contain valid CDR header: \
                 {buffer:?}"
            ))),
        }
    }

    match kind {
        crate::internal::serdata::Kind::Key => {
            if let Ok((key, _)) = deserialize::<T::Key>(buffer) {
                let key = SampleOrKey::new_key(key);
                let serdata = Box::new(crate::internal::serdata::Serdata::new(sertype, key));
                Box::into_raw(serdata).cast()
            } else {
                std::ptr::null_mut()
            }
        }
        crate::internal::serdata::Kind::Data => {
            if let Ok((data, _)) = deserialize::<T>(buffer) {
                let sample = SampleOrKey::new_sample(data);
                let serdata = Box::new(crate::internal::serdata::Serdata::new(sertype, sample));
                Box::into_raw(serdata).cast()
            } else {
                std::ptr::null_mut()
            }
        }
    }
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
        if let Ok((key, _)) = cdr_encoding::from_bytes::<_, byteorder::BigEndian>(&keyhash.value) {
            let serdata = Box::new(Serdata::new(sertype, SampleOrKey::<T>::new_key(key)));
            Box::into_raw(serdata).cast()
        } else {
            std::ptr::null_mut()
        }
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

    serdata
        .serialized_with_size_hint(size)
        .ok()
        .and_then(|serialized| {
            let slice = serialized.get(offset..)?;
            container.iov_base = slice.as_ptr() as *mut _;

            let iov_len = slice.len();
            // `ddsrt_iov_len_t` is a `usize` for every platform except Windows.
            #[cfg(target_os = "windows")]
            let iov_len = cyclonedds_sys::ddsrt_iov_len_t::try_from(iov_len).ok()?;
            container.iov_len = iov_len;
            Some(())
        })
        .map(|()| unsafe { cyclonedds_sys::ddsi_serdata_ref(&raw const serdata.inner) })
        .unwrap_or_default()
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
    let _ = write!(cursor, "{:#?}", &*serdata);

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

/// TODO Unimplemented.
/// ## Safety
pub unsafe extern "C" fn from_loaned_sample<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    a: cyclonedds_sys::ddsi_serdata_kind,
    b: *const std::ffi::c_char,
    c: *mut cyclonedds_sys::dds_loaned_sample,
    d: bool,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let args = (sertype, a, b, c, d);
    eprintln!(
        "serdata_ops::from_loaned_sample<{}>({args:?})",
        std::any::type_name::<T>()
    );
    std::ptr::null_mut()
}

/// TODO Unimplemented.
/// ## Safety
pub unsafe extern "C" fn from_psmx<T>(
    sertype: *const cyclonedds_sys::ddsi_sertype,
    loan: *mut cyclonedds_sys::dds_loaned_sample,
) -> *mut cyclonedds_sys::ddsi_serdata
where
    T: crate::Topicable,
{
    let args = (sertype, loan);

    eprintln!(
        "serdata_ops::from_psmx<{}>({args:?})",
        std::any::type_name::<T>()
    );

    std::ptr::null_mut()
}
