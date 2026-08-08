//! Borrowed sample memory, for publishing without a copy.

use crate::internal::ffi;
use crate::representation::ZeroCopy;
use crate::{Result, Writer};

/// Memory borrowed from a [`Writer`] to build a sample in place.
///
/// Obtained from [`Writer::loan`] and published with [`Writer::write_loaned`].
/// When the writer's topic opted into zero-copy and its domain has a
/// shared-memory PSMX interface, this memory *is* the shared memory that readers
/// on the same machine will look at, so a sample written here reaches them
/// without being copied or serialized on the way.
///
/// Filling the sample through [`as_uninit_mut`](Loan::as_uninit_mut) is what
/// makes that worthwhile for a large payload: [`write`](Loan::write) has to build
/// the sample somewhere first and then move it in, whereas writing through the
/// borrowed memory directly touches the payload once.
///
/// Dropping a `Loan` without publishing it hands the memory back to the writer.
///
/// # Examples
///
/// See [`Writer::loan`].
#[derive(Debug)]
pub struct Loan<'writer, 'domain, 'participant, 'topic, T>
where
    T: crate::FixedSizeTopic,
{
    writer: &'writer Writer<'domain, 'participant, 'topic, T, ZeroCopy>,
    /// The loaned memory, sized and aligned for one `T`. Never null.
    sample: *mut T,
}

impl<'w, 'd, 'p, 't, T> Loan<'w, 'd, 'p, 't, T>
where
    T: crate::FixedSizeTopic,
{
    /// Borrow memory for one sample from `writer`.
    pub(crate) fn new(writer: &'w Writer<'d, 'p, 't, T, ZeroCopy>) -> Result<Self> {
        // The writer's `ZeroCopy` representation is what makes this sound. Without
        // it the sertype would not be memcpy-safe, so Cyclone DDS would answer the
        // request with heap memory allocated through this binding's own
        // `realloc_samples` — that is, an `InternalSample<T>` — and writing a bare
        // `T` over one would be undefined behaviour.
        let sample = ffi::dds_request_loan(writer.inner)?;

        // Cyclone DDS sizes the loan from the type size registered with the
        // sertype, which is `size_of::<T>()` here. A PSMX implementation is only
        // required to align loaned memory for the types IDL maps to, so check
        // rather than trust: a misaligned reference would be undefined
        // behaviour. This is a third-party plugin's doing, so it is an error
        // rather than a panic.
        if !sample.cast::<T>().is_aligned() {
            let _ = ffi::dds_return_loan(writer.inner, sample);
            return Err(crate::Error::NonSpecific);
        }

        Ok(Self {
            writer,
            sample: sample.cast::<T>(),
        })
    }

    /// The loaned memory, as a sample that may not be initialized yet.
    ///
    /// Write the payload through this to keep the transfer free of copies. The
    /// memory starts out holding whatever the PSMX implementation last left
    /// there.
    ///
    /// Leaving it partly initialized is not unsound — every bit pattern is a
    /// valid `T` by the [`FixedSizeTopic`](crate::FixedSizeTopic) contract — but
    /// readers will see whatever those bytes happen to be.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::builder::TopicBuilder;
    /// # use cyclonedds::{Domain, Participant, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, cyclonedds::FixedSize, serde::Serialize, serde::Deserialize,
    /// #     Clone, Copy, Debug, Default,
    /// # )]
    /// # #[repr(C)]
    /// # struct Frame {
    /// #     #[dds(key)]
    /// #     id: u32,
    /// #     pixels: [u8; 32],
    /// # }
    /// # let topic = TopicBuilder::<Frame>::new(&participant, "Frames").with_zero_copy().build()?;
    /// # let writer = Writer::new(&topic)?;
    /// let mut loan = writer.loan()?;
    ///
    /// // Fill the payload in place, rather than building a `Frame` and moving it.
    /// let frame = loan.as_uninit_mut().write(Frame::default());
    /// frame.id = 3;
    /// frame.pixels[0] = 0xff;
    ///
    /// writer.write_loaned(loan)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[allow(
        unsafe_code,
        reason = "reborrowing the loaned memory, which is the point of this type"
    )]
    pub fn as_uninit_mut(&mut self) -> &mut std::mem::MaybeUninit<T> {
        // SAFETY: `sample` is non-null, aligned for `T` (asserted in `new`), and
        // valid for writes of one `T` until this `Loan` is published or dropped.
        // `MaybeUninit<T>` has the same layout as `T` and imposes no
        // initialization requirement.
        unsafe { &mut *self.sample.cast::<std::mem::MaybeUninit<T>>() }
    }

    /// Fill the loaned memory with `sample`.
    ///
    /// Convenient, but it builds the sample before moving it into the loan. Use
    /// [`as_uninit_mut`](Loan::as_uninit_mut) when that extra move matters.
    pub fn write(&mut self, sample: T) -> &mut T {
        self.as_uninit_mut().write(sample)
    }

    /// Give up ownership of the loaned memory, for handing to Cyclone DDS.
    ///
    /// Suppresses the [`Drop`] that would otherwise return the loan, since a
    /// write takes ownership of it.
    pub(crate) fn into_raw(self) -> *mut std::ffi::c_void {
        // `ManuallyDrop` rather than dropping: publishing hands the loan to
        // Cyclone DDS, so it must not also be returned.
        let loan = std::mem::ManuallyDrop::new(self);
        loan.sample.cast()
    }

    /// The writer this loan came from.
    pub(crate) const fn writer(&self) -> &'w Writer<'d, 'p, 't, T, ZeroCopy> {
        self.writer
    }
}

impl<T> Drop for Loan<'_, '_, '_, '_, T>
where
    T: crate::FixedSizeTopic,
{
    fn drop(&mut self) {
        // A loan that was never published goes back to the writer. Nothing useful
        // can be done if that fails, and it must not panic while dropping.
        let _ = ffi::dds_return_loan(self.writer.inner, self.sample.cast());
    }
}

/// A sample borrowed from a [`Reader`](crate::Reader).
///
/// Handed out by [`Reader::take_loaned`](crate::Reader::take_loaned) and its
/// siblings. Where an ordinary [`read`](crate::Reader::read) copies the payload
/// out, this borrows it in place and derefs to it, releasing it on drop.
///
/// On a topic that opted into zero-copy that borrow reaches all the way: for a
/// sample delivered through a shared-memory PSMX Instance, what this derefs to
/// *is* the writer's shared memory, never copied on the way. On any other topic
/// the payload was decoded into the reader's own memory, so this saves the copy a
/// read would otherwise make but nothing more.
///
/// The borrow is tied to the reader, so a sample cannot outlive it. That matters
/// most for a zero-copy topic, where holding samples holds shared memory that the
/// writer's PSMX Instance cannot reuse until they go back.
///
/// # Key-only samples
///
/// A disposed or unregistered instance arrives as a key with no payload. There is
/// nothing to borrow in that case, so the sample is materialized through
/// [`Topicable::from_key`](crate::Topicable::from_key) — the same value an
/// ordinary read would produce. [`is_key`](LoanedSample::is_key) distinguishes
/// the two.
///
/// # Examples
///
/// See [`Reader::take_loaned`](crate::Reader::take_loaned).
pub struct LoanedSample<'reader, T>
where
    T: crate::Topicable,
{
    /// One reference to the serdata, which is what keeps the payload alive — and
    /// with it, for a zero-copy topic, the PSMX loan the payload lives in.
    serdata: *mut cyclonedds_sys::ddsi_serdata,
    /// Set only for a key-only sample, which has no payload to borrow.
    materialized: Option<Box<T>>,
    info: crate::sample::Info,
    phantom_reader: std::marker::PhantomData<&'reader ()>,
}

impl<T> LoanedSample<'_, T>
where
    T: crate::Topicable,
{
    /// Take over one reference to `serdata`.
    ///
    /// ## Safety
    /// - `serdata` must point to a fully-initialized [`Serdata<T>`][crate::internal::serdata::Serdata],
    ///   and ownership of one reference to it passes to the returned value.
    /// - `materialized` must be [`Some`] exactly when the serdata carries only a key, since that is
    ///   when there is no payload to borrow.
    #[allow(
        unsafe_code,
        reason = "taking over a reference the caller must vouch for"
    )]
    pub(crate) const unsafe fn new(
        serdata: *mut cyclonedds_sys::ddsi_serdata,
        materialized: Option<Box<T>>,
        info: crate::sample::Info,
    ) -> Self {
        Self {
            serdata,
            materialized,
            info,
            phantom_reader: std::marker::PhantomData,
        }
    }

    /// The [`Info`](crate::sample::Info) Cyclone DDS reported alongside the
    /// sample.
    #[must_use]
    pub const fn info(&self) -> &crate::sample::Info {
        &self.info
    }

    /// Whether this arrived as a key-only notification, and so was materialized
    /// rather than borrowed.
    #[must_use]
    pub const fn is_key(&self) -> bool {
        self.materialized.is_some()
    }

    /// The key of the instance this sample belongs to.
    ///
    /// Returned by value rather than by reference. A borrowed key would have to be
    /// stored somewhere, and the only place available is the serdata — which
    /// several samples can share across threads, so caching one there is what this
    /// type is built to avoid. For a full sample the key is extracted from the
    /// payload; for a key-only notification it is the key that arrived.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cyclonedds::builder::TopicBuilder;
    /// # use cyclonedds::{Domain, Participant, Reader, Topicable, Writer};
    /// # let domain = Domain::default();
    /// # let participant = Participant::new(&domain)?;
    /// # #[derive(
    /// #     cyclonedds::Topicable, cyclonedds::FixedSize, serde::Serialize, serde::Deserialize,
    /// #     Clone, Copy, Debug, Default,
    /// # )]
    /// # #[repr(C)]
    /// # struct Reading {
    /// #     #[dds(key)]
    /// #     sensor_id: u32,
    /// #     sequence: u32,
    /// #     value: f64,
    /// # }
    /// # let topic = TopicBuilder::<Reading>::new(&participant, "Readings")
    /// #     .with_zero_copy()
    /// #     .build()?;
    /// # let reader = Reader::new(&topic)?;
    /// # let writer = Writer::new(&topic)?;
    /// let sample = Reading {
    ///     sensor_id: 4,
    ///     sequence: 1,
    ///     value: 0.5,
    /// };
    /// writer.write(&sample)?;
    ///
    /// let samples = reader.take_loaned()?;
    /// assert_eq!(samples[0].key(), sample.as_key());
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    #[must_use]
    pub fn key(&self) -> T::Key {
        crate::Topicable::as_key(&**self)
    }
}

impl<T> std::ops::Deref for LoanedSample<'_, T>
where
    T: crate::Topicable,
{
    type Target = T;

    #[allow(
        unsafe_code,
        reason = "borrowing the payload the retained serdata owns, which is the point of this type"
    )]
    fn deref(&self) -> &T {
        if let Some(materialized) = &self.materialized {
            return materialized;
        }

        // SAFETY: the reference taken when this was built keeps the `Serdata`, and
        // any loan behind it, alive for at least as long as `self`. The pointer
        // was non-null at construction, since a key-only sample is the only case
        // with no payload and that one is materialized instead.
        unsafe {
            &*(*self.serdata.cast::<crate::internal::serdata::Serdata<T>>())
                .sample_ptr()
                .unwrap_unchecked()
        }
    }
}

impl<T> std::fmt::Debug for LoanedSample<'_, T>
where
    T: crate::Topicable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoanedSample")
            .field("sample", &**self)
            .field("info", &self.info)
            .finish()
    }
}

impl<T> Drop for LoanedSample<'_, T>
where
    T: crate::Topicable,
{
    #[allow(
        unsafe_code,
        reason = "relinquishing the serdata reference this type was handed"
    )]
    fn drop(&mut self) {
        // SAFETY: this owns exactly one reference, taken when it was built and not
        // released since. Once it reaches zero Cyclone DDS frees the serdata,
        // which is where any PSMX loan behind it goes back.
        unsafe {
            let serdata = &mut *self.serdata;
            ffi::ddsi_serdata_unref(serdata);
        }
    }
}
