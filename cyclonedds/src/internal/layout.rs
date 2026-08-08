#![allow(unsafe_code)]

//! How each [`Representation`][crate::representation] interprets the sample
//! pointers Cyclone DDS passes across the FFI boundary.
//!
//! Cyclone DDS hands application samples around as a bare `void *` and leaves
//! their meaning to the sertype's operations. This module holds that meaning, one
//! implementation of [`Layout`] per representation marker, so that every
//! operation which has to read or write a sample pointer resolves the choice at
//! compile time.
//!
//! Keeping it in a trait rather than a runtime flag is what makes the raw paths
//! unreachable for a type that did not opt in: [`Layout`] is implemented for
//! [`ZeroCopy`] only where `T` is a [`FixedSizeTopic`][crate::FixedSizeTopic], so
//! a sertype that would `memcpy` an unsuitable type cannot be instantiated.

use crate::internal::ffi::{InternalSample, SampleArgument};
use crate::internal::serdata::{Kind, Serdata};
use crate::representation::{Serialized, ZeroCopy};
use crate::sample::SampleOrKeyInner as SampleOrKey;

/// The interpretation of the `void *` Cyclone DDS uses for application samples of
/// `T`.
///
/// Every method that takes a sample pointer states the representation it expects
/// in its own safety contract; the implementation is what defines it.
pub(crate) trait Layout<T>: 'static
where
    T: crate::Topicable,
{
    /// Whether Cyclone DDS may move samples of this topic as raw memory.
    ///
    /// Reported to the C library when the sertype is created, and what makes it
    /// serve a loan request out of a PSMX Instance rather than the heap.
    const IS_MEMCPY_SAFE: bool;

    /// Read the sample or key that `sample` points at.
    ///
    /// Returns [`None`] when the pointer does not hold a payload of `kind`, which
    /// tells the caller to reject the sample.
    ///
    /// ## Safety
    /// `sample` must be a non-null pointer to a fully-initialized sample in this
    /// representation.
    unsafe fn read_sample(kind: Kind, sample: *const std::ffi::c_void) -> Option<SampleOrKey<T>>;

    /// Write what `serdata` holds into `destination` in this representation.
    ///
    /// ## Safety
    /// `destination` must be valid for writes of one sample in this
    /// representation, and aligned for it.
    unsafe fn write_sample(serdata: &mut Serdata<T>, destination: *mut std::ffi::c_void);

    /// CDR-encode the payload of the sample `sample` points at.
    ///
    /// Writes native-endian CDR *without* the four byte encapsulation header,
    /// which the `get_serialized_size`/`serialize_into` pair of sertype ops
    /// reports separately.
    ///
    /// ## Safety
    /// `sample` must be a non-null pointer to a fully-initialized sample in this
    /// representation.
    unsafe fn serialize_payload<W>(
        kind: Kind,
        sample: *const std::ffi::c_void,
        writer: W,
    ) -> Result<(), cdr_encoding::Error>
    where
        W: std::io::Write;

    /// Reset `count` consecutive samples to an empty value.
    ///
    /// ## Safety
    /// `samples` must be valid for writes of `count` samples in this
    /// representation.
    unsafe fn zero_samples(samples: *mut std::ffi::c_void, count: usize);

    /// Grow or shrink a buffer of application samples to `new_count`, writing one
    /// pointer per sample into `pointers`.
    ///
    /// ## Safety
    /// - `pointers` must be valid for writes of `new_count` pointers.
    /// - `old_samples`, when non-null, must be `old_count` samples previously produced by this
    ///   function for the same representation.
    unsafe fn realloc_samples(
        pointers: *mut *mut std::ffi::c_void,
        old_samples: *mut std::ffi::c_void,
        old_count: usize,
        new_count: usize,
    );

    /// Release samples previously produced by [`realloc_samples`][Self::realloc_samples].
    ///
    /// `free_all` releases the samples themselves, `free_contents` only what they
    /// own.
    ///
    /// ## Safety
    /// `pointers` must point to `count` sample pointers produced by
    /// [`realloc_samples`][Self::realloc_samples] for the same representation.
    unsafe fn free_samples(
        pointers: *mut *mut std::ffi::c_void,
        count: usize,
        free_all: bool,
        free_contents: bool,
    );

    /// Package a whole sample for handing to Cyclone DDS.
    fn sample_argument(sample: &T) -> SampleArgument<'_, T>;

    /// Package a key for handing to Cyclone DDS.
    fn key_argument(key: &T::Key) -> SampleArgument<'_, T>;

    /// Reinterpret the raw payload of a PSMX loan as a `T` that stays in the
    /// loaned memory.
    ///
    /// Returns [`None`] when this representation cannot interpret such a payload,
    /// which makes the caller drop the sample.
    ///
    /// ## Safety
    /// `sample` must point to memory holding one fully-initialized sample in this
    /// representation, which must outlive the returned pointer.
    unsafe fn borrow_raw_loan(sample: *const std::ffi::c_void) -> Option<*const T>;
}

impl<T> Layout<T> for Serialized
where
    T: crate::Topicable,
{
    const IS_MEMCPY_SAFE: bool = false;

    unsafe fn read_sample(kind: Kind, sample: *const std::ffi::c_void) -> Option<SampleOrKey<T>> {
        // SAFETY: guaranteed by the caller.
        let sample = unsafe { &*sample.cast::<InternalSample<'_, T>>() };

        match (kind, sample) {
            (Kind::Data, InternalSample::SampleRef(sample)) => {
                Some(SampleOrKey::new_sample((*sample).clone()))
            }
            (Kind::Data, InternalSample::Sample(sample)) => {
                Some(SampleOrKey::new_sample(sample.clone()))
            }
            (Kind::Key, InternalSample::KeyRef(key)) => Some(SampleOrKey::new_key((*key).clone())),
            (Kind::Key, InternalSample::Key(key)) => Some(SampleOrKey::new_key(key.clone())),
            _ => None,
        }
    }

    unsafe fn write_sample(serdata: &mut Serdata<T>, destination: *mut std::ffi::c_void) {
        let destination = destination.cast::<InternalSample<'_, T>>();

        let sample = match serdata.kind() {
            Kind::Data => InternalSample::Sample(serdata.sample().clone()),
            Kind::Key => InternalSample::Key(serdata.key().clone()),
        };

        // SAFETY: guaranteed by the caller.
        unsafe { destination.write(sample) };
    }

    unsafe fn serialize_payload<W>(
        kind: Kind,
        sample: *const std::ffi::c_void,
        writer: W,
    ) -> Result<(), cdr_encoding::Error>
    where
        W: std::io::Write,
    {
        // SAFETY: guaranteed by the caller.
        let sample = unsafe { &*sample.cast::<InternalSample<'_, T>>() };

        match (kind, sample) {
            (Kind::Data, InternalSample::SampleRef(sample)) => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(writer, *sample)
            }
            (Kind::Data, InternalSample::Sample(sample)) => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(writer, sample)
            }
            (Kind::Key, InternalSample::KeyRef(key)) => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(writer, *key)
            }
            (Kind::Key, InternalSample::Key(key)) => {
                cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(writer, key)
            }
            _ => Err(cdr_encoding::Error::Message(format!(
                "cannot serialize a {kind:?} serdata from this sample representation"
            ))),
        }
    }

    unsafe fn zero_samples(samples: *mut std::ffi::c_void, count: usize) {
        let samples = samples.cast::<InternalSample<'_, T>>();

        for index in 0..count {
            // SAFETY: guaranteed by the caller.
            unsafe { samples.add(index).write(InternalSample::None) };
        }
    }

    unsafe fn realloc_samples(
        pointers: *mut *mut std::ffi::c_void,
        old_samples: *mut std::ffi::c_void,
        old_count: usize,
        new_count: usize,
    ) {
        let pointers = pointers.cast::<*mut InternalSample<'_, T>>();
        let old_samples = old_samples.cast::<InternalSample<'_, T>>();

        // SAFETY: guaranteed by the caller.
        let pointers = unsafe { std::slice::from_raw_parts_mut(pointers, new_count) };

        // Copy over initial samples.
        if !old_samples.is_null() && old_count != 0 {
            // SAFETY: guaranteed by the caller.
            let old_samples = unsafe { std::slice::from_raw_parts_mut(old_samples, old_count) };

            let count = old_count.min(new_count);

            // Split into samples to keep and samples to drop
            let (samples_to_keep, samples_to_drop) = old_samples.split_at_mut(count);

            // Move retained samples
            for (pointer, sample) in pointers.iter_mut().zip(samples_to_keep.iter()) {
                // SAFETY: the sample is left behind in `old_samples`, which the
                // caller relinquishes, so it is not read again.
                let sample = unsafe { std::ptr::read(sample) };
                *pointer = Box::into_raw(Box::new(sample));
            }

            // If it's shrinking.
            for sample in samples_to_drop {
                // SAFETY: the sample is initialized and not retained above.
                unsafe { std::ptr::drop_in_place(sample) };
            }

            #[allow(clippy::indexing_slicing)]
            // `count <= pointers.len()` as `count` is the minimum of `new_count`
            // (which `pointers` was created with) and `old_count`.
            let remaining_slots = &mut pointers[count..];

            // Allocate new samples for additional slots
            for pointer in remaining_slots {
                *pointer = Box::into_raw(Box::new(InternalSample::None));
            }
        } else {
            // Allocate new samples for additional slots
            for pointer in pointers {
                *pointer = Box::into_raw(Box::new(InternalSample::None));
            }
        }
    }

    unsafe fn free_samples(
        pointers: *mut *mut std::ffi::c_void,
        count: usize,
        free_all: bool,
        free_contents: bool,
    ) {
        let samples = pointers.cast::<*mut InternalSample<'_, T>>();

        if free_all {
            for index in 0..count {
                // SAFETY: guaranteed by the caller; each pointer came from
                // `Box::into_raw` in `realloc_samples`.
                let sample = unsafe { Box::from_raw(*samples.add(index)) };
                drop(sample);
            }
        } else if free_contents {
            for index in 0..count {
                // SAFETY: guaranteed by the caller.
                unsafe { (*samples.add(index)).drop_in_place() };
            }
        }
    }

    fn sample_argument(sample: &T) -> SampleArgument<'_, T> {
        SampleArgument::Internal(InternalSample::SampleRef(sample))
    }

    fn key_argument(key: &T::Key) -> SampleArgument<'_, T> {
        SampleArgument::Internal(InternalSample::KeyRef(key))
    }

    unsafe fn borrow_raw_loan(_sample: *const std::ffi::c_void) -> Option<*const T> {
        // The payload is some other binding's memory layout, and a Serde/CDR
        // based `Topicable` has nothing valid to reinterpret it as.
        None
    }
}

impl<T> Layout<T> for ZeroCopy
where
    T: crate::FixedSizeTopic,
{
    const IS_MEMCPY_SAFE: bool = true;

    unsafe fn read_sample(kind: Kind, sample: *const std::ffi::c_void) -> Option<SampleOrKey<T>> {
        // SAFETY: guaranteed by the caller; a raw sample is a bare `T`.
        let sample = unsafe { &*sample.cast::<T>() };

        // A raw sample arrives either straight from the application or out of a
        // loan it filled in, and either way it has to be copied because the
        // `Serdata` outlives the call.
        Some(match kind {
            Kind::Data => SampleOrKey::new_sample(*sample),
            // The application representation of a key is a whole sample whose
            // key fields carry the value.
            Kind::Key => SampleOrKey::new_key(crate::Topicable::as_key(sample)),
        })
    }

    unsafe fn write_sample(serdata: &mut Serdata<T>, destination: *mut std::ffi::c_void) {
        // A key-only serdata is expanded into a whole sample, since the raw
        // representation of a key is a sample whose key fields carry the value.
        let sample = match serdata.kind() {
            Kind::Data => *serdata.sample(),
            Kind::Key => T::from_key(serdata.key()),
        };

        // SAFETY: guaranteed by the caller. `write` does not drop the previous
        // contents, which is correct: a memcpy-safe type owns nothing, and the
        // buffer may be uninitialized.
        unsafe { destination.cast::<T>().write(sample) };
    }

    unsafe fn serialize_payload<W>(
        kind: Kind,
        sample: *const std::ffi::c_void,
        writer: W,
    ) -> Result<(), cdr_encoding::Error>
    where
        W: std::io::Write,
    {
        // SAFETY: guaranteed by the caller; a raw sample is a bare `T`.
        let sample = unsafe { &*sample.cast::<T>() };

        match kind {
            Kind::Data => cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(writer, sample),
            Kind::Key => cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(
                writer,
                &crate::Topicable::as_key(sample),
            ),
        }
    }

    unsafe fn zero_samples(samples: *mut std::ffi::c_void, count: usize) {
        // Matches what the C library's default sertype does: a memcpy-safe type
        // is required to be valid when all-zero, so clearing the bytes is enough
        // and there is nothing to drop.
        //
        // SAFETY: guaranteed by the caller to be valid for `count` samples of
        // `size_of::<T>()` bytes each.
        unsafe {
            std::ptr::write_bytes(samples.cast::<u8>(), 0, std::mem::size_of::<T>() * count);
        }
    }

    /// Follows the same contract as the C library's default sertype: one
    /// contiguous block holds every sample, `pointers[i]` addresses the `i`th
    /// sample within it, and new samples are zeroed. Allocation goes through
    /// `dds_realloc` so that the block can be released with `dds_free`, exactly as
    /// Cyclone DDS expects, and so that
    /// [`free_samples`][Self::free_samples] needs no layout information.
    unsafe fn realloc_samples(
        pointers: *mut *mut std::ffi::c_void,
        old_samples: *mut std::ffi::c_void,
        old_count: usize,
        new_count: usize,
    ) {
        let size = std::mem::size_of::<T>();

        let block = if old_count == new_count {
            old_samples
        } else {
            // SAFETY: `old_samples` is either null or a block from a previous
            // call, which was allocated through `dds_alloc`/`dds_realloc`.
            unsafe { cyclonedds_sys::dds_realloc(old_samples, size * new_count) }
        };

        if block.is_null() {
            return;
        }

        if new_count > old_count {
            // SAFETY: `block` is valid for `size * new_count` bytes, so the tail
            // beyond `old_count` samples is in bounds. A memcpy-safe type is valid
            // when all-zero.
            unsafe {
                std::ptr::write_bytes(
                    block.cast::<u8>().add(size * old_count),
                    0,
                    size * (new_count - old_count),
                );
            }
        }

        for index in 0..new_count {
            // SAFETY: `pointers` is valid for `new_count` writes, and each
            // computed address lies within `block`.
            unsafe {
                pointers
                    .add(index)
                    .write(block.cast::<u8>().add(size * index).cast());
            }
        }
    }

    unsafe fn free_samples(
        pointers: *mut *mut std::ffi::c_void,
        count: usize,
        free_all: bool,
        _free_contents: bool,
    ) {
        // A memcpy-safe type owns nothing, so there are no contents to release;
        // only the single block backing every sample has to go. As in the C
        // library's default sertype, that block is addressed by the first pointer.
        if free_all && count > 0 {
            // SAFETY: `pointers` holds at least one pointer, and it addresses the
            // start of a block allocated by `realloc_samples` through
            // `dds_realloc`.
            unsafe { cyclonedds_sys::dds_free(*pointers) };
        }
    }

    fn sample_argument(sample: &T) -> SampleArgument<'_, T> {
        SampleArgument::RawBorrowed(sample)
    }

    fn key_argument(key: &T::Key) -> SampleArgument<'_, T> {
        // In the raw representation there is no separate key type: a key is a
        // whole sample whose key fields carry the value, which is the convention
        // Cyclone DDS and the C bindings use. One therefore has to be
        // materialized.
        SampleArgument::RawOwned(T::from_key(key))
    }

    unsafe fn borrow_raw_loan(sample: *const std::ffi::c_void) -> Option<*const T> {
        // The loaned bytes already *are* a `T`: any bit pattern is a valid one by
        // the `FixedSize` contract, which this representation requires. So the
        // receive side needs no conversion at all — this is the half of zero-copy
        // that makes a read free of copies.
        Some(sample.cast::<T>())
    }
}
