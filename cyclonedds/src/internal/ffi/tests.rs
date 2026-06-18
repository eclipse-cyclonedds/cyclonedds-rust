use super::*;
use crate::Topicable;
use crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE;
use crate::sample::SampleOrKeyInner;

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
            ..crate::internal::sertype::Sertype::<crate::tests::topic::Data>::SERTYPE_OPS
        },
        &crate::internal::sertype::Sertype::<crate::tests::topic::Data>::SERDATA_OPS,
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
            ..crate::internal::sertype::Sertype::<crate::tests::topic::Data>::SERTYPE_OPS
        },
        &crate::internal::sertype::Sertype::<crate::tests::topic::Data>::SERDATA_OPS,
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            topic_name,
            crate::tests::topic::Data::IS_KEYED,
        ),
    );

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

    let result = unsafe {
        sertype_ops::get_serialized_size::<crate::tests::topic::Data>(
            std::ptr::null(),
            Default::default(),
            std::ptr::null(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(result, 0);

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
        serdata_ops::from_loaned_sample::<crate::tests::topic::Data>(
            std::ptr::null(),
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
            false,
        )
    };
    assert_eq!(result, std::ptr::null_mut(),);

    let result = unsafe {
        serdata_ops::from_psmx::<crate::tests::topic::Data>(std::ptr::null(), std::ptr::null_mut())
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

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default, Hash)]
    pub struct MockedKey(u32, i32, String);

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
    let mut sertype = Box::new(crate::internal::sertype::Sertype::<Data>::new(
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

    let result = serdata_ops::from_keyhash_with_mode::<Data>(&sertype, &keyhash, true);
    assert_eq!(result, std::ptr::null_mut());

    let result = serdata_ops::from_keyhash_with_mode::<Data>(&sertype, &keyhash, false);
    assert_ne!(result, std::ptr::null_mut());

    let mut serdata =
        unsafe { Box::from_raw(result.cast::<crate::internal::serdata::Serdata<Data>>()) };
    assert_eq!(serdata.kind(), crate::internal::serdata::Kind::Key);
    assert_eq!(serdata.key(), &key);
    assert_eq!(serdata.sample(), &sample);
    assert_eq!(serdata.sample(), &Data::from_key(&key));

    let result =
        unsafe { serdata_ops::from_keyhash::<Data>(&raw const sertype.inner, &raw const keyhash) };
    assert_ne!(result, std::ptr::null_mut());

    let mut serdata =
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
    let result =
        unsafe { serdata_ops::from_keyhash::<Data>(&raw const sertype.inner, &raw const keyhash) };
    assert_eq!(result, std::ptr::null_mut());

    let bad_serialization = [0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, u8::MAX, 97, 0, 0, 0];
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: bad_serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Bounded(15);
    let result =
        unsafe { serdata_ops::from_keyhash::<Data>(&raw const sertype.inner, &raw const keyhash) };
    assert_eq!(result, std::ptr::null_mut());

    let bad_serialization = [0, 0, 0, 101, 0, 0, 0, 102, 0, 0, 0, u8::MAX, 97, 0, 0, 0];
    let keyhash = cyclonedds_sys::ddsi_keyhash {
        value: bad_serialization,
    };
    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = crate::cdr_bounds::CdrSize::Unbounded;
    let result =
        unsafe { serdata_ops::from_keyhash::<Data>(&raw const sertype.inner, &raw const keyhash) };
    assert_eq!(result, std::ptr::null_mut());

    crate::internal::ffi::ddsi_sertype_unref(&mut sertype.inner);
    let _ = Box::into_raw(sertype);
}

#[test]
fn test_serdata_ops_untyped_to_sample() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

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
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data>(
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
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data>(
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
        serdata_ops::untyped_to_sample::<crate::tests::topic::Data>(
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

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
        serdata_ops::to_sample::<crate::tests::topic::Data>(
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
        serdata_ops::to_sample::<crate::tests::topic::Data>(
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
        serdata_ops::to_sample::<crate::tests::topic::Data>(
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    // Test serdata kind and sample kind mismatch.
    let kind = crate::internal::serdata::Kind::Key;
    let sample = crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::SampleRef(
        &crate::tests::topic::Data::default(),
    );
    let serdata = unsafe {
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
        serdata_ops::from_sample::<crate::tests::topic::Data>(
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    let kind = crate::internal::serdata::Kind::Data;
    let serdata = unsafe {
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser::<crate::tests::topic::Data>(
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    let kind = crate::internal::serdata::Kind::Data;
    let size = 0;
    let containers: Vec<cyclonedds_sys::ddsrt_iovec_t> = vec![];
    let containers_len = containers.len() as cyclonedds_sys::ddsrt_msg_iovlen_t;

    let serdata = unsafe {
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
        serdata_ops::from_ser_iov::<crate::tests::topic::Data>(
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
            1000,
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
fn test_sertype_ops_realloc_samples() {
    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    let mut pointers = [std::ptr::null_mut(); 10];
    let old_samples = std::ptr::null_mut();
    let old_count = 0;
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    let mut pointers = [std::ptr::null_mut(); 10];
    let old_samples = std::ptr::null_mut();
    let old_count = 0;
    let new_count = pointers.len();

    unsafe {
        sertype_ops::realloc_samples::<crate::tests::topic::Data>(
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
        sertype_ops::free_samples::<crate::tests::topic::Data>(
            &raw const sertype.inner,
            pointers.as_mut_ptr(),
            pointers.len(),
            !(cyclonedds_sys::DDS_FREE_ALL_BIT | cyclonedds_sys::DDS_FREE_CONTENTS_BIT)
                as cyclonedds_sys::dds_free_op_t,
        );
    }

    unsafe {
        sertype_ops::free_samples::<crate::tests::topic::Data>(
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
    use crate::internal::traits::CdrHeader;

    let type_name =
        std::ffi::CString::new(crate::tests::topic::Data::dds_type_name().as_ref()).unwrap();
    let topic_has_key = crate::tests::topic::Data::IS_KEYED;
    let mut sertype = Box::new(
        crate::internal::sertype::Sertype::<crate::tests::topic::Data>::new(
            &type_name,
            topic_has_key,
        ),
    );

    let mut destination_buffer: Vec<u8> = Vec::new();
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
            &raw const sertype.inner,
            kind.into(),
            std::ptr::null(),
            destination_buffer.as_mut_ptr().cast(),
            destination_buffer.len(),
        )
    };
    assert!(!result);

    let sample = crate::tests::topic::Data::default();
    let mut serialized: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(vec![0; 13])
        .collect();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(
        &mut serialized[DDSI_RTPS_HEADER_SIZE..],
        &sample,
    )
    .unwrap();

    let mut internal_sample =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Sample(sample);

    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE + 13];
    let kind = crate::internal::serdata::Kind::Data;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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

    let mut serialized: Vec<_> = byteorder::NativeEndian::cdr_header()
        .into_iter()
        .chain(vec![0; 8])
        .collect();
    cdr_encoding::to_writer::<_, byteorder::NativeEndian, _>(
        &mut serialized[DDSI_RTPS_HEADER_SIZE..],
        &key,
    )
    .unwrap();

    let mut internal_key =
        crate::internal::ffi::InternalSample::<crate::tests::topic::Data>::Key(key);

    let mut destination_buffer: Vec<u8> =
        vec![0; crate::internal::ffi::serdata_ops::DDSI_RTPS_HEADER_SIZE + 8];
    let kind = crate::internal::serdata::Kind::Key;
    let result = unsafe {
        sertype_ops::serialize_into::<crate::tests::topic::Data>(
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
