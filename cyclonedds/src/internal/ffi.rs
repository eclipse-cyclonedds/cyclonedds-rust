//! Safe and ergonomic wrappers for the functionality from [`cyclonedds_sys`].
//!
//! This module is exposed to make it easier for applications to poke at the
//! low-level implementation details of Cyclone DDS.

#![allow(unsafe_code)]

mod listener;
pub mod serdata_ops;
pub mod sertype_ops;

pub use listener::{
    Listener, dds_create_listener, dds_delete_listener, dds_listener_set_data_available,
    dds_listener_set_data_on_readers, dds_listener_set_inconsistent_topic,
    dds_listener_set_liveliness_changed, dds_listener_set_liveliness_lost,
    dds_listener_set_offered_deadline_missed, dds_listener_set_offered_incompatible_qos,
    dds_listener_set_publication_matched, dds_listener_set_requested_deadline_missed,
    dds_listener_set_requested_incompatible_qos, dds_listener_set_sample_lost,
    dds_listener_set_sample_rejected, dds_listener_set_subscription_matched, dds_set_listener,
};

use crate::Result;
use crate::error::IntoError;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InternalSample<'a, T>
where
    T: crate::Topicable,
{
    None,
    SampleRef(&'a T),
    KeyRef(&'a T::Key),
    Sample(T),
    Key(T::Key),
}

pub fn dds_get_instance_handle(
    entity: cyclonedds_sys::dds_entity_t,
) -> Result<cyclonedds_sys::dds_instance_handle_t> {
    let mut handle = 0;
    unsafe { cyclonedds_sys::dds_get_instance_handle(entity, &raw mut handle) }.into_error()?;
    Ok(handle)
}

pub fn dds_get_status_changes(entity: cyclonedds_sys::dds_entity_t) -> Result<u32> {
    let mut status = 0;
    unsafe { cyclonedds_sys::dds_get_status_changes(entity, &raw mut status) }.into_error()?;
    Ok(status)
}

pub fn dds_read_status(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<u32> {
    let mut status: u32 = 0;
    unsafe { cyclonedds_sys::dds_read_status(entity, &raw mut status, mask) }.into_error()?;
    Ok(status)
}

pub fn dds_take_status(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<u32> {
    let mut status: u32 = 0;
    unsafe { cyclonedds_sys::dds_take_status(entity, &raw mut status, mask) }.into_error()?;
    Ok(status)
}

pub fn dds_get_status_mask(entity: cyclonedds_sys::dds_entity_t) -> Result<u32> {
    let mut mask: u32 = 0;
    unsafe { cyclonedds_sys::dds_get_status_mask(entity, &raw mut mask) }.into_error()?;
    Ok(mask)
}

pub fn dds_set_status_mask(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<()> {
    unsafe { cyclonedds_sys::dds_set_status_mask(entity, mask) }.into_error()?;
    Ok(())
}

/// Create a new serdata. This is primarily used by the
/// [`Serdata`][`crate::internal::serdata::Serdata`] wrapper.
pub fn ddsi_serdata_new(
    sertype: &cyclonedds_sys::ddsi_sertype,
    kind: cyclonedds_sys::ddsi_serdata_kind,
) -> cyclonedds_sys::ddsi_serdata {
    let mut serdata = cyclonedds_sys::ddsi_serdata::default();

    unsafe { cyclonedds_sys::ddsi_serdata_init(&raw mut serdata, sertype, kind) }
    serdata
}

/// Increment the reference count of a serdata. This is primarily used by the
/// [`Serdata`][`crate::internal::serdata::Serdata`] wrapper.
#[allow(unused)]
#[inline]
pub fn ddsi_serdata_ref(serdata: &mut cyclonedds_sys::ddsi_serdata) {
    unsafe { cyclonedds_sys::ddsi_serdata_ref(serdata) };
}

/// Decrement the reference count of a serdata. This is primarily used by the
/// [`Serdata`][`crate::internal::serdata::Serdata`] wrapper.
#[inline]
pub fn ddsi_serdata_unref(serdata: &mut cyclonedds_sys::ddsi_serdata) {
    unsafe { cyclonedds_sys::ddsi_serdata_unref(serdata) };
}

/// Create a new sertype. This is primarily used by the
/// [`Sertype`][`crate::internal::sertype::Sertype`] wrapper.
pub fn ddsi_sertype_new(
    type_name: &std::ffi::CStr,
    sertype_ops: &cyclonedds_sys::ddsi_sertype_ops,
    serdata_ops: &cyclonedds_sys::ddsi_serdata_ops,
    topic_has_key: bool,
) -> cyclonedds_sys::ddsi_sertype {
    let mut sertype = cyclonedds_sys::ddsi_sertype::default();

    unsafe {
        cyclonedds_sys::ddsi_sertype_init(
            &raw mut sertype,
            type_name.as_ptr(),
            sertype_ops,
            serdata_ops,
            // the interface is based around topic_kind_no_key so invert here.
            !topic_has_key,
        );
    };

    sertype
}

/// Increment the reference count of a sertype. This is primarily used by the
/// [`Sertype`][`crate::internal::sertype::Sertype`] wrapper.
#[inline]
#[allow(unused)]
pub fn ddsi_sertype_ref(sertype: &mut cyclonedds_sys::ddsi_sertype) {
    unsafe { cyclonedds_sys::ddsi_sertype_ref(sertype) };
}

/// Decrement the reference count of a sertype. This is primarily used by the
/// [`Sertype`][`crate::internal::sertype::Sertype`] wrapper.
#[inline]
pub fn ddsi_sertype_unref(sertype: &mut cyclonedds_sys::ddsi_sertype) {
    unsafe { cyclonedds_sys::ddsi_sertype_unref(sertype) };
}

/// Finalize a sertype. This is primarily used by the
/// [`Sertype`][`crate::internal::sertype::Sertype`] wrapper.
pub fn ddsi_sertype_fini(sertype: &mut cyclonedds_sys::ddsi_sertype) {
    unsafe {
        cyclonedds_sys::ddsi_sertype_fini(sertype);
    }
}

/// Delete an entity.
pub fn dds_delete(entity: cyclonedds_sys::dds_entity_t) -> Result<()> {
    unsafe { cyclonedds_sys::dds_delete(entity) }.into_error()?;
    Ok(())
}

/// Create a domain. This is primarily used by the
/// [`Domain`][`crate::Domain`] wrapper.
pub fn dds_create_domain(
    domain_id: cyclonedds_sys::dds_domainid_t,
) -> Result<cyclonedds_sys::dds_entity_t> {
    Ok(
        unsafe { cyclonedds_sys::dds_create_domain(domain_id, std::ptr::null()) }.into_error()?
            as _,
    )
}

/// Create a domain with a specific XML config. This is primarily used by the
/// [`Domain`][`crate::Domain`] wrapper.
pub fn dds_create_domain_with_config(
    domain_id: cyclonedds_sys::dds_domainid_t,
    config: &std::ffi::CStr,
) -> Result<cyclonedds_sys::dds_entity_t> {
    Ok(unsafe { cyclonedds_sys::dds_create_domain(domain_id, config.as_ptr()) }.into_error()? as _)
}

pub fn dds_reset_qos(qos: &mut cyclonedds_sys::dds_qos_t) {
    unsafe {
        cyclonedds_sys::dds_reset_qos(qos);
    }
}

pub fn dds_qos_set_user_data(qos: &mut cyclonedds_sys::dds_qos_t, user_data: &[u8]) {
    unsafe {
        cyclonedds_sys::dds_qset_userdata(qos, user_data.as_ptr().cast(), user_data.len());
    }
}

pub fn dds_qos_set_topic_data(qos: &mut cyclonedds_sys::dds_qos_t, topic_data: &[u8]) {
    unsafe {
        cyclonedds_sys::dds_qset_userdata(qos, topic_data.as_ptr().cast(), topic_data.len());
    }
}

pub fn dds_qos_set_group_data(qos: &mut cyclonedds_sys::dds_qos_t, group_data: &[u8]) {
    unsafe {
        cyclonedds_sys::dds_qset_userdata(qos, group_data.as_ptr().cast(), group_data.len());
    }
}

pub fn dds_qos_set_durability(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_durability_kind_t,
) {
    unsafe { cyclonedds_sys::dds_qset_durability(qos, kind) }
}

pub fn dds_qos_set_durability_service(
    qos: &mut cyclonedds_sys::dds_qos_t,
    service_cleanup_delay: cyclonedds_sys::dds_duration_t,
    history_kind: cyclonedds_sys::dds_history_kind_t,
    history_depth: i32,
    max_samples: i32,
    max_instances: i32,
    max_samples_per_instance: i32,
) {
    unsafe {
        cyclonedds_sys::dds_qset_durability_service(
            qos,
            service_cleanup_delay,
            history_kind,
            history_depth,
            max_samples,
            max_instances,
            max_samples_per_instance,
        );
    }
}

pub fn dds_qos_set_presentation(
    qos: &mut cyclonedds_sys::dds_qos_t,
    access_scope: cyclonedds_sys::dds_presentation_access_scope_kind,
    coherent_access: bool,
    ordered_access: bool,
) {
    unsafe {
        cyclonedds_sys::dds_qset_presentation(qos, access_scope, coherent_access, ordered_access);
    }
}

pub fn dds_qos_set_deadline(
    qos: &mut cyclonedds_sys::dds_qos_t,
    deadline: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_deadline(qos, deadline) }
}

pub fn dds_qos_set_latency_budget(
    qos: &mut cyclonedds_sys::dds_qos_t,
    duration: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_latency_budget(qos, duration) }
}

pub fn dds_qos_set_ownership(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_ownership_kind_t,
) {
    unsafe { cyclonedds_sys::dds_qset_ownership(qos, kind) }
}

pub fn dds_qos_set_ownership_strength(qos: &mut cyclonedds_sys::dds_qos_t, value: i32) {
    unsafe { cyclonedds_sys::dds_qset_ownership_strength(qos, value) }
}

pub fn dds_qos_set_liveliness(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_liveliness_kind_t,
    lease_duration: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_liveliness(qos, kind, lease_duration) }
}

pub fn dds_qos_set_time_based_filter(
    qos: &mut cyclonedds_sys::dds_qos_t,
    minimum_separation: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_time_based_filter(qos, minimum_separation) }
}

pub fn dds_qos_set_partition(
    qos: &mut cyclonedds_sys::dds_qos_t,
    partitions: &[std::ffi::CString],
) {
    let n = u32::try_from(partitions.len())
        .expect("too many partitions supplied (DDS limits the number of partitions to u32::MAX)");
    let mut ps: Vec<_> = partitions.iter().map(|str| str.as_ptr()).collect();
    unsafe { cyclonedds_sys::dds_qset_partition(qos, n, ps.as_mut_ptr()) }
}

pub fn dds_qos_set_reliability(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_reliability_kind_t,
    max_blocking_time: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_reliability(qos, kind, max_blocking_time) }
}

pub fn dds_qos_set_transport_priority(qos: &mut cyclonedds_sys::dds_qos_t, value: i32) {
    unsafe { cyclonedds_sys::dds_qset_transport_priority(qos, value) }
}

pub fn dds_qos_set_lifespan(
    qos: &mut cyclonedds_sys::dds_qos_t,
    lifespan: cyclonedds_sys::dds_duration_t,
) {
    unsafe { cyclonedds_sys::dds_qset_lifespan(qos, lifespan) }
}

pub fn dds_qos_set_destination_order(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_destination_order_kind_t,
) {
    unsafe { cyclonedds_sys::dds_qset_destination_order(qos, kind) }
}

pub fn dds_qos_set_history(
    qos: &mut cyclonedds_sys::dds_qos_t,
    kind: cyclonedds_sys::dds_history_kind_t,
    depth: i32,
) {
    unsafe { cyclonedds_sys::dds_qset_history(qos, kind, depth) }
}

pub fn dds_qos_set_resource_limits(
    qos: &mut cyclonedds_sys::dds_qos_t,
    max_samples: i32,
    max_instances: i32,
    max_samples_per_instance: i32,
) {
    unsafe {
        cyclonedds_sys::dds_qset_resource_limits(
            qos,
            max_samples,
            max_instances,
            max_samples_per_instance,
        );
    }
}

pub const fn dds_qos_set_entity_factory(
    qos: &mut cyclonedds_sys::dds_qos_t,
    autoenable_created_entities: bool,
) {
    // FIXME? there's no associated dds_qset_?
    qos.entity_factory.autoenable_created_entities =
        if autoenable_created_entities { 1 } else { 0 };
}

pub fn dds_qos_set_writer_data_lifecycle(qos: &mut cyclonedds_sys::dds_qos_t, autodispose: bool) {
    unsafe { cyclonedds_sys::dds_qset_writer_data_lifecycle(qos, autodispose) }
}

pub fn dds_qos_set_reader_data_lifecycle(
    qos: &mut cyclonedds_sys::dds_qos_t,
    autopurge_nowriter_samples_delay: cyclonedds_sys::dds_duration_t,
    autopurge_disposed_samples_delay: cyclonedds_sys::dds_duration_t,
) {
    unsafe {
        cyclonedds_sys::dds_qset_reader_data_lifecycle(
            qos,
            autopurge_nowriter_samples_delay,
            autopurge_disposed_samples_delay,
        );
    }
}

pub fn dds_qos_set_entity_name(qos: &mut cyclonedds_sys::dds_qos_t, name: &std::ffi::CStr) {
    unsafe {
        cyclonedds_sys::dds_qset_entity_name(qos, name.as_ptr());
    }
}

/// Create a participant within a domain. This is primarily used by the
/// [`Participant`][`crate::Participant`] wrapper.
pub fn dds_create_participant(
    domain: cyclonedds_sys::dds_domainid_t,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe {
        cyclonedds_sys::dds_create_participant(
            domain,
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
        )
    }
    .into_error()
}

/// Create a topic under a participant. This is primarily used by the
/// [`Topic`][`crate::Topic`] wrapper.
#[allow(clippy::mut_mut)]
pub fn dds_create_topic(
    participant: cyclonedds_sys::dds_entity_t,
    name: &std::ffi::CStr,
    sertype: &mut &mut cyclonedds_sys::ddsi_sertype,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    let sedp_plist = std::ptr::null();

    unsafe {
        cyclonedds_sys::dds_create_topic_sertype(
            participant,
            name.as_ptr(),
            std::ptr::from_mut::<&mut _>(sertype).cast::<*mut _>(),
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
            sedp_plist,
        )
    }
    .into_error()
}

/// Create a publisher under a participant. This is primarily used by the
/// [`Publisher`][`crate::Publisher`] wrapper.
pub fn dds_create_publisher(
    participant: cyclonedds_sys::dds_entity_t,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe {
        cyclonedds_sys::dds_create_publisher(
            participant,
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
        )
    }
    .into_error()
}

pub fn dds_suspend(publisher: cyclonedds_sys::dds_entity_t) -> Result<()> {
    unsafe { cyclonedds_sys::dds_suspend(publisher) }
        .into_error()
        .map(|_| ())
}

pub fn dds_resume(publisher: cyclonedds_sys::dds_entity_t) -> Result<()> {
    unsafe { cyclonedds_sys::dds_resume(publisher) }
        .into_error()
        .map(|_| ())
}

pub fn dds_wait_for_acks(
    publisher_or_writer: cyclonedds_sys::dds_entity_t,
    timeout: cyclonedds_sys::dds_duration_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_wait_for_acks(publisher_or_writer, timeout) }.into_error()?;
    Ok(())
}

/// Create a subscriber under a participant. This is primarily used by the
/// [`Subscriber`][`crate::Subscriber`] wrapper.
pub fn dds_create_subscriber(
    participant: cyclonedds_sys::dds_entity_t,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe {
        cyclonedds_sys::dds_create_subscriber(
            participant,
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
        )
    }
    .into_error()
}

pub fn dds_notify_readers(subscriber: cyclonedds_sys::dds_entity_t) -> Result<()> {
    unsafe { cyclonedds_sys::dds_notify_readers(subscriber) }
        .into_error()
        .map(|_| ())
}

/// Create a reader under a participant or subscriber on a topic. This is
/// primarily used by the [`Reader`][`crate::Reader`] wrapper.
pub fn dds_create_reader(
    participant_or_subscriber: cyclonedds_sys::dds_entity_t,
    topic: cyclonedds_sys::dds_entity_t,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe {
        cyclonedds_sys::dds_create_reader(
            participant_or_subscriber,
            topic,
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
        )
    }
    .into_error()
}

/// Create a writer under a participant or publisher on a topic. This is
/// primarily used by the [`Writer`][`crate::Writer`] wrapper.
pub fn dds_create_writer(
    participant_or_publisher: cyclonedds_sys::dds_entity_t,
    topic: cyclonedds_sys::dds_entity_t,
    qos: Option<&cyclonedds_sys::dds_qos_t>,
    listener: Option<&Listener>,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe {
        cyclonedds_sys::dds_create_writer(
            participant_or_publisher,
            topic,
            qos.map_or(std::ptr::null(), std::ptr::from_ref),
            listener.map_or(std::ptr::null(), |listener| {
                listener.inner.as_ptr().cast_const()
            }),
        )
    }
    .into_error()
}

pub fn dds_write<T>(writer: cyclonedds_sys::dds_entity_t, sample: &T) -> Result<()>
where
    T: crate::Topicable,
{
    let sample = InternalSample::SampleRef(sample);
    let sample = (&raw const sample).cast();
    unsafe { cyclonedds_sys::dds_write(writer, sample) }.into_error()?;
    Ok(())
}

pub fn dds_write_with_timestamp<T>(
    writer: cyclonedds_sys::dds_entity_t,
    sample: &T,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()>
where
    T: crate::Topicable,
{
    let sample = InternalSample::SampleRef(sample);
    let sample = (&raw const sample).cast();
    unsafe { cyclonedds_sys::dds_write_ts(writer, sample, timestamp) }.into_error()?;
    Ok(())
}

pub fn dds_write_flush(writer: cyclonedds_sys::dds_entity_t) -> Result<()> {
    unsafe { cyclonedds_sys::dds_write_flush(writer) }.into_error()?;
    Ok(())
}

pub fn dds_register_instance<T>(
    writer: cyclonedds_sys::dds_entity_t,
    key: &T::Key,
) -> Result<cyclonedds_sys::dds_instance_handle_t>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    let mut instance_handle = 0;
    unsafe { cyclonedds_sys::dds_register_instance(writer, &raw mut instance_handle, key) }
        .into_error()?;
    Ok(instance_handle)
}

pub fn dds_unregister_instance<T>(writer: cyclonedds_sys::dds_entity_t, key: &T::Key) -> Result<()>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    unsafe { cyclonedds_sys::dds_unregister_instance(writer, key) }.into_error()?;
    Ok(())
}

pub fn dds_unregister_instance_with_timestamp<T>(
    writer: cyclonedds_sys::dds_entity_t,
    key: &T::Key,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    unsafe { cyclonedds_sys::dds_unregister_instance_ts(writer, key, timestamp) }.into_error()?;
    Ok(())
}

pub fn dds_unregister_instance_by_handle(
    writer: cyclonedds_sys::dds_entity_t,
    instance_handle: cyclonedds_sys::dds_instance_handle_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_unregister_instance_ih(writer, instance_handle) }.into_error()?;
    Ok(())
}

pub fn dds_unregister_instance_by_handle_with_timestamp(
    writer: cyclonedds_sys::dds_entity_t,
    instance_handle: cyclonedds_sys::dds_instance_handle_t,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_unregister_instance_ih_ts(writer, instance_handle, timestamp) }
        .into_error()?;
    Ok(())
}

pub fn dds_lookup_instance<T>(
    writer: cyclonedds_sys::dds_entity_t,
    key: &T::Key,
) -> Option<cyclonedds_sys::dds_instance_handle_t>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    let instance_handle = unsafe { cyclonedds_sys::dds_lookup_instance(writer, key) };
    if instance_handle == u64::from(cyclonedds_sys::DDS_HANDLE_NIL) {
        None
    } else {
        Some(instance_handle)
    }
}

pub fn dds_write_dispose<T>(writer: cyclonedds_sys::dds_entity_t, data: &T) -> Result<()>
where
    T: crate::Topicable,
{
    let sample = InternalSample::SampleRef(data);
    let sample = (&raw const sample).cast();
    unsafe { cyclonedds_sys::dds_writedispose(writer, sample) }.into_error()?;
    Ok(())
}

pub fn dds_write_dispose_with_timestamp<T>(
    writer: cyclonedds_sys::dds_entity_t,
    data: &T,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()>
where
    T: crate::Topicable,
{
    let sample = InternalSample::SampleRef(data);
    let sample = (&raw const sample).cast();
    unsafe { cyclonedds_sys::dds_writedispose_ts(writer, sample, timestamp) }.into_error()?;
    Ok(())
}

pub fn dds_dispose<T>(writer: cyclonedds_sys::dds_entity_t, key: &T::Key) -> Result<()>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    unsafe { cyclonedds_sys::dds_dispose(writer, key) }.into_error()?;
    Ok(())
}

pub fn dds_dispose_with_timestamp<T>(
    writer: cyclonedds_sys::dds_entity_t,
    key: &T::Key,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()>
where
    T: crate::Topicable,
{
    let key = InternalSample::<T>::KeyRef(key);
    let key = (&raw const key).cast();
    unsafe { cyclonedds_sys::dds_dispose_ts(writer, key, timestamp) }.into_error()?;
    Ok(())
}

pub fn dds_dispose_instance_by_handle(
    writer: cyclonedds_sys::dds_entity_t,
    instance_handle: cyclonedds_sys::dds_instance_handle_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_dispose_ih(writer, instance_handle) }.into_error()?;
    Ok(())
}

pub fn dds_dispose_instance_by_handle_with_timestamp(
    writer: cyclonedds_sys::dds_entity_t,
    instance_handle: cyclonedds_sys::dds_instance_handle_t,
    timestamp: cyclonedds_sys::dds_time_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_dispose_ih_ts(writer, instance_handle, timestamp) }
        .into_error()?;
    Ok(())
}

pub fn dds_get_matched_subscriptions(
    writer: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<cyclonedds_sys::dds_instance_handle_t>> {
    let count = usize::try_from(
        unsafe { cyclonedds_sys::dds_get_matched_subscriptions(writer, std::ptr::null_mut(), 0) }
            .into_error()?,
    )
    .expect("count is a non-negative i32 and so always fits in usize");

    let mut matched = vec![0; count];
    if count > 0 {
        unsafe {
            cyclonedds_sys::dds_get_matched_subscriptions(writer, matched.as_mut_ptr(), count)
        }
        .into_error()
        .map(|count| {
            let count = usize::try_from(count)
                .expect("count is a non-negative i32 and so always fits in a usize");
            debug_assert!(
                count <= matched.len(),
                "actual number of subscriptions exceeds expected number of subscriptions"
            );
            matched.truncate(count);
            matched
        })
    } else {
        Ok(matched)
    }
}

pub(crate) unsafe extern "C" fn dds_read_with_collector_callback<T>(
    arg: *mut std::ffi::c_void,
    info: *const cyclonedds_sys::dds_sample_info_t,
    sertype: *const cyclonedds_sys::ddsi_sertype,
    serdata: *mut cyclonedds_sys::ddsi_serdata,
) -> cyclonedds_sys::dds_return_t
where
    T: crate::Topicable,
{
    let buffer = unsafe { &mut *(arg.cast::<Vec<crate::sample::SampleOrKey<T>>>()) };

    let info = unsafe { &*info };
    let mut _sertype = std::mem::ManuallyDrop::new(unsafe {
        Box::from_raw(sertype as *mut crate::internal::sertype::Sertype<T>)
    });
    let mut serdata = std::mem::ManuallyDrop::new(unsafe {
        Box::from_raw(serdata.cast::<crate::internal::serdata::Serdata<T>>())
    });

    let valid_data = info.valid_data;
    let info: crate::sample::Info = info.into();

    if valid_data {
        let sample = serdata.sample().clone();
        buffer.push(crate::sample::SampleOrKey::new_sample(sample, info));
        cyclonedds_sys::DDS_RETCODE_OK.cast_signed()
    } else {
        let key = serdata.key().clone();
        buffer.push(crate::sample::SampleOrKey::new_key(key, info));
        cyclonedds_sys::DDS_RETCODE_OK.cast_signed()
    }
}

pub(crate) mod read_operation {
    type Collector = unsafe extern "C" fn(
        reader_or_condition: cyclonedds_sys::dds_entity_t,
        maxs: u32,
        handle: cyclonedds_sys::dds_instance_handle_t,
        mask: u32,
        collect_sample: cyclonedds_sys::dds_read_with_collector_fn_t,
        collect_sample_arg: *mut std::ffi::c_void,
    ) -> cyclonedds_sys::dds_return_t;

    pub trait ReadOperation {
        const COLLECTOR: Collector;
    }

    pub struct Peek;
    pub struct Read;
    pub struct Take;

    impl ReadOperation for Peek {
        const COLLECTOR: Collector = cyclonedds_sys::dds_peek_with_collector;
    }
    impl ReadOperation for Read {
        const COLLECTOR: Collector = cyclonedds_sys::dds_read_with_collector;
    }
    impl ReadOperation for Take {
        const COLLECTOR: Collector = cyclonedds_sys::dds_take_with_collector;
    }
}

pub(crate) fn dds_peek_read_take<T, RO>(
    reader_or_condition: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<crate::sample::SampleOrKey<T>>>
where
    T: crate::Topicable,
    RO: read_operation::ReadOperation,
{
    let mut samples = Vec::new();

    let handle = Default::default();
    let mask = Default::default();
    let maxs = i32::MAX as u32;
    let len = usize::try_from(
        unsafe {
            RO::COLLECTOR(
                reader_or_condition,
                maxs,
                handle,
                mask,
                Some(dds_read_with_collector_callback::<T>),
                (&raw mut samples).cast(),
            )
        }
        .into_error()?,
    )
    .expect("len is a non-negative i32 and so always fits in usize");

    assert_eq!(
        len,
        samples.len(),
        "number of samples reported from the C side does not match the final number in the buffer"
    );

    Ok(samples)
}

pub fn dds_take<T>(
    reader_or_condition: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<crate::sample::SampleOrKey<T>>>
where
    T: crate::Topicable,
{
    dds_peek_read_take::<T, read_operation::Take>(reader_or_condition)
}

pub fn dds_read<T>(
    reader_or_condition: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<crate::sample::SampleOrKey<T>>>
where
    T: crate::Topicable,
{
    dds_peek_read_take::<T, read_operation::Read>(reader_or_condition)
}

pub fn dds_peek<T>(
    reader_or_condition: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<crate::sample::SampleOrKey<T>>>
where
    T: crate::Topicable,
{
    dds_peek_read_take::<T, read_operation::Peek>(reader_or_condition)
}

pub fn dds_reader_wait_for_historical_data(
    reader: cyclonedds_sys::dds_entity_t,
    timeout: cyclonedds_sys::dds_duration_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_reader_wait_for_historical_data(reader, timeout) }
        .into_error()
        .map(|_| ())
}

pub fn dds_get_matched_publications(
    reader: cyclonedds_sys::dds_entity_t,
) -> Result<Vec<cyclonedds_sys::dds_instance_handle_t>> {
    let count = usize::try_from(
        unsafe { cyclonedds_sys::dds_get_matched_publications(reader, std::ptr::null_mut(), 0) }
            .into_error()?,
    )
    .expect("count is a non-negative i32 and so always fits in usize");

    let mut matched = vec![0; count];

    if count > 0 {
        unsafe { cyclonedds_sys::dds_get_matched_publications(reader, matched.as_mut_ptr(), count) }
            .into_error()
            .map(|count| {
                let count = usize::try_from(count)
                    .expect("count is a non-negative i32 and so always fits in a usize");
                debug_assert!(
                    count <= matched.len(),
                    "actual number of publications exceeds expected number of publications"
                );
                matched.truncate(count);
                matched
            })
    } else {
        Ok(matched)
    }
}

pub fn dds_get_participant(
    entity: cyclonedds_sys::dds_entity_t,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe { cyclonedds_sys::dds_get_participant(entity) }.into_error()
}

pub fn dds_create_readcondition(
    reader: cyclonedds_sys::dds_entity_t,
    mask: u32,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe { cyclonedds_sys::dds_create_readcondition(reader, mask) }.into_error()
}

pub fn dds_create_querycondition<T, F, Callback>(
    reader: cyclonedds_sys::dds_entity_t,
    mask: u32,
) -> Result<cyclonedds_sys::dds_entity_t>
where
    T: std::panic::UnwindSafe + std::panic::RefUnwindSafe + crate::Topicable,
    F: Fn(&T) -> bool,
    Callback: Filter<T, F>,
{
    let _ = Callback::IS_PROVIDED_CALLBACK_ZERO_SIZED;
    unsafe {
        cyclonedds_sys::dds_create_querycondition(
            reader,
            mask,
            Some(wrap_filter::<T, F, Callback>()),
        )
    }
    .into_error()
}

fn wrap_filter<T, F, Callback>() -> unsafe extern "C" fn(*const std::ffi::c_void) -> bool
where
    T: std::panic::UnwindSafe + std::panic::RefUnwindSafe + crate::Topicable,
    F: Fn(&T) -> bool,
    Callback: Filter<T, F>,
{
    unsafe extern "C" fn filter_callback<T, F, Callback>(sample: *const std::ffi::c_void) -> bool
    where
        T: std::panic::UnwindSafe + std::panic::RefUnwindSafe + crate::Topicable,
        F: Fn(&T) -> bool,
        Callback: Filter<T, F>,
    {
        let sample = unsafe { &*(sample.cast::<InternalSample<'_, T>>()) };
        let InternalSample::Sample(sample) = sample else {
            return false;
        };

        std::panic::catch_unwind(|| Callback::filter(sample)).unwrap_or(false)
    }
    filter_callback::<T, F, Callback>
}

pub trait Filter<T, F>
where
    F: Fn(&T) -> bool,
{
    const IS_PROVIDED_CALLBACK_ZERO_SIZED: bool = {
        assert!(
            size_of::<F>() == 0,
            "\
 the provided callback is not zero-sized
   = note: closures that capture values from their environment are not zero-sized
   = help: ensure the callback is either:
           - a function item, e.g. `fn my_callback() {{}}`
           - a closure that does not capture any external state"
        );
        size_of::<F>() == 0
    };

    fn filter(sample: &T) -> bool {
        // NOTE horribly unsafe.
        let function = unsafe { std::mem::zeroed::<F>() };
        function(sample)
    }
}

pub fn dds_create_guardcondition(
    owner: cyclonedds_sys::dds_entity_t,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe { cyclonedds_sys::dds_create_guardcondition(owner) }.into_error()
}

pub fn dds_set_guardcondition(
    guard_condition: cyclonedds_sys::dds_entity_t,
    triggered: bool,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_set_guardcondition(guard_condition, triggered) }.into_error()?;
    Ok(())
}

pub fn dds_read_guardcondition(guard_condition: cyclonedds_sys::dds_entity_t) -> Result<bool> {
    let mut triggered = false;
    unsafe { cyclonedds_sys::dds_read_guardcondition(guard_condition, &raw mut triggered) }
        .into_error()?;
    Ok(triggered)
}

pub fn dds_take_guardcondition(guard_condition: cyclonedds_sys::dds_entity_t) -> Result<bool> {
    let mut triggered = false;
    unsafe { cyclonedds_sys::dds_take_guardcondition(guard_condition, &raw mut triggered) }
        .into_error()?;
    Ok(triggered)
}

pub fn dds_get_mask(condition: cyclonedds_sys::dds_entity_t) -> Result<u32> {
    let mut mask: u32 = 0;
    unsafe { cyclonedds_sys::dds_get_mask(condition, &raw mut mask) }.into_error()?;
    Ok(mask)
}

pub fn dds_triggered(entity: cyclonedds_sys::dds_entity_t) -> Result<bool> {
    let triggered = unsafe { cyclonedds_sys::dds_triggered(entity) }.into_error()?;
    Ok(triggered == 1)
}

pub fn dds_create_waitset(
    participant: cyclonedds_sys::dds_entity_t,
) -> Result<cyclonedds_sys::dds_entity_t> {
    unsafe { cyclonedds_sys::dds_create_waitset(participant) }.into_error()
}

pub fn dds_waitset_attach(
    waitset: cyclonedds_sys::dds_entity_t,
    entity: cyclonedds_sys::dds_entity_t,
    blob: isize,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_waitset_attach(waitset, entity, blob) }.into_error()?;
    Ok(())
}

pub fn dds_waitset_detach(
    waitset: cyclonedds_sys::dds_entity_t,
    entity: cyclonedds_sys::dds_entity_t,
) -> Result<()> {
    unsafe { cyclonedds_sys::dds_waitset_detach(waitset, entity) }.into_error()?;
    Ok(())
}

pub fn dds_waitset_set_trigger(waitset: cyclonedds_sys::dds_entity_t, trigger: bool) -> Result<()> {
    unsafe { cyclonedds_sys::dds_waitset_set_trigger(waitset, trigger) }.into_error()?;
    Ok(())
}

pub fn dds_waitset_wait<'a, A>(
    waitset: cyclonedds_sys::dds_entity_t,
    max_number_of_blobs: usize,
    timeout: cyclonedds_sys::dds_duration_t,
) -> Result<(i32, Vec<&'a A>)> {
    let mut blobs: Vec<isize> = vec![0; max_number_of_blobs];

    let number_of_triggered_entities = unsafe {
        cyclonedds_sys::dds_waitset_wait(waitset, blobs.as_mut_ptr(), blobs.len(), timeout)
    }
    .into_error()?;

    if number_of_triggered_entities == 0 {
        Err(crate::Error::Timeout)
    } else {
        let blobs: Vec<_> = blobs
            .iter()
            .filter_map(|&blob| {
                let blob = blob as *const A;
                if blob.is_null() {
                    None
                } else {
                    let blob = unsafe { &*blob };
                    Some(blob)
                }
            })
            .collect();

        Ok((number_of_triggered_entities, blobs))
    }
}

pub fn dds_waitset_wait_until<'a, A>(
    waitset: cyclonedds_sys::dds_entity_t,
    max_number_of_blobs: usize,
    absolute_time: cyclonedds_sys::dds_time_t,
) -> Result<(i32, Vec<&'a A>)> {
    let mut blobs: Vec<isize> = vec![0; max_number_of_blobs];

    let number_of_triggered_entities = unsafe {
        cyclonedds_sys::dds_waitset_wait_until(
            waitset,
            blobs.as_mut_ptr(),
            blobs.len(),
            absolute_time,
        )
    }
    .into_error()?;

    if number_of_triggered_entities == 0 {
        Err(crate::Error::Timeout)
    } else {
        let blobs: Vec<_> = blobs
            .iter()
            .filter_map(|&blob| {
                let blob = blob as *const A;
                if blob.is_null() {
                    None
                } else {
                    let blob = unsafe { &*blob };
                    Some(blob)
                }
            })
            .collect();

        Ok((number_of_triggered_entities, blobs))
    }
}

#[cfg(test)]
mod tests;
