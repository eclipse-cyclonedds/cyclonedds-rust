//! Event metadata types delivered to DDS listener callbacks.
//!
//! Each type corresponds to a status condition defined in the DCPS
//! specification and carries event-specific detail such as counts and instance
//! handles. See the [`listener`](crate::listener) module for how to register
//! callbacks that receive these types.

pub(crate) mod bitflags {
    bitflags::bitflags! {
        /// Flags for specifying the set of statuses that are of interest.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Status: u32 {
            /// Another topic exists with the same name but with different
            /// characteristics. Also see the [`crate::status::InconsistentTopic`] metadata struct.
            const InconsistentTopic =
                1 << cyclonedds_sys::dds_status_id_DDS_INCONSISTENT_TOPIC_STATUS_ID;
            /// The deadline that the writer has committed through its
            /// [`Deadline`](crate::qos::policy::Deadline) policy was not
            /// respected for a specific instance. Also see the [`crate::status::OfferedDeadlineMissed`] metadata struct.
            const OfferedDeadlineMissed =
                1 << cyclonedds_sys::dds_status_id_DDS_OFFERED_DEADLINE_MISSED_STATUS_ID;
            /// The deadline that the reader was expecting through its
            /// [`Deadline`](crate::qos::policy::Deadline) policy was not
            /// respected for a specific instance. Also see the [`crate::status::RequestedDeadlineMissed`] metadata struct.
            const RequestedDeadlineMissed =
                1 << cyclonedds_sys::dds_status_id_DDS_REQUESTED_DEADLINE_MISSED_STATUS_ID;
            /// A [`QoS`](crate::QoS) policy setting was incompatible with what
            /// was requested. Also see the [`crate::status::OfferedIncompatibleQoS`] metadata struct.
            const OfferedIncompatibleQoS =
                1 << cyclonedds_sys::dds_status_id_DDS_OFFERED_INCOMPATIBLE_QOS_STATUS_ID;
            /// A [`QoS`](crate::QoS) policy setting was incompatible with what
            /// is offered. Also see the [`crate::status::RequestedIncompatibleQoS`] metadata struct.
            const RequestedIncompatibleQoS =
                1 << cyclonedds_sys::dds_status_id_DDS_REQUESTED_INCOMPATIBLE_QOS_STATUS_ID;
            /// A sample has been lost (never received). Also see the [`crate::status::SampleLost`] metadata struct.
            const SampleLost =
                1 << cyclonedds_sys::dds_status_id_DDS_SAMPLE_LOST_STATUS_ID;
            /// A received sample has been rejected. Also see the [`crate::status::SampleRejected`] metadata struct.
            const SampleRejected =
                1 << cyclonedds_sys::dds_status_id_DDS_SAMPLE_REJECTED_STATUS_ID;
            /// New information is available in some of the data readers of a
            /// subscriber.
            const DataOnReaders =
                1 << cyclonedds_sys::dds_status_id_DDS_DATA_ON_READERS_STATUS_ID;
            /// New information is available in a data reader.
            const DataAvailable =
                1 << cyclonedds_sys::dds_status_id_DDS_DATA_AVAILABLE_STATUS_ID;
            /// The liveliness that the writer has committed through its
            /// [`Liveliness`](crate::qos::policy::Liveliness) policy was not
            /// respected; thus readers will consider the writer as no longer
            /// "alive". Also see the [`crate::status::LivelinessLost`] metadata struct.
            const LivelinessLost =
                1 << cyclonedds_sys::dds_status_id_DDS_LIVELINESS_LOST_STATUS_ID;
            /// The liveliness of one or more writers, that were writing instances
            /// read through the readers has changed. Some writers have become
            /// "alive" or "not alive". Also see the [`crate::status::LivelinessChanged`] metadata struct.
            const LivelinessChanged =
                1 << cyclonedds_sys::dds_status_id_DDS_LIVELINESS_CHANGED_STATUS_ID;
            /// The writer has found a reader that matches the topic and has a
            /// compatible [`QoS`](crate::QoS). Also see the [`crate::status::PublicationMatched`] metadata struct.
            const PublicationMatched =
                1 << cyclonedds_sys::dds_status_id_DDS_PUBLICATION_MATCHED_STATUS_ID;
            /// The reader has found a writer that matches the topic and has a
            /// compatible [`QoS`](crate::QoS). Also see the [`crate::status::SubscriptionMatched`] metadata struct.
            const SubscriptionMatched =
                1 << cyclonedds_sys::dds_status_id_DDS_SUBSCRIPTION_MATCHED_STATUS_ID;
        }
    }
}
