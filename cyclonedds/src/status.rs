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

/// Identifies a DDS [`QoS`](crate::QoS) policy.
///
/// Used in incompatible [`QoS`](crate::QoS) status events to report which
/// policy caused the mismatch between a reader and writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoSPolicyId {
    /// No valid policy.
    Invalid,
    /// Attaches application-specific data to an entity. See
    /// [`UserData`](crate::qos::policy::UserData).
    UserData,
    /// Controls whether data is stored for late-joining readers. See
    /// [`Durability`](crate::qos::policy::Durability).
    Durability,
    /// Controls the scope and order of data presentation to subscribers. See
    /// [`Presentation`](crate::qos::policy::Presentation).
    Presentation,
    /// The maximum time between successive writes for a given instance. See
    /// [`Deadline`](crate::qos::policy::Deadline).
    Deadline,
    /// The acceptable delay between writing and receiving a sample. See
    /// [`LatencyBudget`](crate::qos::policy::LatencyBudget).
    LatencyBudget,
    /// Controls whether ownership of an instance is shared or exclusive. See
    /// [`Ownership`](crate::qos::policy::Ownership).
    Ownership,
    /// The strength of an exclusive ownership claim.
    OwnershipStrength,
    /// How the system determines whether a writer is still alive. See
    /// [`Liveliness`](crate::qos::policy::Liveliness).
    Liveliness,
    /// Filters samples based on the minimum time between delivery to a reader.
    /// See [`TimeBasedFilter`](crate::qos::policy::TimeBasedFilter).
    TimeBasedFilter,
    /// Restricts communication to a named logical channel within a domain. See
    /// [`Partition`](crate::qos::policy::Partition).
    Partition,
    /// Delivery guarantee: best-effort or reliable. See
    /// [`Reliability`](crate::qos::policy::Reliability).
    Reliability,
    /// The order in which samples are delivered to a reader. See
    /// [`DestinationOrder`](crate::qos::policy::DestinationOrder).
    DestinationOrder,
    /// How many samples are stored per instance. See
    /// [`History`](crate::qos::policy::History).
    History,
    /// Caps on the number of instances, samples, and samples-per-instance. See
    /// [`ResourceLimits`](crate::qos::policy::ResourceLimits).
    ResourceLimits,
    /// Controls whether child entities are automatically enabled on creation.
    /// See [`EntityFactory`](crate::qos::policy::EntityFactory).
    EntityFactory,
    /// Controls how a writer handles unregistered instances on deletion. See
    /// [`WriterDataLifecycle`](crate::qos::policy::WriterDataLifecycle).
    WriterDataLifecycle,
    /// Controls how a reader handles instances when matched writers disappear.
    /// See [`ReaderDataLifecycle`](crate::qos::policy::ReaderDataLifecycle).
    ReaderDataLifecycle,
    /// Attaches application-specific data to a topic. See
    /// [`TopicData`](crate::qos::policy::TopicData).
    TopicData,
    /// Attaches application-specific data to a publisher or subscriber. See
    /// [`GroupData`](crate::qos::policy::GroupData).
    GroupData,
    /// A hint to the transport layer about relative priority. See
    /// [`TransportPriority`](crate::qos::policy::TransportPriority).
    TransportPriority,
    /// The maximum duration a sample remains valid. See
    /// [`Lifespan`](crate::qos::policy::Lifespan).
    Lifespan,
    /// Configures the durability service's history and resource limits. See
    /// [`DurabilityService`](crate::qos::policy::DurabilityService).
    DurabilityService,
    /// Attaches key-value properties to an entity.
    Property,
    /// Controls type compatibility checking between readers and writers.
    TypeConsistencyEnforcement,
    /// The data representation format used for serialization.
    DataRepresentation,
}

impl From<u32> for QoSPolicyId {
    fn from(value: u32) -> Self {
        // dds_qos_policy_id_t is an enum whose values are signed under Windows
        // and unsigned otherwise (probably?). This is due to the default value
        // being used to represented an enum in `clang` and is propagated by
        // `bindgen`. We want to stabilize these enums to unsigned but need to
        // cast back to the whatever type is actually being exported based on
        // the platform.
        #[allow(clippy::cast_possible_wrap)]
        match value as cyclonedds_sys::dds_qos_policy_id_t {
            cyclonedds_sys::dds_qos_policy_id_DDS_INVALID_QOS_POLICY_ID => Self::Invalid,
            cyclonedds_sys::dds_qos_policy_id_DDS_USERDATA_QOS_POLICY_ID => Self::UserData,
            cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITY_QOS_POLICY_ID => Self::Durability,
            cyclonedds_sys::dds_qos_policy_id_DDS_PRESENTATION_QOS_POLICY_ID => Self::Presentation,
            cyclonedds_sys::dds_qos_policy_id_DDS_DEADLINE_QOS_POLICY_ID => Self::Deadline,
            cyclonedds_sys::dds_qos_policy_id_DDS_LATENCYBUDGET_QOS_POLICY_ID => {
                Self::LatencyBudget
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_OWNERSHIP_QOS_POLICY_ID => Self::Ownership,
            cyclonedds_sys::dds_qos_policy_id_DDS_OWNERSHIPSTRENGTH_QOS_POLICY_ID => {
                Self::OwnershipStrength
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_LIVELINESS_QOS_POLICY_ID => Self::Liveliness,
            cyclonedds_sys::dds_qos_policy_id_DDS_TIMEBASEDFILTER_QOS_POLICY_ID => {
                Self::TimeBasedFilter
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_PARTITION_QOS_POLICY_ID => Self::Partition,
            cyclonedds_sys::dds_qos_policy_id_DDS_RELIABILITY_QOS_POLICY_ID => Self::Reliability,
            cyclonedds_sys::dds_qos_policy_id_DDS_DESTINATIONORDER_QOS_POLICY_ID => {
                Self::DestinationOrder
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_HISTORY_QOS_POLICY_ID => Self::History,
            cyclonedds_sys::dds_qos_policy_id_DDS_RESOURCELIMITS_QOS_POLICY_ID => {
                Self::ResourceLimits
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_ENTITYFACTORY_QOS_POLICY_ID => {
                Self::EntityFactory
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_WRITERDATALIFECYCLE_QOS_POLICY_ID => {
                Self::WriterDataLifecycle
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_READERDATALIFECYCLE_QOS_POLICY_ID => {
                Self::ReaderDataLifecycle
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_TOPICDATA_QOS_POLICY_ID => Self::TopicData,
            cyclonedds_sys::dds_qos_policy_id_DDS_GROUPDATA_QOS_POLICY_ID => Self::GroupData,
            cyclonedds_sys::dds_qos_policy_id_DDS_TRANSPORTPRIORITY_QOS_POLICY_ID => {
                Self::TransportPriority
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_LIFESPAN_QOS_POLICY_ID => Self::Lifespan,
            cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITYSERVICE_QOS_POLICY_ID => {
                Self::DurabilityService
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_PROPERTY_QOS_POLICY_ID => Self::Property,
            cyclonedds_sys::dds_qos_policy_id_DDS_TYPE_CONSISTENCY_ENFORCEMENT_QOS_POLICY_ID => {
                Self::TypeConsistencyEnforcement
            }
            cyclonedds_sys::dds_qos_policy_id_DDS_DATA_REPRESENTATION_QOS_POLICY_ID => {
                Self::DataRepresentation
            }
            value => unreachable!(
                "unsupported value: {value} in conversion to {}",
                std::any::type_name::<Self>()
            ),
        }
    }
}

/// A cumulative status event counter with a per-notification delta.
///
/// Appears in status types to report both the running total of an event and
/// how many times it occurred since the last time the status was read or taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counter {
    /// Total number of times the event has occurred.
    pub count: u32,
    /// Change in `count` since the status was last read or taken.
    pub delta: i32,
}

/// Delivered to the
/// [`with_inconsistent_topic`](crate::listener::TopicListener::with_inconsistent_topic)
/// callback when a remote topic is discovered with the same name but an
/// incompatible type or [`QoS`](crate::QoS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InconsistentTopic {
    /// Running count of inconsistent topic discoveries.
    pub total: Counter,
}

/// Delivered to the
/// [`with_liveliness_lost`](crate::listener::WriterListener::with_liveliness_lost)
/// callback when the writer fails to meet its
/// [`Liveliness`](crate::qos::policy::Liveliness) policy and is considered
/// inactive by matched readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivelinessLost {
    /// Running count of liveliness violations.
    pub total: Counter,
}

/// Delivered to the
/// [`with_offered_deadline_missed`](crate::listener::WriterListener::with_offered_deadline_missed)
/// callback when the writer fails to write a new sample within its offered
/// [`Deadline`](crate::qos::policy::Deadline) period for one or more
/// instances.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferedDeadlineMissed {
    /// Running count of deadline violations.
    pub total: Counter,
    /// Instance handle of the last instance that missed its deadline.
    pub last_instance_handle: crate::entity::InstanceHandle,
}

/// Delivered to the
/// [`with_offered_incompatible_qos`](crate::listener::WriterListener::with_offered_incompatible_qos)
/// callback when a reader is discovered whose requested [`QoS`](crate::QoS) is
/// incompatible with this writer's offered [`QoS`](crate::QoS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfferedIncompatibleQoS {
    /// Running count of incompatible [`QoS`](crate::QoS) discoveries.
    pub total: Counter,
    /// The policy that caused the most recent incompatibility.
    pub last_policy_id: QoSPolicyId,
}

/// Delivered to the
/// [`with_publication_matched`](crate::listener::WriterListener::with_publication_matched)
/// callback when a reader matching this writer's topic and [`QoS`](crate::QoS)
/// is discovered or lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicationMatched {
    /// Running count of reader matches over the lifetime of the writer.
    pub total: Counter,
    /// Current number of matched readers.
    pub current: Counter,
    /// Instance handle of the last reader that matched or unmatched.
    pub last_subscription_handle: crate::entity::InstanceHandle,
}

/// Delivered to the
/// [`with_sample_lost`](crate::listener::ReaderListener::with_sample_lost)
/// callback when a sample is lost before being received by the reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleLost {
    /// Running count of lost samples.
    pub total: Counter,
}

/// Delivered to the
/// [`with_sample_rejected`](crate::listener::ReaderListener::with_sample_rejected)
/// callback when an incoming sample is rejected due to [`resource
/// limits`](crate::qos::policy::ResourceLimits).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SampleRejectedReason {
    // TODO How can sample not rejected be a valid status for the sample
    // rejected status? should this be modeled as an Option in
    // samplerejected?
    /// The sample was not rejected.
    NotRejected,
    /// Rejected because the maximum number of instances has been reached.
    RejectedByInstancesLimit,
    /// Rejected because the maximum number of samples has been reached.
    RejectedBySamplesLimit,
    /// Rejected because the maximum number of samples per instance has been
    /// reached.
    RejectedBySamplesPerInstanceLimit,
}

impl From<cyclonedds_sys::dds_sample_rejected_status_kind> for SampleRejectedReason {
    fn from(reason: cyclonedds_sys::dds_sample_rejected_status_kind) -> Self {
        match reason {
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_NOT_REJECTED => Self::NotRejected,
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_INSTANCES_LIMIT => Self::RejectedByInstancesLimit,
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_SAMPLES_LIMIT => Self::RejectedBySamplesLimit,
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_SAMPLES_PER_INSTANCE_LIMIT => Self::RejectedBySamplesPerInstanceLimit,
            value => unreachable!("unsupported value: {value} in conversion to {}", std::any::type_name::<Self>())
        }
    }
}

/// Delivered to the
/// [`with_sample_rejected`](crate::listener::ReaderListener::with_sample_rejected)
/// callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleRejected {
    /// Running count of rejected samples.
    pub total: Counter,
    /// The reason the most recent sample was rejected.
    pub last_reason: SampleRejectedReason,
    /// Instance handle of the instance whose sample was most recently rejected.
    pub last_instance_handle: crate::entity::InstanceHandle,
}

/// Delivered to the
/// [`with_requested_deadline_missed`](crate::listener::ReaderListener::with_requested_deadline_missed)
/// callback when a sample is not received within the
/// [`Deadline`](crate::qos::policy::Deadline) period offered by a matched
/// writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestedDeadlineMissed {
    /// Running count of deadline misses across all instances.
    pub total: Counter,
    /// Instance handle of the last instance that missed its deadline.
    pub last_instance_handle: crate::entity::InstanceHandle,
}

/// Delivered to the
/// [`with_requested_incompatible_qos`](crate::listener::ReaderListener::with_requested_incompatible_qos)
/// callback when a writer is discovered whose offered [`QoS`](crate::QoS) is
/// incompatible with this reader's requested [`QoS`](crate::QoS).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestedIncompatibleQoS {
    /// Running count of incompatible [`QoS`](crate::QoS) discoveries.
    pub total: Counter,
    /// The policy that caused the most recent incompatibility.
    pub last_policy_id: QoSPolicyId,
}

/// Delivered to the
/// [`with_subscription_matched`](crate::listener::ReaderListener::with_subscription_matched)
/// callback when a writer matching this reader's topic and [`QoS`](crate::QoS)
/// is discovered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubscriptionMatched {
    /// Running count of writer matches over the lifetime of the reader.
    pub total: Counter,
    /// Current number of matched writers.
    pub current: Counter,
    /// Instance handle of the last writer that matched or unmatched.
    pub last_publication_handle: crate::entity::InstanceHandle,
}

/// Delivered to the
/// [`with_liveliness_changed`](crate::listener::ReaderListener::with_liveliness_changed)
/// callback when a matched writer transitions between active and inactive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LivelinessChanged {
    /// Running count of matched writers that are currently active.
    pub alive: Counter,
    /// Running count of matched writers that are currently inactive.
    pub not_alive: Counter,
    /// Instance handle of the last writer whose liveliness changed.
    pub last_publication_handle: crate::entity::InstanceHandle,
}

impl From<cyclonedds_sys::dds_inconsistent_topic_status> for InconsistentTopic {
    fn from(status: cyclonedds_sys::dds_inconsistent_topic_status) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };

        Self { total }
    }
}

impl From<cyclonedds_sys::dds_liveliness_lost_status_t> for LivelinessLost {
    fn from(status: cyclonedds_sys::dds_liveliness_lost_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        Self { total }
    }
}

impl From<cyclonedds_sys::dds_offered_deadline_missed_status_t> for OfferedDeadlineMissed {
    fn from(status: cyclonedds_sys::dds_offered_deadline_missed_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let last_instance_handle = crate::entity::InstanceHandle {
            inner: status.last_instance_handle,
        };

        Self {
            total,
            last_instance_handle,
        }
    }
}

impl From<cyclonedds_sys::dds_offered_incompatible_qos_status_t> for OfferedIncompatibleQoS {
    fn from(status: cyclonedds_sys::dds_offered_incompatible_qos_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let last_policy_id = status.last_policy_id.into();

        Self {
            total,
            last_policy_id,
        }
    }
}

impl From<cyclonedds_sys::dds_publication_matched_status_t> for PublicationMatched {
    fn from(status: cyclonedds_sys::dds_publication_matched_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let current = Counter {
            count: status.current_count,
            delta: status.current_count_change,
        };
        let last_subscription_handle = crate::entity::InstanceHandle {
            inner: status.last_subscription_handle,
        };
        Self {
            total,
            current,
            last_subscription_handle,
        }
    }
}

impl From<cyclonedds_sys::dds_sample_lost_status_t> for SampleLost {
    fn from(status: cyclonedds_sys::dds_sample_lost_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        Self { total }
    }
}

impl From<cyclonedds_sys::dds_sample_rejected_status_t> for SampleRejected {
    fn from(status: cyclonedds_sys::dds_sample_rejected_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let last_reason = status.last_reason.into();
        let last_instance_handle = crate::entity::InstanceHandle {
            inner: status.last_instance_handle,
        };
        Self {
            total,
            last_reason,
            last_instance_handle,
        }
    }
}

impl From<cyclonedds_sys::dds_liveliness_changed_status_t> for LivelinessChanged {
    fn from(status: cyclonedds_sys::dds_liveliness_changed_status_t) -> Self {
        let alive = Counter {
            count: status.alive_count,
            delta: status.alive_count_change,
        };
        let not_alive = Counter {
            count: status.not_alive_count,
            delta: status.not_alive_count_change,
        };

        let last_publication_handle = crate::entity::InstanceHandle {
            inner: status.last_publication_handle,
        };
        Self {
            alive,
            not_alive,
            last_publication_handle,
        }
    }
}

impl From<cyclonedds_sys::dds_requested_deadline_missed_status_t> for RequestedDeadlineMissed {
    fn from(status: cyclonedds_sys::dds_requested_deadline_missed_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let last_instance_handle = crate::entity::InstanceHandle {
            inner: status.last_instance_handle,
        };
        Self {
            total,
            last_instance_handle,
        }
    }
}

impl From<cyclonedds_sys::dds_requested_incompatible_qos_status_t> for RequestedIncompatibleQoS {
    fn from(status: cyclonedds_sys::dds_requested_incompatible_qos_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let last_policy_id = status.last_policy_id.into();
        Self {
            total,
            last_policy_id,
        }
    }
}

impl From<cyclonedds_sys::dds_subscription_matched_status_t> for SubscriptionMatched {
    fn from(status: cyclonedds_sys::dds_subscription_matched_status_t) -> Self {
        let total = Counter {
            count: status.total_count,
            delta: status.total_count_change,
        };
        let current = Counter {
            count: status.current_count,
            delta: status.current_count_change,
        };
        let last_publication_handle = crate::entity::InstanceHandle {
            inner: status.last_publication_handle,
        };

        Self {
            total,
            current,
            last_publication_handle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_policy_id_conversion() {
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_INVALID_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Invalid);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_USERDATA_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::UserData);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITY_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::Durability);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_PRESENTATION_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::Presentation);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_DEADLINE_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Deadline);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_LATENCYBUDGET_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::LatencyBudget);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_OWNERSHIP_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Ownership);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_OWNERSHIPSTRENGTH_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::OwnershipStrength);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_LIVELINESS_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::Liveliness);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_TIMEBASEDFILTER_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::TimeBasedFilter);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_PARTITION_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Partition);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_RELIABILITY_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::Reliability);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_DESTINATIONORDER_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::DestinationOrder);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_HISTORY_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::History);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_RESOURCELIMITS_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::ResourceLimits);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_ENTITYFACTORY_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::EntityFactory);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_WRITERDATALIFECYCLE_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::WriterDataLifecycle);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_READERDATALIFECYCLE_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::ReaderDataLifecycle);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_TOPICDATA_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::TopicData);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_GROUPDATA_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::GroupData);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_TRANSPORTPRIORITY_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::TransportPriority);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_LIFESPAN_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Lifespan);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITYSERVICE_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::DurabilityService);
        let result =
            QoSPolicyId::from(cyclonedds_sys::dds_qos_policy_id_DDS_PROPERTY_QOS_POLICY_ID as u32);
        assert_eq!(result, QoSPolicyId::Property);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_TYPE_CONSISTENCY_ENFORCEMENT_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::TypeConsistencyEnforcement);
        let result = QoSPolicyId::from(
            cyclonedds_sys::dds_qos_policy_id_DDS_DATA_REPRESENTATION_QOS_POLICY_ID as u32,
        );
        assert_eq!(result, QoSPolicyId::DataRepresentation);
    }

    #[test]
    #[should_panic = "internal error: entered unreachable code: unsupported value"]
    fn test_qos_policy_id_conversion_out_of_range() {
        let _ = QoSPolicyId::from(u32::MAX);
    }

    #[test]
    fn test_sample_rejected_reason_conversion() {
        let result = SampleRejectedReason::from(
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_NOT_REJECTED,
        );
        assert_eq!(result, SampleRejectedReason::NotRejected);
        let result = SampleRejectedReason::from(
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_INSTANCES_LIMIT,
        );
        assert_eq!(result, SampleRejectedReason::RejectedByInstancesLimit);
        let result = SampleRejectedReason::from(
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_SAMPLES_LIMIT,
        );
        assert_eq!(result, SampleRejectedReason::RejectedBySamplesLimit);
        let result = SampleRejectedReason::from(cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_SAMPLES_PER_INSTANCE_LIMIT);
        assert_eq!(
            result,
            SampleRejectedReason::RejectedBySamplesPerInstanceLimit
        );
    }

    #[test]
    #[should_panic = "internal error: entered unreachable code: unsupported value"]
    fn test_sample_rejected_reason_conversion_out_of_range() {
        let _ = SampleRejectedReason::from(cyclonedds_sys::dds_sample_rejected_status_kind::MAX);
    }

    #[test]
    fn test_inconsistent_topic_conversion() {
        let total_count = 10;
        let total_count_change = 20;

        let status = InconsistentTopic::from(cyclonedds_sys::dds_inconsistent_topic_status {
            total_count,
            total_count_change,
        });

        assert_eq!(
            (status.total.count, status.total.delta),
            (total_count, total_count_change)
        );
    }

    #[test]
    fn test_liveliness_lost_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let status = LivelinessLost::from(cyclonedds_sys::dds_liveliness_lost_status {
            total_count,
            total_count_change,
        });

        assert_eq!(
            (status.total.count, status.total.delta),
            (total_count, total_count_change)
        );
    }

    #[test]
    fn test_offered_deadline_missed_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let last_instance_handle = 30;

        let status =
            OfferedDeadlineMissed::from(cyclonedds_sys::dds_offered_deadline_missed_status {
                total_count,
                total_count_change,
                last_instance_handle,
            });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.last_instance_handle.inner
            ),
            (total_count, total_count_change, last_instance_handle)
        );
    }

    #[test]
    fn test_offered_incompatible_qos_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let last_policy_id = cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITY_QOS_POLICY_ID as u32;

        let status =
            OfferedIncompatibleQoS::from(cyclonedds_sys::dds_offered_incompatible_qos_status {
                total_count,
                total_count_change,
                last_policy_id,
            });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.last_policy_id
            ),
            (total_count, total_count_change, last_policy_id.into())
        );
    }

    #[test]
    fn test_publication_matched_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let current_count = 30;
        let current_count_change = 40;
        let last_subscription_handle = 50;

        let status = PublicationMatched::from(cyclonedds_sys::dds_publication_matched_status {
            total_count,
            total_count_change,
            current_count,
            current_count_change,
            last_subscription_handle,
        });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.current.count,
                status.current.delta,
                status.last_subscription_handle.inner
            ),
            (
                total_count,
                total_count_change,
                current_count,
                current_count_change,
                last_subscription_handle
            )
        );
    }

    #[test]
    fn test_sample_lost_conversion() {
        let total_count = 10;
        let total_count_change = 20;

        let status = SampleLost::from(cyclonedds_sys::dds_sample_lost_status {
            total_count,
            total_count_change,
        });

        assert_eq!(
            (status.total.count, status.total.delta),
            (total_count, total_count_change)
        );
    }

    #[test]
    fn test_sample_rejected_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let last_reason =
            cyclonedds_sys::dds_sample_rejected_status_kind_DDS_REJECTED_BY_INSTANCES_LIMIT;
        let last_instance_handle = 40;

        let status = SampleRejected::from(cyclonedds_sys::dds_sample_rejected_status {
            total_count,
            total_count_change,
            last_reason,
            last_instance_handle,
        });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.last_reason,
                status.last_instance_handle.inner
            ),
            (
                total_count,
                total_count_change,
                last_reason.into(),
                last_instance_handle
            )
        );
    }

    #[test]
    fn test_liveliness_changed_conversion() {
        let alive_count = 10;
        let alive_count_change = 20;
        let not_alive_count = 30;
        let not_alive_count_change = 40;
        let last_publication_handle = 50;

        let status = LivelinessChanged::from(cyclonedds_sys::dds_liveliness_changed_status {
            alive_count,
            alive_count_change,
            not_alive_count,
            not_alive_count_change,
            last_publication_handle,
        });

        assert_eq!(
            (
                status.alive.count,
                status.alive.delta,
                status.not_alive.count,
                status.not_alive.delta,
                status.last_publication_handle.inner
            ),
            (
                alive_count,
                alive_count_change,
                not_alive_count,
                not_alive_count_change,
                last_publication_handle
            )
        );
    }

    #[test]
    fn test_requested_deadline_missed_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let last_instance_handle = 30;

        let status =
            RequestedDeadlineMissed::from(cyclonedds_sys::dds_requested_deadline_missed_status {
                total_count,
                total_count_change,
                last_instance_handle,
            });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.last_instance_handle.inner
            ),
            (total_count, total_count_change, last_instance_handle)
        );
    }

    #[test]
    fn test_requested_incompatible_qos_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let last_policy_id = cyclonedds_sys::dds_qos_policy_id_DDS_DURABILITY_QOS_POLICY_ID as u32;

        let status =
            RequestedIncompatibleQoS::from(cyclonedds_sys::dds_requested_incompatible_qos_status {
                total_count,
                total_count_change,
                last_policy_id,
            });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.last_policy_id
            ),
            (total_count, total_count_change, last_policy_id.into())
        );
    }

    #[test]
    fn test_subscription_matched_conversion() {
        let total_count = 10;
        let total_count_change = 20;
        let current_count = 30;
        let current_count_change = 40;
        let last_publication_handle = 50;

        let status = SubscriptionMatched::from(cyclonedds_sys::dds_subscription_matched_status {
            total_count,
            total_count_change,
            current_count,
            current_count_change,
            last_publication_handle,
        });

        assert_eq!(
            (
                status.total.count,
                status.total.delta,
                status.current.count,
                status.current.delta,
                status.last_publication_handle.inner
            ),
            (
                total_count,
                total_count_change,
                current_count,
                current_count_change,
                last_publication_handle
            )
        );
    }
}
