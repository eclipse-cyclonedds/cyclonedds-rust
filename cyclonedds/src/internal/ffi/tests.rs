use super::*;
use crate::Topicable;
use crate::representation::{Serialized, ZeroCopy};
use crate::sample::SampleOrKeyInner;
use crate::tests::topic::RawData;

unsafe extern "C" fn sertype_free_mock(_: *mut cyclonedds_sys::ddsi_sertype) {}

#[test]
fn test_zeroed_serdata_ops() {
    let actual = crate::internal::ffi::serdata_ops::zeroed_serdata_ops();

    // Check all currently known fields.
    assert!(actual.eqkey.is_none());
    assert!(actual.get_size.is_none());
    assert!(actual.from_ser.is_none());
    assert!(actual.from_ser_iov.is_none());
    assert!(actual.from_keyhash.is_none());
    assert!(actual.from_sample.is_none());
    assert!(actual.to_ser.is_none());
    assert!(actual.to_ser_ref.is_none());
    assert!(actual.to_ser_unref.is_none());
    assert!(actual.to_sample.is_none());
    assert!(actual.to_untyped.is_none());
    assert!(actual.untyped_to_sample.is_none());
    assert!(actual.free.is_none());
    assert!(actual.print.is_none());
    assert!(actual.get_keyhash.is_none());
    assert!(actual.from_loaned_sample.is_none());
    assert!(actual.from_psmx.is_none());
}

#[test]
fn test_zeroed_sertype_ops() {
    let actual = crate::internal::ffi::sertype_ops::zeroed_sertype_ops();

    // Check all currently known fields.
    assert!(actual.version.is_none());
    assert_eq!(actual.arg, std::ptr::null_mut());
    assert!(actual.free.is_none());
    assert!(actual.zero_samples.is_none());
    assert!(actual.realloc_samples.is_none());
    assert!(actual.free_samples.is_none());
    assert!(actual.equal.is_none());
    assert!(actual.hash.is_none());
    assert!(actual.type_id.is_none());
    assert!(actual.type_map.is_none());
    assert!(actual.type_info.is_none());
    assert!(actual.derive_sertype.is_none());
    assert!(actual.get_serialized_size.is_none());
    assert!(actual.serialize_into.is_none());
}

#[test]
fn test_ddsi_sertype_new_ref_unref_fini() {
    let mut sertype = ddsi_sertype_new(
        c"data",
        &cyclonedds_sys::ddsi_sertype_ops {
            free: Some(sertype_free_mock),
            ..crate::internal::sertype::Sertype::<crate::tests::topic::Data, Serialized>::SERTYPE_OPS
        },
        &crate::internal::sertype::Sertype::<crate::tests::topic::Data, Serialized>::SERDATA_OPS,
        false,
    );
    assert_eq!(sertype.flags_refc.v, 1);
    ddsi_sertype_ref(&mut sertype);
    assert_eq!(sertype.flags_refc.v, 2);
    ddsi_sertype_unref(&mut sertype);
    assert_eq!(sertype.flags_refc.v, 1);
    ddsi_sertype_unref(&mut sertype);
    assert_eq!(sertype.flags_refc.v, 0);
    ddsi_sertype_fini(&mut sertype);
}

#[test]
fn test_ddsi_serdata_new_ref_unref() {
    let mut sertype = ddsi_sertype_new(
        c"data",
        &cyclonedds_sys::ddsi_sertype_ops {
            free: Some(sertype_free_mock),
            ..crate::internal::sertype::Sertype::<crate::tests::topic::Data, Serialized>::SERTYPE_OPS
        },
        &crate::internal::sertype::Sertype::<crate::tests::topic::Data, Serialized>::SERDATA_OPS,
        false,
    );

    let mut serdata = ddsi_serdata_new(&sertype, crate::internal::serdata::Kind::Data.into());
    assert_eq!(serdata.refc.v, 1);
    ddsi_serdata_ref(&mut serdata);
    assert_eq!(serdata.refc.v, 2);
    ddsi_serdata_unref(&mut serdata);
    assert_eq!(serdata.refc.v, 1);

    ddsi_sertype_unref(&mut sertype);
    assert_eq!(sertype.flags_refc.v, 0);
    ddsi_sertype_fini(&mut sertype);
}

#[test]
fn test_dds_delete_on_non_existent_entity() {
    let entity = 101;
    let result = dds_delete(entity);
    assert!(result.is_err());
}

#[test]
fn test_dds_peek_read_take_on_invalid_entity() {
    let result = dds_peek_read_take::<crate::tests::topic::Data, read_operation::Peek>(0);
    assert!(result.is_err());
}

#[test]
fn test_dds_read_with_collector_callback_on_key() {
    let info = cyclonedds_sys::dds_sample_info_t::default();
    let topic_name = c"Data";
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(topic_name, crate::tests::topic::Data::IS_KEYED));

    let key = (10, 20);
    let sample = SampleOrKeyInner::<crate::tests::topic::Data>::new_key(key);
    let mut serdata = crate::internal::serdata::Serdata::new(&sertype, sample);

    let mut samples = Vec::<crate::sample::SampleOrKey<crate::tests::topic::Data>>::new();
    let arg = (&raw mut samples).cast::<std::ffi::c_void>();

    unsafe {
        dds_read_with_collector_callback::<crate::tests::topic::Data>(
            arg,
            &raw const info,
            &raw const sertype.inner,
            &raw mut serdata.inner,
        );
    }

    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].key().unwrap(), &key);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_unimplemented_sertype_ops() {
    let result = unsafe {
        sertype_ops::type_id::<crate::tests::topic::Data>(std::ptr::null(), Default::default())
    };
    assert_eq!(result, std::ptr::null_mut());

    let result = unsafe { sertype_ops::type_map::<crate::tests::topic::Data>(std::ptr::null()) };
    assert_eq!(result, std::ptr::null_mut());

    let result = unsafe {
        sertype_ops::derive_sertype::<crate::tests::topic::Data>(
            std::ptr::null(),
            Default::default(),
            cyclonedds_sys::dds_type_consistency_enforcement_qospolicy::default(),
        )
    };
    assert_eq!(result, std::ptr::null_mut());

    let result = unsafe { sertype_ops::type_info::<crate::tests::topic::Data>(std::ptr::null()) };
    assert_eq!(result, std::ptr::null_mut());
}

#[test]
fn test_unimplemented_serdata_ops() {
    unsafe {
        serdata_ops::to_ser::<crate::tests::topic::Data>(
            std::ptr::null(),
            0,
            0,
            std::ptr::null_mut(),
        );
    }

    let result = unsafe {
        serdata_ops::from_loaned_sample::<crate::tests::topic::Data, Serialized>(
            std::ptr::null(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
            false,
        )
    };
    assert_eq!(result, std::ptr::null_mut(),);

    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data, Serialized>(
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(result, std::ptr::null_mut(),);
}

#[test]
fn test_serdata_ops_from_keyhash() {
    use crate::cdr_bounds::Padding;
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default)]
    struct Data {
        x: u32,
        y: i32,
        message: String,
    }

    impl crate::Topicable for Data {
        type Key = MockedKey;

        fn from_key(key: &Self::Key) -> Self {
            Self {
                x: key.0,
                y: key.1,
                message: key.2.clone(),
            }
        }

        fn as_key(&self) -> Self::Key {
            MockedKey(self.x, self.y, self.message.clone())
        }
    }

    static MOCKED_MAX_SERIALIZED_CDR_SIZE: std::sync::Mutex<crate::cdr_bounds::CdrSize> =
        std::sync::Mutex::new(crate::cdr_bounds::CdrSize::Unbounded);

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Hash)]
    pub struct MockedKey(u32, i32, String);

    const INITIAL_CAPACITY: usize = 64;

    impl Default for MockedKey {
        fn default() -> Self {
            Self(0, 0, String::with_capacity(INITIAL_CAPACITY))
        }
    }

    impl crate::cdr_bounds::CdrBounds for MockedKey {
        fn max_serialized_cdr_size() -> crate::cdr_bounds::CdrSize {
            (*MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap()).with_padding(Self::alignment())
        }

        fn alignment() -> usize {
            <(u32, i32, String)>::alignment()
        }
    }

    let sample = Data {
        x: 101,
        y: 102,
        message: "a".to_string(),
    };

    let type_name = std::ffi::CString::new(Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<Data, Serialized>::new(
        &type_name,
        topic_has_key,
    ));

    let key = sample.as_key();

    let serialization = &mut [0; 16];
    cdr_encoding::to_writer::<_, byteorder::BigEndian, _>(&mut serialization[..], &key).unwrap();
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: *serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Bounded(15);

    let result = serdata_ops::from_keyhash_with_mode::<Data, Serialized>(&sertype, &keyhash, true);
    assert_eq!(result, std::ptr::null_mut());

    let result = serdata_ops::from_keyhash_with_mode::<Data, Serialized>(&sertype, &keyhash, false);
    assert_ne!(result, std::ptr::null_mut());

    let serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<Data>>()) };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Key);
    assert_eq!(serdata.key(), &key);
    // In-place decoding reuses the string allocated by the key's default constructor.
    assert!(serdata.key().2.capacity() >= INITIAL_CAPACITY);
    assert_eq!(serdata.sample(), &sample);
    assert_eq!(serdata.sample(), &Data::from_key(&key));

    let result = unsafe {
        serdata_ops::from_keyhash::<Data, Serialized>(&raw const sertype.inner, &raw const keyhash)
    };
    assert_ne!(result, std::ptr::null_mut());

    let serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<Data>>()) };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Key);
    assert_eq!(serdata.key(), &key);
    assert_eq!(serdata.sample(), &sample);
    assert_eq!(serdata.sample(), &Data::from_key(&key));

    let serialization = &mut [0; 16];
    cdr_encoding::to_writer::<_, byteorder::BigEndian, _>(&mut serialization[..], &key).unwrap();
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: *serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Unbounded;
    let result = unsafe {
        serdata_ops::from_keyhash::<Data, Serialized>(&raw const sertype.inner, &raw const keyhash)
    };
    assert_eq!(result, std::ptr::null_mut());

    let bad_serialization = [0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, u8::MAX, 97, 0, 0, 0];
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: bad_serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Bounded(15);
    let result = unsafe {
        serdata_ops::from_keyhash::<Data, Serialized>(&raw const sertype.inner, &raw const keyhash)
    };
    assert_eq!(result, std::ptr::null_mut());

    let bad_serialization = [0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, u8::MAX, 97, 0, 0, 0];
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: bad_serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Unbounded;
    let result = unsafe {
        serdata_ops::from_keyhash::<Data, Serialized>(&raw const sertype.inner, &raw const keyhash)
    };
    assert_eq!(result, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_untyped_to_sample() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let sample = crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: "hello".to_string(),
    };
    let serdata = crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_sample(sample.clone()),
    );

    let result = unsafe {
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            &raw const serdata.inner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(!result);

    let mut written_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let result = unsafe {
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            &raw const serdata.inner,
            (&raw mut written_sample).cast(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(result);
    assert_eq!(
        written_sample,
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(sample.clone())
    );

    let mut written_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let key = sample.as_key();
    let serdata = crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_key(key),
    );
    let result = unsafe {
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            &raw const serdata.inner,
            (&raw mut written_sample).cast(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(result);
    assert_eq!(
        written_sample,
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(key)
    );

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_to_sample() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let sample = crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: "hello".to_string(),
    };
    let serdata = crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_sample(sample.clone()),
    );

    let result = unsafe {
        serdata_ops::to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const serdata.inner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(!result);

    let mut written_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let result = unsafe {
        serdata_ops::to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const serdata.inner,
            (&raw mut written_sample).cast(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(result);
    assert_eq!(
        written_sample,
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(sample.clone())
    );

    let mut written_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let key = sample.as_key();
    let serdata = crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_key(key),
    );
    let result = unsafe {
        serdata_ops::to_sample::<crate::tests::topic::Data, Serialized>(
            &raw const serdata.inner,
            (&raw mut written_sample).cast(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(result);
    assert_eq!(
        written_sample,
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(key)
    );

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_sample() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    // Test serdata kind and sample kind mismatch.
    let kind = crate::internal::serdata::Kind::Key;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::SampleRef(
        &crate::tests::topic::Data::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let kind = crate::internal::serdata::Kind::Key;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(
        crate::tests::topic::Data::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::KeyRef(
        &Default::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let kind = crate::internal::serdata::Kind::Data;
    let sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(Default::default());
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    // Test serdata kind and sample type aligned.
    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::SampleRef(
        &crate::tests::topic::Data::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), kind);
    assert_eq!(serdata.sample(), &crate::tests::topic::Data::default());
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(
        crate::tests::topic::Data::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), kind);
    assert_eq!(serdata.sample(), &crate::tests::topic::Data::default());
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);

    let kind = crate::internal::serdata::Kind::Key;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::KeyRef(
        &Default::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), kind);
    assert_eq!(serdata.key(), &Default::default());
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);

    let kind = crate::internal::serdata::Kind::Key;
    let sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(Default::default());
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw const sample).cast(),
        )
    };
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    assert_eq!(serdata.kind(), kind);
    assert_eq!(serdata.key(), &Default::default());
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_ser() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            std::ptr::null(),
            0,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        min: 100,
        ..Default::default()
    };
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            cyclonedds_sys::ddsi_serdata_kind::MAX,
            &raw const fragment_chain,
            0,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        min: 100,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            0,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        maxp1: 0,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            0,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        maxp1: 100,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            0,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        maxp1: 0,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            10,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let mut subfragment = cyclonedds_sys::ddsi_rdata {
        ..Default::default()
    };
    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        nextfrag: &raw mut subfragment,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            10,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let mut subfragment = cyclonedds_sys::ddsi_rdata {
        min: 100,
        ..Default::default()
    };
    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        nextfrag: &raw mut subfragment,
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            10,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let mut subfragment = cyclonedds_sys::ddsi_rdata {
        ..Default::default()
    };
    let mut rmsg = [cyclonedds_sys::ddsi_rmsg::default(); 2];
    let fragment_chain = cyclonedds_sys::ddsi_rdata {
        nextfrag: &raw mut subfragment,
        maxp1: 1,
        rmsg: rmsg.as_mut_ptr(),
        ..Default::default()
    };
    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            &raw const fragment_chain,
            10,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_from_ser_iov() {
    use crate::internal::traits::CdrHeader;

    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let kind = crate::internal::serdata::Kind::Data;
    let size = 0;
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![];
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let size = 0;
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![];
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            cyclonedds_sys::ddsi_serdata_kind::MAX,
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::tests::topic::Data {
        x: 101,
        y: 102,
        message: "hello".to_string(),
    };
    let serialized_sample: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap())
        .collect();

    let size = serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::tests::topic::Data {
        x: 101,
        y: 102,
        message: "hello".to_string(),
    };
    let big_endian_serialized_sample: Vec<_> = byteorder::BigEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::BigEndian>(&sample).unwrap())
        .collect();

    let size = big_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![big_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::tests::topic::Data {
        x: 101,
        y: 102,
        message: "hello".to_string(),
    };
    let little_endian_serialized_sample: Vec<_> = byteorder::LittleEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::LittleEndian>(&sample).unwrap())
        .collect();

    let size = little_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![little_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::tests::topic::Data {
        x: 101,
        y: 102,
        message: "hello".to_string(),
    };
    let unknown_endian_serialized_sample: Vec<_> = [u8::MAX, u8::MAX, u8::MAX, u8::MAX]
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap())
        .collect();

    let size = unknown_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![unknown_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Key;
    let sample = (101, 102);
    let serialized_sample: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap())
        .collect();

    let size = serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Data;
    let sample = crate::tests::topic::Data {
        x: 101,
        y: 102,
        message: "hello".to_string(),
    };
    let mut serialized_sample: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap())
        .collect();
    let size = serialized_sample.len();

    serialized_sample.resize(size + 20, 0);

    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![
        serialized_sample[0..10].to_vec(),
        serialized_sample[10..].to_vec(),
    ]
    .into_iter()
    .map(|container: Vec<u8>| {
        assert_eq!(container.len(), container.capacity());
        let (iov_base, iov_len, _) = container.into_raw_parts();
        let iov_base = iov_base.cast();
        let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
        cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
    })
    .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Key;
    let sample = (101, 102);
    let mut serialized_sample: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap())
        .collect();

    let size = serialized_sample.len();

    serialized_sample.resize(size + 20, 0);

    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![
        serialized_sample[0..10].to_vec(),
        serialized_sample[10..].to_vec(),
    ]
    .into_iter()
    .map(|container: Vec<u8>| {
        assert_eq!(container.len(), container.capacity());
        let (iov_base, iov_len, _) = container.into_raw_parts();
        let iov_base = iov_base.cast();
        let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
        cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
    })
    .collect();

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_ne!(serdata, std::ptr::null_mut());
    let serdata = unsafe {
        &mut *(serdata.cast::<crate::internal::serdata::Serdata<crate::tests::topic::Data>>())
    };
    crate::internal::ffi::ddsi_serdata_unref(&mut serdata.inner);
    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Data;
    let bad_serialization = [0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, u8::MAX, 97, 0, 0, 0];
    let serialized_sample: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(bad_serialization)
        .collect();

    let size = serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Key;
    let bad_serialization = [u8::MAX, u8::MAX, u8::MAX, u8::MAX];
    let big_endian_serialized_sample: Vec<_> = byteorder::BigEndian::cdr_header()
        .into_iter()
        .chain(bad_serialization)
        .collect();

    let size = big_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![big_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Key;
    let bad_serialization = [u8::MAX, u8::MAX, u8::MAX, u8::MAX];
    let little_endian_serialized_sample: Vec<_> = byteorder::LittleEndian::cdr_header()
        .into_iter()
        .chain(bad_serialization)
        .collect();

    let size = little_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![little_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let kind = crate::internal::serdata::Kind::Key;
    let bad_serialization = [u8::MAX, u8::MAX, u8::MAX, u8::MAX];
    let unknown_endian_serialized_sample: Vec<_> = [u8::MAX, u8::MAX, u8::MAX, u8::MAX]
        .into_iter()
        .chain(bad_serialization)
        .collect();

    let size = unknown_endian_serialized_sample.len();
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![unknown_endian_serialized_sample]
        .into_iter()
        .map(|container: Vec<u8>| {
            assert_eq!(container.len(), container.capacity());
            let (iov_base, iov_len, _) = container.into_raw_parts();
            let iov_base = iov_base.cast();
            let iov_len = iov_len as cyclonedds_sys::ddsrt_iov_len_t;
            cyclonedds_sys::ddsrt_iovec_t { iov_base, iov_len }
        })
        .collect();
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            containers_len,
            containers.as_ptr(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    for container in containers {
        let base = container.iov_base;
        let len = container.iov_len as usize;
        // NOTE: the capacity and the len where asserted to be the same on
        // construction.
        let capacity = container.iov_len as usize;
        unsafe {
            Vec::from_raw_parts(base, len, capacity);
        }
    }

    let sample = crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: "hello".to_string(),
    };
    let mut serdata = Box::new(crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_sample(sample.clone()),
    ));

    let mut container = cyclonedds_sys::ddsrt_iovec_t::default();

    let result = unsafe {
        serdata_ops::to_ser_ref::<crate::tests::topic::Data>(
            &raw mut serdata.inner,
            usize::MAX,
            22,
            &raw mut container,
        )
    };

    assert_eq!(result, std::ptr::null_mut());

    let sample = crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: "hello".to_string(),
    };
    let mut serdata = Box::new(crate::internal::serdata::Serdata::new(
        &sertype,
        crate::sample::SampleOrKeyInner::new_sample(sample.clone()),
    ));

    let mut container = cyclonedds_sys::ddsrt_iovec_t::default();

    let result = unsafe {
        serdata_ops::to_ser_ref::<crate::tests::topic::Data>(
            &raw mut serdata.inner,
            22,
            22,
            &raw mut container,
        )
    };

    assert_ne!(result, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_to_ser_ref_fragments() {
    use crate::internal::traits::CdrHeader;

    const FRAGMENT_SIZE: usize = 1344;

    let name = std::ffi::CString::new("FragmentValidation").unwrap();
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&name, crate::tests::topic::Data::IS_KEYED));

    for message_size in [41 * 1024, 164 * 1024] {
        let sample = crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: "x".repeat(message_size),
        };
        let mut expected = byteorder::NativeEndian::cdr_header().to_vec();
        expected.extend(cdr_encoding::to_vec::<_, byteorder::NativeEndian>(&sample).unwrap());
        assert_ne!(expected.len() % 4, 0);
        expected.resize(expected.len().next_multiple_of(4), 0);

        // Exercise padding both before and after the serialized buffer is cached.
        for last_fragment_first in [false, true] {
            let mut serdata = Box::new(crate::internal::serdata::Serdata::new(
                &sertype,
                SampleOrKeyInner::new_sample(sample.clone()),
            ));
            let mut offsets: Vec<_> = (0..expected.len()).step_by(FRAGMENT_SIZE).collect();
            if last_fragment_first {
                offsets.reverse();
            }
            for offset in offsets {
                let size = FRAGMENT_SIZE.min(expected.len() - offset);
                let mut container = cyclonedds_sys::ddsrt_iovec_t::default();
                let result = unsafe {
                    serdata_ops::to_ser_ref::<crate::tests::topic::Data>(
                        &raw mut serdata.inner,
                        offset,
                        size,
                        &raw mut container,
                    )
                };
                assert!(!result.is_null());
                assert_eq!(container.iov_len as usize, size);
                let bytes =
                    unsafe { std::slice::from_raw_parts(container.iov_base.cast::<u8>(), size) };
                assert_eq!(bytes, &expected[offset..offset + size]);
                unsafe {
                    serdata_ops::to_ser_unref::<crate::tests::topic::Data>(
                        result,
                        &raw const container,
                    );
                }
            }
        }
    }

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_realloc_samples() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let mut pointers = [std::ptr::null_mut(); 10];
    let old_samples = std::ptr::null_mut();
    let old_count = 0;
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            old_samples,
            old_count,
            new_count,
        );
    }

    for pointer in pointers {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &crate::internal::ffi::InternalSample::None);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    let mut pointers = [std::ptr::null_mut(); 10];
    let sample = crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: String::new(),
    });
    let mut old_samples = [const {
        crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: String::new(),
        })
    }; 10];
    let old_count = old_samples.len();
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            old_samples.as_mut_ptr().cast(),
            old_count,
            new_count,
        );
    }

    for pointer in pointers {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &sample);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    let mut pointers = [std::ptr::null_mut(); 10];
    let sample = crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: String::new(),
    });
    let mut old_samples = [const {
        crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: String::new(),
        })
    }; 8];
    let old_count = old_samples.len();
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            old_samples.as_mut_ptr().cast(),
            old_count,
            new_count,
        );
    }

    for &pointer in &pointers[0..8] {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &sample);
    }

    for &pointer in &pointers[8..] {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &crate::internal::ffi::InternalSample::None);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    let mut pointers = [std::ptr::null_mut(); 6];
    let sample = crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
        x: 101,
        y: 202,
        message: String::new(),
    });
    let mut old_samples = [const {
        crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: String::new(),
        })
    }; 8];
    let old_count = old_samples.len();
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            old_samples.as_mut_ptr().cast(),
            old_count,
            new_count,
        );
    }

    for &pointer in &pointers[0..6] {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &sample);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    let mut pointers = [std::ptr::null_mut(); 6];
    let mut old_samples = [const {
        crate::internal::ffi::InternalSample::Sample(crate::tests::topic::Data {
            x: 101,
            y: 202,
            message: String::new(),
        })
    }; 1];
    let old_count = 0;
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw mut sertype.inner,
            old_samples.as_mut_ptr().cast(),
            old_count,
            new_count,
        );
    }

    for pointer in pointers {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &crate::internal::ffi::InternalSample::None);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_free_samples_unknown_free_operation() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let mut pointers = [std::ptr::null_mut(); 10];
    let old_samples = std::ptr::null_mut();
    let old_count = 0;
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data, Serialized>(
            pointers.as_mut_ptr(),
            &raw const sertype.inner,
            old_samples,
            old_count,
            new_count,
        );
    }

    for pointer in pointers {
        assert_ne!(pointer, std::ptr::null_mut());

        let pointer = unsafe {
            &*(pointer
                .cast::<crate::internal::ffi::InternalSample<'_, crate::tests::topic::Data>>())
        };
        assert_eq!(pointer, &crate::internal::ffi::InternalSample::None);
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            !(cyclonedds_sys::DDS_FREE_ALL_BIT | cyclonedds_sys::DDS_FREE_CONTENTS_BIT)
                as cyclonedds_sys::dds_free_op_t,
        );
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            cyclonedds_sys::DDS_FREE_ALL_BIT as cyclonedds_sys::dds_free_op_t,
        );
    }

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_sertype_ops_serialize_into() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let mut destination_buffer: Vec<u8> = Vec::new();
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            cyclonedds_sys::ddsi_serdata_kind::MAX,
            std::ptr::null(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let mut sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE];
    let kind = crate::internal::serdata::Kind::Data;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let mut sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::None;
    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE];
    let kind = crate::internal::serdata::Kind::Data;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let mut sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(
        crate::tests::topic::Data::default(),
    );
    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE];
    let kind = crate::internal::serdata::Kind::Data;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let mut sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(Default::default());
    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE];
    let kind = crate::internal::serdata::Kind::Key;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE];
    let kind = crate::internal::serdata::Kind::Key;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            std::ptr::null(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    // NOTE: `serialize_into` writes the CDR payload only. The four byte
    // encapsulation header is reported separately, through
    // `get_serialized_size`, and Cyclone DDS stores it out-of-band in the PSMX
    // loan metadata rather than in the payload.
    let sample = crate::tests::topic::Data::default();
    let mut serialized: Vec<u8> = vec![0; 13];
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut serialized[..], &sample).unwrap();

    let mut internal_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(sample);

    let mut destination_buffer: Vec<u8> = vec![0; 13];
    let kind = crate::internal::serdata::Kind::Data;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut internal_sample).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(result);
    assert_eq!(serialized, destination_buffer);

    let key = (100, 100);

    let mut serialized: Vec<u8> = vec![0; 8];
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(&mut serialized[..], &key).unwrap();

    let mut internal_key =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(key);

    let mut destination_buffer: Vec<u8> = vec![0; 8];
    let kind = crate::internal::serdata::Kind::Key;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data, Serialized>(
            &raw const sertype.inner,
            kind.into(),
            (&raw mut internal_key).cast(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(result);
    assert_eq!(serialized, destination_buffer);

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_sertype_version_conversion() {
    let version = crate::internal::ffi::sertype_ops::SertypeVersion::V0
        .as_ffi()
        .unwrap();

    let version = version as usize;

    let expected = cyclonedds_sys::ddsi_sertype_v0 as *const () as usize;

    assert_eq!(version, expected);
}

#[test]
#[cfg(target_os = "windows")]
fn test_sertype_version_conversion() {
    let version = crate::internal::ffi::sertype_ops::SertypeVersion::V0
        .as_ffi()
        .unwrap();

    let version = usize::from(version);

    let expected = 0x01;

    assert_eq!(version, expected);
}

#[test]
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn test_from_ser_iov_with_msg_iovlen_t_out_of_bounds() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<
        crate::tests::topic::Data,
        Serialized,
    >::new(&type_name, topic_has_key));

    let kind = crate::internal::serdata::Kind::Data;
    let size = 0;

    let containers_len: i32 = -1;
    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data, Serialized>(
            &raw mut sertype.inner,
            kind.into(),
            containers_len,
            std::ptr::null_mut(),
            size,
        )
    };
    assert_eq!(serdata, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
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

/// Build a sertype for the shared test topic, along with its type name.
///
/// Boxed because the tests hand its address to the ops and then release the
/// Cyclone DDS reference by hand, which means suppressing the `Drop` that would
/// otherwise release it a second time.
#[allow(
    clippy::unnecessary_box_returns,
    reason = "the tests leak the box deliberately, in place of running `Sertype`'s `Drop`"
)]
fn test_sertype() -> Box<crate::internal::sertype::Sertype<crate::tests::topic::Data, Serialized>> {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();

    Box::new(crate::internal::sertype::Sertype::new(
        &type_name,
        crate::tests::topic::Data::IS_KEYED,
    ))
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
