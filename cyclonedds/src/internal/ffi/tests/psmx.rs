//! Unit tests for the shared-memory paths.
//!
//! Kept apart from the tests either side of them because they are what this work
//! added: the loans a PSMX Instance delivers, the raw sample representation, and
//! the serialize pair the PSMX write path calls. The fixtures only these use live
//! here too; `test_sertype` stays with the tests that share it.

use super::*;

/// `to_ser` copies a slice of the serialized form out, header included.
///
/// This is how Cyclone DDS fills a PSMX loan from an existing serdata
/// (`dds_writer_psmx_loan_from_serdata`): four bytes of encapsulation header
/// first, then the payload. Leaving it unimplemented published whatever bytes the
/// loan already held, and only to peers reached through PSMX — so nothing local
/// ever noticed.
#[test]
fn test_serdata_ops_to_ser_copies_the_serialized_form() {
    let sertype = test_sertype();

    let sample = SampleOrKeyInner::new_sample(crate::tests::topic::Data {
        x: 3,
        y: -4,
        message: "to_ser".to_owned(),
    });
    let serdata = Box::new(crate::internal::serdata::Serdata::new(&sertype, sample));
    let expected = serdata.serialized().unwrap().to_vec();
    let serdata_ptr = std::ptr::from_ref(&serdata.inner);

    // The header, as `dds_writer_psmx_loan_from_serdata` asks for it.
    let mut header = [0xaa_u8; 4];
    unsafe {
        serdata_ops::to_ser::<crate::tests::topic::Data>(
            serdata_ptr,
            0,
            header.len(),
            header.as_mut_ptr().cast(),
        );
    }
    assert_eq!(
        header,
        expected[..4],
        "the encapsulation header must come back"
    );

    // Then the payload, at the offset that follows it.
    let mut payload = vec![0xaa_u8; expected.len() - 4];
    unsafe {
        serdata_ops::to_ser::<crate::tests::topic::Data>(
            serdata_ptr,
            4,
            payload.len(),
            payload.as_mut_ptr().cast(),
        );
    }
    assert_eq!(payload, expected[4..], "the payload must come back");

    // A range past the end writes nothing rather than reading out of bounds.
    let mut untouched = [0xaa_u8; 8];
    unsafe {
        serdata_ops::to_ser::<crate::tests::topic::Data>(
            serdata_ptr,
            expected.len(),
            untouched.len(),
            untouched.as_mut_ptr().cast(),
        );
    }
    assert_eq!(
        untouched, [0xaa; 8],
        "an out-of-range request must be a no-op"
    );

    let mut sertype = sertype;
    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
    let _ = Box::into_raw(serdata);
}

/// A PSMX loan carrying a serialized payload, laid out the way the C library's
/// writer side does it.
///
/// `dds_writer_psmx_loan_from_serdata` splits the four byte encapsulation header
/// off the serialized sample, stores it in the metadata, and puts only the
/// remainder in the loaned memory. The fixture reproduces that split so
/// [`serdata_ops::from_psmx`] is exercised against a realistic loan.
struct PsmxLoanFixture {
    loan: cyclonedds_sys::dds_loaned_sample,
    metadata: Box<cyclonedds_sys::dds_psmx_metadata>,
    payload: Vec<u8>,
}

impl PsmxLoanFixture {
    /// Build a loan for `sample`, serialized in native byte order.
    fn new<T>(sample: &T, state: cyclonedds_sys::dds_loaned_sample_state_t) -> Self
    where
        T: serde::Serialize,
    {
        let mut payload = Vec::new();
        cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut payload, sample).unwrap();

        Self::from_payload(payload, state)
    }

    /// Build a loan around an already-encoded `payload`, without the
    /// encapsulation header.
    ///
    /// A raw state gets the `cdr_identifier` a PSMX Instance is required to use
    /// for an unserialized payload, matching what the C library's own write path
    /// puts there.
    fn from_payload(payload: Vec<u8>, state: cyclonedds_sys::dds_loaned_sample_state_t) -> Self {
        use crate::internal::traits::CdrHeader;

        let [identifier_hi, identifier_lo, options_hi, options_lo] =
            byteorder::NativeEndian::cdr_header();

        // SAFETY: `dds_psmx_metadata` is repr(C) and consists solely of
        // integers, C enums, and a `dds_guid_t` byte array, so an all-zero bit
        // pattern is a valid instance.
        let mut metadata: Box<cyclonedds_sys::dds_psmx_metadata> =
            Box::new(unsafe { std::mem::MaybeUninit::zeroed().assume_init() });

        metadata.sample_state = state;
        metadata.sample_size = u32::try_from(payload.len()).unwrap();
        metadata.cdr_identifier = match state {
            cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_KEY
            | cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA => {
                cyclonedds_sys::RTPS_SAMPLE_NATIVE
            }
            _ => u16::from_ne_bytes([identifier_hi, identifier_lo]),
        };
        metadata.cdr_options = u16::from_ne_bytes([options_hi, options_lo]);

        // SAFETY: `dds_loaned_sample` is repr(C) and consists solely of raw
        // pointers, an optional function pointer, a C enum, and an integer, so
        // an all-zero bit pattern is a valid instance.
        let mut loan: cyclonedds_sys::dds_loaned_sample =
            unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

        loan.loan_origin.origin_kind =
            cyclonedds_sys::dds_loaned_sample_origin_kind_DDS_LOAN_ORIGIN_KIND_PSMX;

        let mut fixture = Self {
            loan,
            metadata,
            payload,
        };

        fixture.loan.metadata = &raw mut *fixture.metadata;
        fixture.loan.sample_ptr = fixture.payload.as_mut_ptr().cast();

        fixture
    }

    /// The loan, as Cyclone DDS would pass it to the op.
    fn as_ptr(&mut self) -> *mut cyclonedds_sys::dds_loaned_sample {
        &raw mut self.loan
    }
}

#[test]
fn test_serdata_ops_from_psmx_serialized_data() {
    let sample = crate::tests::topic::Data {
        x: 7,
        y: -9,
        message: "shared memory".to_string(),
    };

    let mut sertype = test_sertype();
    let mut fixture = PsmxLoanFixture::new(
        &sample,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA,
    );

    // The writer's timestamp and status info travel in the metadata and must be
    // carried onto the serdata: Cyclone DDS reads `statusinfo` off it as soon as
    // this returns, to look up the originating writer.
    fixture.metadata.timestamp = 1_234_567_890;
    fixture.metadata.statusinfo = 0;

    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            fixture.as_ptr(),
        )
    };
    assert_ne!(result, std::ptr::null_mut(), "expected a serdata");

    {
        let inner = unsafe { &*result };
        assert_eq!(inner.timestamp.v, 1_234_567_890);
        assert_eq!(inner.statusinfo, 0);
        assert_eq!(
            inner.loan,
            std::ptr::null_mut(),
            "a serialized payload is copied out, so the serdata must not retain the loan"
        );
    }

    let serdata = unsafe {
        Box::from_raw(result.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Data);
    assert_eq!(serdata.sample(), &sample);
    drop(serdata);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_psmx_serialized_key() {
    let sample = crate::tests::topic::Data {
        x: 11,
        y: -13,
        message: "ignored for a key".to_string(),
    };
    let key = crate::Topicable::as_key(&sample);

    let mut sertype = test_sertype();
    let mut fixture = PsmxLoanFixture::new(
        &key,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_KEY,
    );

    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            fixture.as_ptr(),
        )
    };
    assert_ne!(result, std::ptr::null_mut(), "expected a serdata");

    let serdata = unsafe {
        Box::from_raw(result.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Key);
    assert_eq!(serdata.key(), &key);
    drop(serdata);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_psmx_rejects_unusable_loans() {
    let sample = crate::tests::topic::Data {
        x: 1,
        y: 2,
        message: "rejected".to_string(),
    };

    let mut sertype = test_sertype();

    // A raw payload is the writing binding's own memory layout for the type. A
    // `Topicable` is encoded through Serde rather than laid out in a fixed,
    // memcpy-safe representation, so there is nothing valid to reinterpret it
    // as and the sample has to be dropped.
    for state in [
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_KEY,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_UNITIALIZED,
    ] {
        let mut fixture = PsmxLoanFixture::new(&sample, state);
        let result = unsafe {
            serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
                &raw const sertype.inner,
                fixture.as_ptr(),
            )
        };
        assert_eq!(
            result,
            std::ptr::null_mut(),
            "sample state {state} should be rejected"
        );
    }

    // A payload that is not decodable as the topic type.
    let mut fixture = PsmxLoanFixture::from_payload(
        vec![0xff; 3],
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA,
    );
    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            fixture.as_ptr(),
        )
    };
    assert_eq!(
        result,
        std::ptr::null_mut(),
        "an undecodable payload should be rejected"
    );

    // A loan with no metadata at all.
    let mut fixture = PsmxLoanFixture::new(
        &sample,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA,
    );
    fixture.loan.metadata = std::ptr::null_mut();
    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            fixture.as_ptr(),
        )
    };
    assert_eq!(
        result,
        std::ptr::null_mut(),
        "a loan without metadata should be rejected"
    );

    // Null arguments.
    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(result, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_classify_loan() {
    use crate::internal::ffi::serdata_ops::{PayloadLayout, classify_loan};
    use crate::internal::serdata::Kind;

    assert_eq!(
        classify_loan(cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_KEY),
        Some((Kind::Key, PayloadLayout::Raw))
    );
    assert_eq!(
        classify_loan(cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA),
        Some((Kind::Data, PayloadLayout::Raw))
    );
    assert_eq!(
        classify_loan(
            cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_KEY
        ),
        Some((Kind::Key, PayloadLayout::Serialized))
    );
    assert_eq!(
        classify_loan(
            cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA
        ),
        Some((Kind::Data, PayloadLayout::Serialized))
    );
    assert_eq!(
        classify_loan(cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_UNITIALIZED),
        None
    );
    // Anything the C library may add later.
    assert_eq!(classify_loan(u32::MAX), None);
}

#[test]
fn test_sertype_ops_get_serialized_size() {
    use crate::internal::traits::CdrHeader;

    let mut sertype = test_sertype();

    let sample = crate::tests::topic::Data {
        x: 42,
        y: -42,
        message: "sized".to_string(),
    };

    // The size the op reports must exclude the four byte encapsulation header,
    // and must be exactly what `serialize_into` goes on to write. Cyclone DDS
    // sizes the PSMX loan from the first and fills it with the second, so a
    // mismatch either truncates the sample or over-allocates.
    let mut expected = Vec::new();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut expected, &sample).unwrap();

    let mut internal_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::SampleRef(&sample);

    let mut size = 0_usize;
    let mut encoding_identifier = 0_u16;
    let result = unsafe {
        sertype_ops::get_serialized_size::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw mut internal_sample).cast(),
            &raw mut size,
            &raw mut encoding_identifier,
        )
    };
    assert_eq!(result, 0, "expected DDS_RETCODE_OK");
    assert_eq!(size, expected.len());

    // The identifier holds the first two header bytes as loaded through a native
    // `uint16_t`, which is the representation `from_psmx` reverses.
    let [identifier_high, identifier_low, _, _] = byteorder::NativeEndian::cdr_header();
    assert_eq!(
        encoding_identifier,
        u16::from_ne_bytes([identifier_high, identifier_low])
    );

    // Writing into a buffer of exactly the reported size must succeed and
    // produce the same bytes.
    let mut destination_buffer: Vec<u8> = vec![0; size];
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw mut internal_sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(result);
    assert_eq!(destination_buffer, expected);

    // Keys are measured the same way.
    let key = crate::Topicable::as_key(&sample);
    let mut expected_key = Vec::new();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut expected_key, &key).unwrap();

    let mut internal_key =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::KeyRef(&key);
    let mut size = 0_usize;
    let result = unsafe {
        sertype_ops::get_serialized_size::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Key.into(),
            (&raw mut internal_key).cast(),
            &raw mut size,
            &raw mut encoding_identifier,
        )
    };
    assert_eq!(result, 0, "expected DDS_RETCODE_OK");
    assert_eq!(size, expected_key.len());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_get_serialized_size_rejects_bad_input() {
    let mut sertype = test_sertype();

    let mut size = 0_usize;
    let mut encoding_identifier = 0_u16;

    // Null out-parameters and a null sample.
    for (sample, size_out, identifier_out) in [
        (
            std::ptr::null_mut::<std::ffi::c_void>(),
            &raw mut size,
            &raw mut encoding_identifier,
        ),
        (
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &raw mut encoding_identifier,
        ),
        (std::ptr::null_mut(), &raw mut size, std::ptr::null_mut()),
    ] {
        let result = unsafe {
            sertype_ops::get_serialized_size::<crate::tests::topic::Data, Serialized>(
                &raw const sertype.inner,
                crate::internal::serdata::Kind::Data.into(),
                sample,
                size_out,
                identifier_out,
            )
        };
        assert_eq!(result, cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER);
    }

    // A sample representation that does not match the requested kind: asking for
    // data while holding only a key.
    let key = (1_u32, 2_i32);
    let mut internal_key =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::KeyRef(&key);
    let result = unsafe {
        sertype_ops::get_serialized_size::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw mut internal_key).cast(),
            &raw mut size,
            &raw mut encoding_identifier,
        )
    };
    assert_eq!(result, cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

/// Build a sertype for [`RawData`] in the raw representation.
///
/// Boxed for the same reason as [`test_sertype`].
#[allow(
    clippy::unnecessary_box_returns,
    reason = "the tests leak the box deliberately, in place of running `Sertype`'s `Drop`"
)]
fn raw_sertype() -> Box<crate::internal::sertype::Sertype<RawData, ZeroCopy>> {
    let type_name = std::ffi::CString::new(RawData::dds_type_name().as_ref()).unwrap();

    Box::new(crate::internal::sertype::Sertype::new(
        &type_name,
        RawData::IS_KEYED,
    ))
}

#[test]
fn test_sertype_raw_declares_memcpy_safe_properties() {
    let mut sertype = raw_sertype();

    // These are exactly what Cyclone DDS keys off when deciding whether a sample
    // may be moved as raw memory: without them it silently falls back to
    // serializing, and the zero-copy path would never run.
    assert_eq!(
        sertype.inner.is_memcpy_safe(),
        1,
        "a raw sertype must be memcpy-safe"
    );
    assert_eq!(
        sertype.inner.sizeof_type as usize,
        std::mem::size_of::<RawData>()
    );
    assert_eq!(sertype.inner.has_key(), u32::from(RawData::IS_KEYED));

    // The ordinary sertype must be unchanged: no size, not memcpy-safe.
    let mut internal = test_sertype();
    assert_eq!(internal.inner.is_memcpy_safe(), 0);
    assert_eq!(internal.inner.sizeof_type, 0);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
    crate::internal::ffi::ddsi_sertype_unref(&mut internal.inner);
    let _ = Box::into_raw(internal);
}

/// A raw loan whose metadata does not claim native encoding is rejected.
///
/// The state says the payload is the application's own memory while the encoding
/// identifier says it is serialized CDR. The two cannot both be true, so there is
/// no reason to believe the bytes are this topic's type — reinterpreting them as
/// one would be reading another layout entirely. The C library's own sertype
/// refuses on the same grounds.
#[test]
fn test_serdata_ops_from_psmx_raw_rejects_a_non_native_encoding() {
    use crate::internal::traits::CdrHeader;

    let sample = RawData {
        id: 9,
        sequence: 4,
        payload: [1, 2, 3, 4, 5, 6, 7, 8],
    };

    let sertype = raw_sertype();

    let payload = unsafe {
        std::slice::from_raw_parts(
            (&raw const sample).cast::<u8>(),
            std::mem::size_of::<RawData>(),
        )
    }
    .to_vec();

    let mut fixture = PsmxLoanFixture::from_payload(
        payload,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA,
    );

    // Claim a serialized encoding, which contradicts the raw state.
    let [identifier_hi, identifier_lo, _, _] = byteorder::NativeEndian::cdr_header();
    fixture.metadata.cdr_identifier = u16::from_ne_bytes([identifier_hi, identifier_lo]);

    let result = unsafe {
        serdata_ops::from_psmx::<RawData, ZeroCopy>(&raw const sertype.inner, fixture.as_ptr())
    };
    assert_eq!(
        result,
        std::ptr::null_mut(),
        "a raw loan that does not claim native encoding must be dropped"
    );

    let mut sertype = sertype;
    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

/// A raw loan smaller than the topic's type is rejected.
///
/// A peer can share the type name and still disagree about its size, and reading
/// a whole sample out of a shorter loan would run past the end of it.
#[test]
fn test_serdata_ops_from_psmx_raw_rejects_a_short_loan() {
    let sertype = raw_sertype();

    let mut fixture = PsmxLoanFixture::from_payload(
        vec![0; std::mem::size_of::<RawData>()],
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA,
    );
    fixture.metadata.sample_size -= 1;

    let result = unsafe {
        serdata_ops::from_psmx::<RawData, ZeroCopy>(&raw const sertype.inner, fixture.as_ptr())
    };
    assert_eq!(
        result,
        std::ptr::null_mut(),
        "a raw loan shorter than the sample must be dropped"
    );

    let mut sertype = sertype;
    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_psmx_raw_data() {
    let sample = RawData {
        id: 9,
        sequence: 4,
        payload: [1, 2, 3, 4, 5, 6, 7, 8],
    };

    let mut sertype = raw_sertype();

    // A raw loan carries the sample's own bytes, with no encapsulation header.
    let payload = unsafe {
        std::slice::from_raw_parts(
            (&raw const sample).cast::<u8>(),
            std::mem::size_of::<RawData>(),
        )
    }
    .to_vec();

    let mut fixture = PsmxLoanFixture::from_payload(
        payload,
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA,
    );

    let result = unsafe {
        serdata_ops::from_psmx::<RawData, ZeroCopy>(&raw const sertype.inner, fixture.as_ptr())
    };
    assert_ne!(result, std::ptr::null_mut(), "expected a serdata");

    let serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<RawData>>()) };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Data);
    assert_eq!(serdata.sample(), &sample);
    drop(serdata);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_psmx_raw_rejected_for_an_ordinary_topic() {
    // A topic that did not opt into the raw representation has no way to
    // interpret another binding's memory layout, so a raw loan must be dropped
    // rather than reinterpreted.
    let mut sertype = test_sertype();

    let mut fixture = PsmxLoanFixture::from_payload(
        vec![0; 64],
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA,
    );

    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            fixture.as_ptr(),
        )
    };
    assert_eq!(result, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_raw_sample_buffers_round_trip() {
    // Cyclone DDS allocates application samples through these ops, notably for
    // the heap loans it falls back to when no PSMX interface is configured. The
    // raw representation follows the same contract as the C library's own
    // sertype: one contiguous block, with `pointers[i]` addressing the `i`th
    // sample in it.
    const COUNT: usize = 3;

    let mut sertype = raw_sertype();
    let mut pointers = [std::ptr::null_mut::<std::ffi::c_void>(); COUNT];

    unsafe {
        sertype_ops::realloc_samples::<RawData, ZeroCopy>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            std::ptr::null_mut(),
            0,
            COUNT,
        );
    }

    for (index, pointer) in pointers.iter().enumerate() {
        assert!(!pointer.is_null());
        assert_eq!(
            *pointer as usize,
            pointers[0] as usize + index * std::mem::size_of::<RawData>(),
            "samples must be contiguous"
        );
        // New samples are zeroed, which is a valid `RawData`.
        assert_eq!(unsafe { *pointer.cast::<RawData>() }, RawData::default());
    }

    // Writing and then zeroing must go through the raw layout, not the binding's
    // internal enum.
    unsafe {
        pointers[1].cast::<RawData>().write(RawData {
            id: 1,
            sequence: 2,
            payload: [9; 8],
        });
        sertype_ops::zero_samples::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            pointers[0],
            COUNT,
        );
    }
    assert_eq!(
        unsafe { *pointers[1].cast::<RawData>() },
        RawData::default()
    );

    unsafe {
        sertype_ops::free_samples::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            COUNT,
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_serialize_raw_representation() {
    let sample = RawData {
        id: 3,
        sequence: 5,
        payload: [7; 8],
    };

    let mut sertype = raw_sertype();

    // In the raw representation the pointer Cyclone DDS passes is a bare `T`, not
    // the binding's own sample enum, so these ops have to read it as one.
    let mut expected = Vec::new();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut expected, &sample).unwrap();

    let mut size = 0_usize;
    let mut encoding_identifier = 0_u16;
    let result = unsafe {
        sertype_ops::get_serialized_size::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw const sample).cast(),
            &raw mut size,
            &raw mut encoding_identifier,
        )
    };
    assert_eq!(result, 0);
    assert_eq!(size, expected.len());

    let mut buffer = vec![0_u8; size];
    let result = unsafe {
        sertype_ops::serialize_into::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw const sample).cast(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    assert!(result);
    assert_eq!(buffer, expected);

    // A key is a whole sample whose key fields carry the value.
    let mut expected_key = Vec::new();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut expected_key, &sample.id)
        .unwrap();

    let mut size = 0_usize;
    let result = unsafe {
        sertype_ops::get_serialized_size::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Key.into(),
            (&raw const sample).cast(),
            &raw mut size,
            &raw mut encoding_identifier,
        )
    };
    assert_eq!(result, 0);
    assert_eq!(size, expected_key.len());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_sample_raw_representation() {
    let sample = RawData {
        id: 21,
        sequence: 22,
        payload: [3; 8],
    };

    let mut sertype = raw_sertype();

    let result = unsafe {
        serdata_ops::from_sample::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Data.into(),
            (&raw const sample).cast(),
        )
    };
    assert_ne!(result, std::ptr::null_mut());

    let serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<RawData>>()) };
    assert_eq!(serdata.sample(), &sample);
    drop(serdata);

    // A key built from the same pointer takes only the key fields.
    let result = unsafe {
        serdata_ops::from_sample::<RawData, ZeroCopy>(
            &raw const sertype.inner,
            crate::internal::serdata::Kind::Key.into(),
            (&raw const sample).cast(),
        )
    };
    assert_ne!(result, std::ptr::null_mut());

    let serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<RawData>>()) };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Key);
    assert_eq!(serdata.key(), &sample.id);
    drop(serdata);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_to_sample_raw_representation() {
    let sample = RawData {
        id: 31,
        sequence: 32,
        payload: [4; 8],
    };

    let mut sertype = raw_sertype();
    let serdata = Box::new(crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_sample(sample),
    ));

    // Cyclone DDS provides a buffer in the topic's representation, which here is
    // a bare `RawData` rather than the binding's sample enum.
    let mut destination = RawData::default();
    let result = unsafe {
        serdata_ops::to_sample::<RawData, ZeroCopy>(
            &raw const serdata.inner,
            (&raw mut destination).cast(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(result);
    assert_eq!(destination, sample);

    drop(serdata);
    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}
