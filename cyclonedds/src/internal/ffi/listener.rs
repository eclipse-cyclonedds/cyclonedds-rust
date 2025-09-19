use crate::Result;
use crate::error::IntoError;
use crate::internal::ffi;

pub struct Listener {
    pub inner: std::ptr::NonNull<cyclonedds_sys::dds_listener_t>,
}

impl Listener {
    pub fn new() -> Result<Self> {
        ffi::dds_create_listener().map(|inner| Self { inner })
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        ffi::dds_delete_listener(self.inner);
    }
}

pub fn dds_create_listener() -> Result<std::ptr::NonNull<cyclonedds_sys::dds_listener_t>> {
    let inner = unsafe { cyclonedds_sys::dds_create_listener(std::ptr::null_mut()) };
    std::ptr::NonNull::new(inner).ok_or(crate::Error::OutOfResources)
}

pub fn dds_set_listener(
    entity: cyclonedds_sys::dds_entity_t,
    listener: Option<std::ptr::NonNull<cyclonedds_sys::dds_listener_t>>,
) -> Result<()> {
    unsafe {
        cyclonedds_sys::dds_set_listener(
            entity,
            listener.map_or(std::ptr::null(), |listener| listener.as_ptr().cast_const()),
        )
    }
    .into_error()?;

    Ok(())
}

pub fn dds_delete_listener(mut listener: std::ptr::NonNull<cyclonedds_sys::dds_listener_t>) {
    unsafe {
        cyclonedds_sys::dds_delete_listener(listener.as_mut());
    }
}

pub fn dds_listener_set_inconsistent_topic<T>(
    listener: &mut Listener,
    callback: fn(&crate::Topic<'_, '_, T>, crate::status::InconsistentTopic),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_inconsistent_topic_arg(
            listener.inner.as_mut(),
            Some(on_inconsistent_topic_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_liveliness_lost<T>(
    listener: &mut Listener,
    callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::LivelinessLost),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_liveliness_lost_arg(
            listener.inner.as_mut(),
            Some(on_liveliness_lost_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_offered_deadline_missed<T>(
    listener: &mut Listener,
    callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::OfferedDeadlineMissed),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_offered_deadline_missed_arg(
            listener.inner.as_mut(),
            Some(on_offered_deadline_missed_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_offered_incompatible_qos<T>(
    listener: &mut Listener,
    callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::OfferedIncompatibleQoS),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_offered_incompatible_qos_arg(
            listener.inner.as_mut(),
            Some(on_offered_incompatible_qos_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_publication_matched<T>(
    listener: &mut Listener,
    callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::PublicationMatched),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_publication_matched_arg(
            listener.inner.as_mut(),
            Some(on_publication_matched_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_sample_lost<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SampleLost),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_sample_lost_arg(
            listener.inner.as_mut(),
            Some(on_sample_lost_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_data_available<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_data_available_arg(
            listener.inner.as_mut(),
            Some(on_data_available_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_sample_rejected<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SampleRejected),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_sample_rejected_arg(
            listener.inner.as_mut(),
            Some(on_sample_rejected_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_liveliness_changed<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::LivelinessChanged),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_liveliness_changed_arg(
            listener.inner.as_mut(),
            Some(on_liveliness_changed_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_requested_deadline_missed<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::RequestedDeadlineMissed),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_requested_deadline_missed_arg(
            listener.inner.as_mut(),
            Some(on_requested_deadline_missed_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_requested_incompatible_qos<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::RequestedIncompatibleQoS),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_requested_incompatible_qos_arg(
            listener.inner.as_mut(),
            Some(on_requested_incompatible_qos_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_subscription_matched<T>(
    listener: &mut Listener,
    callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SubscriptionMatched),
) where
    T: crate::Topicable,
{
    unsafe {
        cyclonedds_sys::dds_lset_subscription_matched_arg(
            listener.inner.as_mut(),
            Some(on_subscription_matched_shim::<T>),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

pub fn dds_listener_set_data_on_readers(
    listener: &mut Listener,
    callback: fn(&crate::Subscriber<'_, '_>),
) {
    unsafe {
        cyclonedds_sys::dds_lset_data_on_readers_arg(
            listener.inner.as_mut(),
            Some(on_data_on_readers_shim),
            callback as *mut std::ffi::c_void,
            true,
        )
    }
    .into_error()
    .unwrap();
}

unsafe extern "C" fn on_inconsistent_topic_shim<T>(
    topic: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_inconsistent_topic_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let topic = crate::Topic::from_existing(topic);
    let status = status.into();
    let callback: fn(&crate::Topic<'_, '_, T>, crate::status::InconsistentTopic) =
        unsafe { std::mem::transmute(arg) };
    callback(&topic, status);
}

unsafe extern "C" fn on_liveliness_lost_shim<T>(
    writer: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_liveliness_lost_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let writer = crate::Writer::from_existing(writer);
    let status = status.into();
    let callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::LivelinessLost) =
        unsafe { std::mem::transmute(arg) };
    callback(&writer, status);
}

unsafe extern "C" fn on_offered_deadline_missed_shim<T>(
    writer: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_offered_deadline_missed_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let writer = crate::Writer::from_existing(writer);
    let status = status.into();
    let callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::OfferedDeadlineMissed) =
        unsafe { std::mem::transmute(arg) };
    callback(&writer, status);
}

unsafe extern "C" fn on_offered_incompatible_qos_shim<T>(
    writer: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_offered_incompatible_qos_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let writer = crate::Writer::from_existing(writer);
    let status = status.into();
    let callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::OfferedIncompatibleQoS) =
        unsafe { std::mem::transmute(arg) };
    callback(&writer, status);
}

unsafe extern "C" fn on_publication_matched_shim<T>(
    writer: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_publication_matched_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let writer = crate::Writer::from_existing(writer);
    let status = status.into();
    let callback: fn(&crate::Writer<'_, '_, '_, T>, crate::status::PublicationMatched) =
        unsafe { std::mem::transmute(arg) };
    callback(&writer, status);
}

unsafe extern "C" fn on_sample_lost_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_sample_lost_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SampleLost) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_data_available_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let callback: fn(&crate::Reader<'_, '_, '_, T>) = unsafe { std::mem::transmute(arg) };
    callback(&reader);
}

unsafe extern "C" fn on_sample_rejected_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_sample_rejected_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SampleRejected) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_liveliness_changed_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_liveliness_changed_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::LivelinessChanged) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_requested_deadline_missed_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_requested_deadline_missed_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::RequestedDeadlineMissed) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_requested_incompatible_qos_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_requested_incompatible_qos_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::RequestedIncompatibleQoS) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_subscription_matched_shim<T>(
    reader: cyclonedds_sys::dds_entity_t,
    status: cyclonedds_sys::dds_subscription_matched_status_t,
    arg: *mut std::ffi::c_void,
) where
    T: crate::Topicable,
{
    let reader = crate::Reader::from_existing(reader);
    let status = status.into();
    let callback: fn(&crate::Reader<'_, '_, '_, T>, crate::status::SubscriptionMatched) =
        unsafe { std::mem::transmute(arg) };
    callback(&reader, status);
}

unsafe extern "C" fn on_data_on_readers_shim(
    subscriber: cyclonedds_sys::dds_entity_t,
    arg: *mut std::ffi::c_void,
) {
    let subscriber = crate::Subscriber::from_existing(subscriber);
    let callback: fn(&crate::Subscriber<'_, '_>) = unsafe { std::mem::transmute(arg) };
    callback(&subscriber);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inconsistent_topic_shim_currently_untriggerable() {
        static TRIGGERED: std::sync::Mutex<bool> = std::sync::Mutex::new(false);
        static TOPIC_HANDLE: std::sync::Mutex<Option<cyclonedds_sys::dds_entity_t>> =
            std::sync::Mutex::new(None);

        let domain_id = crate::tests::domain::unique_id();
        let domain = crate::Domain::new(domain_id).unwrap();
        let participant = crate::Participant::new(&domain).unwrap();
        let topic_name = crate::tests::topic::unique_name();
        let topic =
            crate::Topic::<crate::tests::topic::Data>::new(&participant, &topic_name).unwrap();
        let status = cyclonedds_sys::dds_inconsistent_topic_status {
            total_count: 10,
            total_count_change: 0,
        };

        *TOPIC_HANDLE.lock().unwrap() = Some(topic.inner);

        let callback: fn(
            &crate::Topic<'_, '_, crate::tests::topic::Data>,
            crate::status::InconsistentTopic,
        ) = |topic, metadata| {
            assert_eq!(topic.inner, TOPIC_HANDLE.lock().unwrap().unwrap());
            assert_eq!(
                metadata,
                crate::status::InconsistentTopic {
                    total: crate::status::Counter {
                        count: 10,
                        delta: 0
                    }
                }
            );
            *TRIGGERED.lock().unwrap() |= true;
        };

        unsafe {
            on_inconsistent_topic_shim::<crate::tests::topic::Data>(
                topic.inner,
                status,
                callback as *mut std::ffi::c_void,
            );
        }

        assert!(*TRIGGERED.lock().unwrap());
    }
}
