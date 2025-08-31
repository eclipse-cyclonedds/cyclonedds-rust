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
