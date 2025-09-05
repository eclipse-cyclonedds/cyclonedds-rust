//! [`QoS`](crate::QoS) policy types for entities.
//!
//! Each type in this module corresponds to a [`QoS`](crate::QoS) policy defined
//! in the DCPS specification. Policies are set on a [`QoS`](crate::QoS)
//! instance via its
//! `with_*` methods and applied to entities through their builders.
//!
//! Not all policies apply to all entity types. Refer to the DDS specification
//! or Cyclone DDS documentation for the applicability of each policy.

// TODO add the details on what QoS policies apply to what types here too.

use crate::Duration;
use crate::internal::traits::AsFfi;

/// Attaches arbitrary application-specific data to an entity.
///
/// The value is propagated during discovery and made available to remote
/// participants, allowing applications to embed metadata such as version
/// information or node identity in the entity itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserData {
    /// The raw byte payload.
    pub value: Vec<u8>,
}

impl AsFfi for UserData {
    type Target<'a> = &'a [u8];

    fn as_ffi(&self) -> Self::Target<'_> {
        &self.value
    }
}

/// Attaches arbitrary application-specific data to a topic.
///
/// Propagated during discovery alongside the topic description, allowing
/// applications to embed metadata in the topic itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicData {
    /// The raw byte payload.
    pub value: Vec<u8>,
}

impl AsFfi for TopicData {
    type Target<'a> = &'a [u8];

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        &self.value
    }
}

/// Attaches arbitrary application-specific data to a publisher or subscriber.
///
/// Propagated during discovery, allowing applications to embed metadata at
/// the publisher or subscriber level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupData {
    /// The raw byte payload.
    pub value: Vec<u8>,
}

impl AsFfi for GroupData {
    type Target<'a> = &'a [u8];

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        &self.value
    }
}

/// Controls whether samples are stored for late-joining readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Durability {
    /// Samples are not stored. Late-joining readers receive only new samples.
    Volatile,
    /// Samples are stored in the writer. Late-joining readers on the same node
    /// receive historical samples.
    TransientLocal,
    /// Samples are stored in a separate durability service. Late-joining
    /// readers anywhere in the domain receive historical samples.
    Transient,
    /// Like [`Transient`](Durability::Transient) but samples survive process
    /// restarts.
    Persistent,
}

impl AsFfi for Durability {
    type Target<'a> = cyclonedds_sys::dds_durability_kind_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            Durability::Volatile => cyclonedds_sys::dds_durability_kind_DDS_DURABILITY_VOLATILE,
            Durability::TransientLocal => {
                cyclonedds_sys::dds_durability_kind_DDS_DURABILITY_TRANSIENT_LOCAL
            }
            Durability::Transient => cyclonedds_sys::dds_durability_kind_DDS_DURABILITY_TRANSIENT,
            Durability::Persistent => cyclonedds_sys::dds_durability_kind_DDS_DURABILITY_PERSISTENT,
        }
    }
}

/// Configures the history and resource limits of the durability service.
///
/// Only relevant when [`Durability`] is [`Transient`](Durability::Transient) or
/// [`Persistent`](Durability::Persistent). Controls how the durability service
/// stores and purges historical samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DurabilityService {
    /// How long the service retains historical data after all matching readers
    /// have been removed.
    pub service_cleanup_delay: Duration,
    /// History depth to be applied within the durability service.
    pub history: History,
    /// Resource limits applied within the durability service.
    pub resource_limits: ResourceLimits,
}

impl AsFfi for DurabilityService {
    type Target<'a> = (
        cyclonedds_sys::dds_duration_t,
        cyclonedds_sys::dds_history_kind_t,
        i32,
        i32,
        i32,
        i32,
    );

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        let (history_kind, history_depth) = self.history.as_ffi();
        let service_cleanup_delay = self.service_cleanup_delay.inner;

        (
            service_cleanup_delay,
            history_kind,
            history_depth,
            self.resource_limits.max_samples.as_ffi(),
            self.resource_limits.max_instances.as_ffi(),
            self.resource_limits.max_samples_per_instance.as_ffi(),
        )
    }
}

/// Controls the scope and ordering of sample presentation to subscribers.
///
/// The access scope determines the boundary within which `coherent_access` and
/// `ordered_access` are applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presentation {
    /// Coherence and ordering are applied per instance.
    Instance {
        /// Whether changes within a transaction are delivered atomically.
        coherent_access: bool,
        /// Whether samples are delivered in order within the scope.
        ordered_access: bool,
    },
    /// Coherence and ordering are applied across all instances of a topic.
    Topic {
        /// Whether changes within a transaction are delivered atomically.
        coherent_access: bool,
        /// Whether samples are delivered in order within the scope.
        ordered_access: bool,
    },
    /// Coherence and ordering are applied across all topics within a publisher
    /// or subscriber group.
    Group {
        /// Whether changes within a transaction are delivered atomically.
        coherent_access: bool,
        /// Whether samples are delivered in order within the scope.
        ordered_access: bool,
    },
}

impl AsFfi for Presentation {
    type Target<'a> = (
        cyclonedds_sys::dds_presentation_access_scope_kind,
        bool,
        bool,
    );

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            Presentation::Instance {
                coherent_access,
                ordered_access,
            } => (
                cyclonedds_sys::dds_presentation_access_scope_kind_DDS_PRESENTATION_INSTANCE,
                *coherent_access,
                *ordered_access,
            ),
            Presentation::Topic {
                coherent_access,
                ordered_access,
            } => (
                cyclonedds_sys::dds_presentation_access_scope_kind_DDS_PRESENTATION_TOPIC,
                *coherent_access,
                *ordered_access,
            ),
            Presentation::Group {
                coherent_access,
                ordered_access,
            } => (
                cyclonedds_sys::dds_presentation_access_scope_kind_DDS_PRESENTATION_GROUP,
                *coherent_access,
                *ordered_access,
            ),
        }
    }
}

/// The maximum time between successive writes for a given instance.
///
/// Writers and readers negotiate a compatible deadline. If a writer does not
/// write within the deadline period, the
/// [`OfferedDeadlineMissed`](crate::status::OfferedDeadlineMissed) event fires.
/// If a reader does not receive a sample within the period, the
/// [`RequestedDeadlineMissed`](crate::status::RequestedDeadlineMissed) event
/// fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deadline {
    /// The maximum interval between writes for a given instance.
    pub period: Duration,
}

impl AsFfi for Deadline {
    type Target<'a> = cyclonedds_sys::dds_duration_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.period.inner
    }
}

/// The acceptable delay between writing and delivering a sample.
///
/// NOTE: this does not enforce any timing guarantees but is rather a
/// configuration hint that allows the middleware to batch samples that arrive
/// within the budget window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LatencyBudget {
    /// The maximum duration to allow batched results to be transmitted within.
    pub duration: Duration,
}

impl AsFfi for LatencyBudget {
    type Target<'a> = cyclonedds_sys::dds_duration_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.duration.inner
    }
}

/// Controls whether ownership of an instance is shared or exclusive among
/// writers.
///
/// With exclusive ownership, only the writer with the highest
/// [`strength`](Ownership::Exclusive::strength) value delivers samples for a
/// given instance. Other writers are silently ignored by readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ownership {
    /// Multiple writers may deliver samples for the same instance.
    Shared,
    /// Only the writer with the highest strength delivers samples for a given
    /// instance.
    Exclusive {
        /// The ownership strength of this writer. Higher values take
        /// precedence.
        strength: i32,
    },
}

impl AsFfi for Ownership {
    type Target<'a> = (cyclonedds_sys::dds_ownership_kind_t, Option<i32>);

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            Ownership::Shared => (
                cyclonedds_sys::dds_ownership_kind_DDS_OWNERSHIP_SHARED,
                None,
            ),
            Ownership::Exclusive { strength } => (
                cyclonedds_sys::dds_ownership_kind_DDS_OWNERSHIP_EXCLUSIVE,
                Some(*strength),
            ),
        }
    }
}

/// Controls how the system determines whether a writer is still active.
///
/// Readers use the liveliness policy to detect when a matched writer has
/// stopped publishing. When a writer's liveliness is lost, the
/// [`LivelinessChanged`](crate::status::LivelinessChanged) event fires on
/// matched readers, and the [`LivelinessLost`](crate::status::LivelinessLost)
/// event fires on the writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Liveliness {
    /// The middleware asserts liveliness automatically on behalf of the writer.
    Automatic {
        /// The duration within which liveliness must be asserted.
        lease_duration: Duration,
    },
    /// Liveliness is asserted by any write activity from the participant.
    ManualByParticipant {
        /// The duration within which liveliness must be asserted.
        lease_duration: Duration,
    },
    /// Liveliness must be asserted explicitly per writer via a write or
    /// liveliness assertion call.
    ManualByTopic {
        /// The duration within which liveliness must be asserted.
        lease_duration: Duration,
    },
}

impl AsFfi for Liveliness {
    type Target<'a> = (
        cyclonedds_sys::dds_liveliness_kind_t,
        cyclonedds_sys::dds_duration_t,
    );

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            Liveliness::Automatic { lease_duration } => (
                cyclonedds_sys::dds_liveliness_kind_DDS_LIVELINESS_AUTOMATIC,
                lease_duration.inner,
            ),
            Liveliness::ManualByParticipant { lease_duration } => (
                cyclonedds_sys::dds_liveliness_kind_DDS_LIVELINESS_MANUAL_BY_PARTICIPANT,
                lease_duration.inner,
            ),
            Liveliness::ManualByTopic { lease_duration } => (
                cyclonedds_sys::dds_liveliness_kind_DDS_LIVELINESS_MANUAL_BY_TOPIC,
                lease_duration.inner,
            ),
        }
    }
}

/// The minimum time between sample deliveries to a reader for a given instance.
///
/// Samples arriving faster than the minimum separation are dropped. Useful for
/// throttling high-frequency writers at the reader side without changing the
/// writer's publish rate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeBasedFilter {
    /// The minimum interval between delivered samples for a given instance.
    pub minimum_separation: Duration,
}

impl AsFfi for TimeBasedFilter {
    type Target<'a> = cyclonedds_sys::dds_duration_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.minimum_separation.inner
    }
}

/// Restricts communication to named logical partitions within a domain.
///
/// A writer and reader only match if they share at least one partition name.
/// Partition names support wildcards as defined by the DCPS specification. The
/// default partition (empty string) is used when no partition is set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Partition {
    /// The list of partition names.
    pub partitions: Vec<String>,
}

impl AsFfi for Partition {
    type Target<'a> = Vec<std::ffi::CString>;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.partitions
            .iter()
            .map(|partition| {
                std::ffi::CString::new(partition.as_str()).expect(
                    "TODO should this be moved to the construction of the partition policy?",
                )
            })
            .collect()
    }
}

/// The delivery guarantee for samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reliability {
    /// Samples may be dropped. No retransmission is attempted.
    BestEffort,
    /// Samples are retransmitted until acknowledged or the blocking time
    /// elapses.
    Reliable {
        /// The maximum time a write call blocks when the writer's resource
        /// limits are reached.
        max_blocking_time: Duration,
    },
}

impl AsFfi for Reliability {
    type Target<'a> = (
        cyclonedds_sys::dds_reliability_kind_t,
        cyclonedds_sys::dds_duration_t,
    );

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            Reliability::BestEffort => (
                cyclonedds_sys::dds_reliability_kind_DDS_RELIABILITY_BEST_EFFORT,
                0,
            ),
            Reliability::Reliable { max_blocking_time } => (
                cyclonedds_sys::dds_reliability_kind_DDS_RELIABILITY_RELIABLE,
                max_blocking_time.inner,
            ),
        }
    }
}

/// A hint to the transport layer about the relative send priority of this
/// entity.
///
/// Higher values indicate higher priority. The interpretation is
/// transport-dependent and not guaranteed to be honored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransportPriority {
    /// The priority value. Higher values indicate higher priority.
    pub priority: i32,
}

impl AsFfi for TransportPriority {
    type Target<'a> = i32;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.priority
    }
}

/// The maximum duration a sample remains valid after being written.
///
/// Samples that have not been delivered within their lifespan are silently
/// expired.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lifespan {
    /// The maximum age of a sample before it is considered expired.
    pub duration: Duration,
}

impl AsFfi for Lifespan {
    type Target<'a> = cyclonedds_sys::dds_duration_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.duration.inner
    }
}

/// Controls the order in which samples are delivered to a reader when multiple
/// writers produce samples for the same instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestinationOrder {
    /// Samples are ordered by the time they were received by the reader.
    ByReceptionTimestamp,
    /// Samples are ordered by the timestamp set by the writer at publication
    /// time.
    BySourceTimestamp,
}

impl AsFfi for DestinationOrder {
    type Target<'a> = cyclonedds_sys::dds_destination_order_kind_t;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            DestinationOrder::ByReceptionTimestamp =>
                cyclonedds_sys::dds_destination_order_kind_DDS_DESTINATIONORDER_BY_RECEPTION_TIMESTAMP,
            DestinationOrder::BySourceTimestamp =>
                cyclonedds_sys::dds_destination_order_kind_DDS_DESTINATIONORDER_BY_SOURCE_TIMESTAMP,
        }
    }
}

/// Controls how many samples are stored per instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum History {
    /// All samples are retained, subject to [`ResourceLimits`].
    KeepAll,
    /// Only the `depth` most recent samples per instance are retained.
    KeepLast {
        /// The number of samples to retain per instance.
        depth: i32,
    },
}

impl AsFfi for History {
    type Target<'a> = (cyclonedds_sys::dds_history_kind_t, i32);

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        match self {
            History::KeepAll => (cyclonedds_sys::dds_history_kind_DDS_HISTORY_KEEP_ALL, 0),
            History::KeepLast { depth } => (
                cyclonedds_sys::dds_history_kind_DDS_HISTORY_KEEP_LAST,
                *depth,
            ),
        }
    }
}

/// Caps on the number of instances, samples, and samples per instance.
///
/// When a limit is reached, incoming samples are rejected and the
/// [`SampleRejected`](crate::status::SampleRejected) event fires. Use
/// [`ResourceLimit::Unlimited`] to impose no cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Maximum total number of samples across all instances.
    pub max_samples: ResourceLimit,
    /// Maximum number of instances.
    pub max_instances: ResourceLimit,
    /// Maximum number of samples per instance.
    pub max_samples_per_instance: ResourceLimit,
}

/// A resource limit value, either bounded or unlimited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLimit {
    /// No limit is imposed.
    Unlimited,
    /// The resource is capped at the given value.
    Limited(u32),
}

impl ResourceLimit {
    #[must_use]
    fn as_ffi(self) -> i32 {
        /// This is an invalid value on the Cyclone C side and will defer the
        /// failure of the resource limit down to the later calls which are able
        /// to correctly propagate an error out.
        const INVALID_LIMIT_IN_CYCLONE_C_LIB: i32 = 0;
        match self {
            ResourceLimit::Unlimited => cyclonedds_sys::DDS_LENGTH_UNLIMITED,
            ResourceLimit::Limited(limit) => {
                i32::try_from(limit).unwrap_or(INVALID_LIMIT_IN_CYCLONE_C_LIB)
            }
        }
    }
}

impl AsFfi for ResourceLimits {
    type Target<'a> = (i32, i32, i32);

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        (
            self.max_samples.as_ffi(),
            self.max_instances.as_ffi(),
            self.max_samples_per_instance.as_ffi(),
        )
    }
}

/// Controls whether child entities are automatically enabled on creation.
///
/// When `autoenable_created_entities` is `false`, entities must be explicitly
/// enabled before they can communicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityFactory {
    /// If `true`, entities are enabled immediately on creation.
    pub autoenable_created_entities: bool,
}

impl AsFfi for EntityFactory {
    type Target<'a> = bool;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.autoenable_created_entities
    }
}

/// Controls how the writer handles instances when it is deleted.
///
/// When `autodispose_unregistered_instances` is `true`, the writer
/// automatically disposes all instances it owns on deletion, notifying readers
/// that the data is no longer available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriterDataLifecycle {
    /// If `true`, all owned instances are disposed when the writer is deleted.
    pub autodispose_unregistered_instances: bool,
}

impl AsFfi for WriterDataLifecycle {
    type Target<'a> = bool;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        self.autodispose_unregistered_instances
    }
}

/// Controls how the reader handles stale instance data after writers disappear.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReaderDataLifecycle {
    /// How long samples for an instance are retained after all matching writers
    /// have gone away.
    pub autopurge_nowriter_samples_delay: Duration,
    /// How long samples for a disposed instance are retained before being
    /// purged from the reader cache.
    pub autopurge_disposed_samples_delay: Duration,
}

impl AsFfi for ReaderDataLifecycle {
    type Target<'a> = (
        cyclonedds_sys::dds_duration_t,
        cyclonedds_sys::dds_duration_t,
    );

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        (
            self.autopurge_nowriter_samples_delay.inner,
            self.autopurge_disposed_samples_delay.inner,
        )
    }
}

// TODO validate the following QoS
// ///
// pub enum IgnoreLocal {
//     ///
//     Nothing,
//     ///
//     Participant,
//     ///
//     Process,
// }

// ///
// pub enum TypeConsistency {
//     ///
//     DisallowTypeCoercion {
//         ///
//         force_type_validation: bool,
//     },
//     ///
//     AllowTypeCoercion {
//         ///
//         ignore_sequence_bounds: bool,
//         ///
//         ignore_string_bounds: bool,
//         ///
//         ignore_member_names: bool,
//         ///
//         prevent_type_widening: bool,
//         ///
//         force_type_validation: bool,
//     },
// }

// ///
// pub struct WriterBatching {
//     ///
//     pub batch_updates: bool,
// }

// ///
// pub struct PsmxInstances {
//     ///
//     pub instances: Vec<String>,
// }

// ///
// pub enum DataRepresentationKind {
//     ///
//     Xcdr1,
//     ///
//     Xml,
//     ///
//     Xcdr2,
// }

// ///
// pub struct DataRepresentation {
//     ///
//     pub representations: std::collections::HashSet<DataRepresentationKind>,
// }

/// Assigns a human-readable name to an entity.
///
/// Used in diagnostics, logging, and monitoring tools to identify entities
/// by name rather than by handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityName {
    /// The name to assign to the entity.
    pub name: String,
}

impl AsFfi for EntityName {
    type Target<'a> = std::ffi::CString;

    #[inline]
    fn as_ffi(&self) -> Self::Target<'_> {
        // TODO should this be moved to the construction of the name policy or deferred
        // to the set_qos + construction of the objects with the QoS?O
        std::ffi::CString::new(self.name.as_str())
            .expect("unable to safely create std::ffi::CString from entity name")
    }
}
