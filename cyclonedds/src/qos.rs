//! Quality of Service ([`QoS`]) policies for DDS entities.
//!
//! [`QoS`] is built through a chainable builder and applied to entities via
//! their respective builders. Policies are defined in the [`policy`] submodule.
//!
//! # Examples
//!
//! ```
//! use cyclonedds::qos::policy;
//! use cyclonedds::{Duration, QoS};
//!
//! let qos = QoS::new()
//!     .with_durability(policy::Durability::TransientLocal)
//!     .with_history(policy::History::KeepAll)
//!     .with_reliability(policy::Reliability::Reliable {
//!         max_blocking_time: Duration::INFINITE,
//!     });
//! ```

pub mod policy;

use crate::internal::ffi;
use crate::internal::traits::AsFfi;

/// A set of Quality of Service [`policies`](policy) applied to a DDS entity.
///
/// Constructed via [`QoS::new`] and configured through chainable `with_*`
/// methods. Unset policies inherit the defaults for the entity type they are
/// applied to.
#[derive(Debug)]
pub struct QoS {
    pub(crate) inner: cyclonedds_sys::dds_qos_t,

    user_data: Option<policy::UserData>,
    topic_data: Option<policy::TopicData>,
    group_data: Option<policy::GroupData>,
    durability: Option<policy::Durability>,
    durability_service: Option<policy::DurabilityService>,
    presentation: Option<policy::Presentation>,
    deadline: Option<policy::Deadline>,
    latency_budget: Option<policy::LatencyBudget>,
    ownership: Option<policy::Ownership>,
    liveliness: Option<policy::Liveliness>,
    time_based_filter: Option<policy::TimeBasedFilter>,
    partition: Option<policy::Partition>,
    reliability: Option<policy::Reliability>,
    transport_priority: Option<policy::TransportPriority>,
    lifespan: Option<policy::Lifespan>,
    destination_order: Option<policy::DestinationOrder>,
    history: Option<policy::History>,
    resource_limits: Option<policy::ResourceLimits>,
    entity_factory: Option<policy::EntityFactory>,
    writer_data_lifecycle: Option<policy::WriterDataLifecycle>,
    reader_data_lifecycle: Option<policy::ReaderDataLifecycle>,
    entity_name: Option<policy::EntityName>,
}

impl std::default::Default for QoS {
    fn default() -> Self {
        Self {
            inner: cyclonedds_sys::dds_qos_t {
                present: 0,
                aliased: 0,
                ..Default::default()
            },
            user_data: Option::default(),
            topic_data: Option::default(),
            group_data: Option::default(),
            durability: Option::default(),
            durability_service: Option::default(),
            presentation: Option::default(),
            deadline: Option::default(),
            latency_budget: Option::default(),
            ownership: Option::default(),
            liveliness: Option::default(),
            time_based_filter: Option::default(),
            partition: Option::default(),
            reliability: Option::default(),
            transport_priority: Option::default(),
            lifespan: Option::default(),
            destination_order: Option::default(),
            history: Option::default(),
            resource_limits: Option::default(),
            entity_factory: Option::default(),
            writer_data_lifecycle: Option::default(),
            reader_data_lifecycle: Option::default(),
            entity_name: Option::default(),
        }
    }
}

impl QoS {
    /// Creates a new [`QoS`] with no policies set.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    ///
    /// let qos = QoS::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`UserData`](policy::UserData) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_user_data(policy::UserData {
    ///     value: b"v1.0".to_vec(),
    /// });
    /// ```
    #[must_use]
    pub fn with_user_data(mut self, user_data: policy::UserData) -> Self {
        ffi::dds_qos_set_user_data(&mut self.inner, user_data.as_ffi());
        self.user_data = Some(user_data);
        self
    }

    /// Sets the [`TopicData`](policy::TopicData) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_topic_data(policy::TopicData {
    ///     value: b"sensor-schema".to_vec(),
    /// });
    /// ```
    #[must_use]
    pub fn with_topic_data(mut self, topic_data: policy::TopicData) -> Self {
        ffi::dds_qos_set_topic_data(&mut self.inner, topic_data.as_ffi());
        self.topic_data = Some(topic_data);
        self
    }

    /// Sets the [`GroupData`](policy::GroupData) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_group_data(policy::GroupData {
    ///     value: b"group-a".to_vec(),
    /// });
    /// ```
    #[must_use]
    pub fn with_group_data(mut self, group_data: policy::GroupData) -> Self {
        ffi::dds_qos_set_group_data(&mut self.inner, group_data.as_ffi());
        self.group_data = Some(group_data);
        self
    }

    /// Sets the [`Durability`](policy::Durability) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_durability(policy::Durability::TransientLocal);
    /// ```
    #[must_use]
    pub fn with_durability(mut self, durability: policy::Durability) -> Self {
        ffi::dds_qos_set_durability(&mut self.inner, durability.as_ffi());
        self.durability = Some(durability);
        self
    }

    /// Sets the [`DurabilityService`](policy::DurabilityService) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_durability_service(policy::DurabilityService {
    ///     service_cleanup_delay: Duration::from_millis(200),
    ///     history: policy::History::KeepLast { depth: 10 },
    ///     resource_limits: policy::ResourceLimits {
    ///         max_samples: policy::ResourceLimit::Unlimited,
    ///         max_instances: policy::ResourceLimit::Unlimited,
    ///         max_samples_per_instance: policy::ResourceLimit::Unlimited,
    ///     },
    /// });
    /// ```
    #[must_use]
    pub fn with_durability_service(
        mut self,
        durability_service: policy::DurabilityService,
    ) -> Self {
        let (
            service_cleanup_delay,
            history_kind,
            history_depth,
            max_samples,
            max_instances,
            max_samples_per_instance,
        ) = durability_service.as_ffi();

        ffi::dds_qos_set_durability_service(
            &mut self.inner,
            service_cleanup_delay,
            history_kind,
            history_depth,
            max_samples,
            max_instances,
            max_samples_per_instance,
        );
        self.durability_service = Some(durability_service);
        self
    }

    /// Sets the [`Presentation`](policy::Presentation) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_presentation(policy::Presentation::Topic {
    ///     coherent_access: true,
    ///     ordered_access: true,
    /// });
    /// ```
    #[must_use]
    pub fn with_presentation(mut self, presentation: policy::Presentation) -> Self {
        let (access_scope, coherent_access, ordered_access) = presentation.as_ffi();
        ffi::dds_qos_set_presentation(
            &mut self.inner,
            access_scope,
            coherent_access,
            ordered_access,
        );
        self.presentation = Some(presentation);
        self
    }

    /// Sets the [`Deadline`](policy::Deadline) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_deadline(policy::Deadline {
    ///     period: Duration::from_millis(100),
    /// });
    /// ```
    #[must_use]
    pub fn with_deadline(mut self, deadline: policy::Deadline) -> Self {
        let period = deadline.as_ffi();
        ffi::dds_qos_set_deadline(&mut self.inner, period);
        self.deadline = Some(deadline);
        self
    }

    /// Sets the [`LatencyBudget`](policy::LatencyBudget) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_latency_budget(policy::LatencyBudget {
    ///     duration: Duration::from_millis(10),
    /// });
    /// ```
    #[must_use]
    pub fn with_latency_budget(mut self, latency_budget: policy::LatencyBudget) -> Self {
        let duration = latency_budget.as_ffi();
        ffi::dds_qos_set_latency_budget(&mut self.inner, duration);
        self.latency_budget = Some(latency_budget);
        self
    }

    /// Sets the [`Ownership`](policy::Ownership) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_ownership(policy::Ownership::Exclusive { strength: 10 });
    /// ```
    #[must_use]
    pub fn with_ownership(mut self, ownership: policy::Ownership) -> Self {
        let (kind, strength) = ownership.as_ffi();
        ffi::dds_qos_set_ownership(&mut self.inner, kind);
        if let Some(strength) = strength {
            ffi::dds_qos_set_ownership_strength(&mut self.inner, strength);
        }

        self.ownership = Some(ownership);
        self
    }

    /// Sets the [`Liveliness`](policy::Liveliness) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_liveliness(policy::Liveliness::Automatic {
    ///     lease_duration: Duration::from_secs(5),
    /// });
    /// ```
    #[must_use]
    pub fn with_liveliness(mut self, liveliness: policy::Liveliness) -> Self {
        let (kind, lease_duration) = liveliness.as_ffi();
        ffi::dds_qos_set_liveliness(&mut self.inner, kind, lease_duration);
        self.liveliness = Some(liveliness);
        self
    }

    /// Sets the [`TimeBasedFilter`](policy::TimeBasedFilter) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_time_based_filter(policy::TimeBasedFilter {
    ///     minimum_separation: Duration::from_millis(50),
    /// });
    /// ```
    #[must_use]
    pub fn with_time_based_filter(mut self, time_based_filter: policy::TimeBasedFilter) -> Self {
        let minimum_separation = time_based_filter.as_ffi();
        ffi::dds_qos_set_time_based_filter(&mut self.inner, minimum_separation);
        self.time_based_filter = Some(time_based_filter);
        self
    }

    /// Sets the [`Partition`](policy::Partition) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_partition(policy::Partition {
    ///     partitions: vec!["sensors".to_string()],
    /// });
    /// ```
    #[must_use]
    pub fn with_partition(mut self, partition: policy::Partition) -> Self {
        let partitions = partition.as_ffi();
        ffi::dds_qos_set_partition(&mut self.inner, &partitions);
        self.partition = Some(partition);
        self
    }

    /// Sets the [`Reliability`](policy::Reliability) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_reliability(policy::Reliability::Reliable {
    ///     max_blocking_time: Duration::from_millis(100),
    /// });
    /// ```
    #[must_use]
    pub fn with_reliability(mut self, reliability: policy::Reliability) -> Self {
        let (kind, max_blocking_time) = reliability.as_ffi();
        ffi::dds_qos_set_reliability(&mut self.inner, kind, max_blocking_time);
        self.reliability = Some(reliability);
        self
    }

    /// Sets the [`TransportPriority`](policy::TransportPriority) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_transport_priority(policy::TransportPriority { priority: 10 });
    /// ```
    #[must_use]
    pub fn with_transport_priority(
        mut self,
        transport_priority: policy::TransportPriority,
    ) -> Self {
        let lifespan = transport_priority.as_ffi();
        ffi::dds_qos_set_transport_priority(&mut self.inner, lifespan);
        self.transport_priority = Some(transport_priority);
        self
    }

    /// Sets the [`Lifespan`](policy::Lifespan) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_lifespan(policy::Lifespan {
    ///     duration: Duration::from_secs(30),
    /// });
    /// ```
    #[must_use]
    pub fn with_lifespan(mut self, lifespan: policy::Lifespan) -> Self {
        let duration = lifespan.as_ffi();
        ffi::dds_qos_set_lifespan(&mut self.inner, duration);
        self.lifespan = Some(lifespan);
        self
    }

    /// Sets the [`DestinationOrder`](policy::DestinationOrder) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_destination_order(policy::DestinationOrder::BySourceTimestamp);
    /// ```
    #[must_use]
    pub fn with_destination_order(mut self, destination_order: policy::DestinationOrder) -> Self {
        let kind = destination_order.as_ffi();
        ffi::dds_qos_set_destination_order(&mut self.inner, kind);
        self.destination_order = Some(destination_order);
        self
    }

    /// Sets the [`History`](policy::History) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_history(policy::History::KeepLast { depth: 10 });
    /// ```
    #[must_use]
    pub fn with_history(mut self, history: policy::History) -> Self {
        let (kind, depth) = history.as_ffi();
        ffi::dds_qos_set_history(&mut self.inner, kind, depth);
        self.history = Some(history);
        self
    }

    /// Sets the [`ResourceLimits`](policy::ResourceLimits) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_resource_limits(policy::ResourceLimits {
    ///     max_samples: policy::ResourceLimit::Limited(1000),
    ///     max_instances: policy::ResourceLimit::Limited(100),
    ///     max_samples_per_instance: policy::ResourceLimit::Limited(10),
    /// });
    /// ```
    #[must_use]
    pub fn with_resource_limits(mut self, resource_limits: policy::ResourceLimits) -> Self {
        let (max_samples, max_instances, max_samples_per_instance) = resource_limits.as_ffi();
        ffi::dds_qos_set_resource_limits(
            &mut self.inner,
            max_samples,
            max_instances,
            max_samples_per_instance,
        );
        self.resource_limits = Some(resource_limits);
        self
    }

    /// Sets the [`EntityFactory`](policy::EntityFactory) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_entity_factory(policy::EntityFactory {
    ///     autoenable_created_entities: false,
    /// });
    /// ```
    #[must_use]
    pub fn with_entity_factory(mut self, entity_factory: policy::EntityFactory) -> Self {
        let auto_enable_created_entities = entity_factory.as_ffi();
        ffi::dds_qos_set_entity_factory(&mut self.inner, auto_enable_created_entities);
        self.entity_factory = Some(entity_factory);
        self
    }

    /// Sets the [`WriterDataLifecycle`](policy::WriterDataLifecycle) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_writer_data_lifecycle(policy::WriterDataLifecycle {
    ///     autodispose_unregistered_instances: false,
    /// });
    /// ```
    #[must_use]
    pub fn with_writer_data_lifecycle(
        mut self,
        writer_data_lifecycle: policy::WriterDataLifecycle,
    ) -> Self {
        let autodispose = writer_data_lifecycle.as_ffi();
        ffi::dds_qos_set_writer_data_lifecycle(&mut self.inner, autodispose);
        self.writer_data_lifecycle = Some(writer_data_lifecycle);
        self
    }

    /// Sets the [`ReaderDataLifecycle`](policy::ReaderDataLifecycle) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::qos::policy;
    /// use cyclonedds::{Duration, QoS};
    ///
    /// let qos = QoS::new().with_reader_data_lifecycle(policy::ReaderDataLifecycle {
    ///     autopurge_nowriter_samples_delay: Duration::from_secs(5),
    ///     autopurge_disposed_samples_delay: Duration::from_secs(1),
    /// });
    /// ```
    #[must_use]
    pub fn with_reader_data_lifecycle(
        mut self,
        reader_data_lifecycle: policy::ReaderDataLifecycle,
    ) -> Self {
        let (autopurge_nowriter_samples_delay, autopurge_disposed_samples_delay) =
            reader_data_lifecycle.as_ffi();
        ffi::dds_qos_set_reader_data_lifecycle(
            &mut self.inner,
            autopurge_nowriter_samples_delay,
            autopurge_disposed_samples_delay,
        );
        self.reader_data_lifecycle = Some(reader_data_lifecycle);
        self
    }

    /// Sets the [`EntityName`](policy::EntityName) policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::QoS;
    /// use cyclonedds::qos::policy;
    ///
    /// let qos = QoS::new().with_entity_name(policy::EntityName {
    ///     name: "my_writer".to_string(),
    /// });
    /// ```
    #[must_use]
    pub fn with_entity_name(mut self, entity_name: policy::EntityName) -> Self {
        let name = entity_name.as_ffi();
        ffi::dds_qos_set_entity_name(&mut self.inner, &name);
        self.entity_name = Some(entity_name);
        self
    }
}

impl Drop for QoS {
    fn drop(&mut self) {
        ffi::dds_reset_qos(&mut self.inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_set() {
        let user_data = policy::UserData {
            value: Vec::default(),
        };
        let topic_data = policy::TopicData {
            value: Vec::default(),
        };
        let group_data = policy::GroupData {
            value: Vec::default(),
        };
        let durability = policy::Durability::TransientLocal;
        let durability_service = policy::DurabilityService {
            service_cleanup_delay: crate::Duration::default(),
            history: policy::History::KeepAll,
            resource_limits: policy::ResourceLimits {
                max_samples: policy::ResourceLimit::Limited(1),
                max_instances: policy::ResourceLimit::Limited(1),
                max_samples_per_instance: policy::ResourceLimit::Limited(1),
            },
        };
        let presentation = policy::Presentation::Group {
            coherent_access: true,
            ordered_access: true,
        };
        let deadline = policy::Deadline {
            period: crate::Duration::default(),
        };
        let latency_budget = policy::LatencyBudget {
            duration: crate::Duration::default(),
        };
        let ownership = policy::Ownership::Shared;
        let liveliness = policy::Liveliness::Automatic {
            lease_duration: crate::Duration::default(),
        };
        let time_based_filter = policy::TimeBasedFilter {
            minimum_separation: crate::Duration::default(),
        };
        let partition = policy::Partition {
            partitions: Vec::default(),
        };
        let reliability = policy::Reliability::BestEffort;
        let transport_priority = policy::TransportPriority { priority: 0 };
        let lifespan = policy::Lifespan {
            duration: crate::Duration::default(),
        };
        let destination_order = policy::DestinationOrder::ByReceptionTimestamp;
        let history = policy::History::KeepAll;
        let resource_limits = policy::ResourceLimits {
            max_samples: policy::ResourceLimit::Limited(1),
            max_instances: policy::ResourceLimit::Limited(1),
            max_samples_per_instance: policy::ResourceLimit::Limited(1),
        };
        let entity_factory = policy::EntityFactory {
            autoenable_created_entities: false,
        };
        let writer_data_lifecycle = policy::WriterDataLifecycle {
            autodispose_unregistered_instances: true,
        };
        let reader_data_lifecycle = policy::ReaderDataLifecycle {
            autopurge_nowriter_samples_delay: crate::Duration::default(),
            autopurge_disposed_samples_delay: crate::Duration::default(),
        };
        let entity_name = policy::EntityName {
            name: String::default(),
        };

        let qos = QoS::new()
            .with_user_data(user_data.clone())
            .with_topic_data(topic_data.clone())
            .with_group_data(group_data.clone())
            .with_durability(durability)
            .with_durability_service(durability_service)
            .with_presentation(presentation)
            .with_deadline(deadline)
            .with_latency_budget(latency_budget)
            .with_ownership(ownership)
            .with_liveliness(liveliness)
            .with_time_based_filter(time_based_filter)
            .with_partition(partition.clone())
            .with_reliability(reliability)
            .with_transport_priority(transport_priority)
            .with_lifespan(lifespan)
            .with_destination_order(destination_order)
            .with_history(history)
            .with_resource_limits(resource_limits)
            .with_entity_factory(entity_factory)
            .with_writer_data_lifecycle(writer_data_lifecycle)
            .with_reader_data_lifecycle(reader_data_lifecycle)
            .with_entity_name(entity_name.clone());

        assert_eq!(qos.user_data, Some(user_data));
        assert_eq!(qos.topic_data, Some(topic_data));
        assert_eq!(qos.group_data, Some(group_data));
        assert_eq!(qos.durability, Some(durability));
        assert_eq!(qos.durability_service, Some(durability_service));
        assert_eq!(qos.presentation, Some(presentation));
        assert_eq!(qos.deadline, Some(deadline));
        assert_eq!(qos.latency_budget, Some(latency_budget));
        assert_eq!(qos.ownership, Some(ownership));
        assert_eq!(qos.liveliness, Some(liveliness));
        assert_eq!(qos.time_based_filter, Some(time_based_filter));
        assert_eq!(qos.partition, Some(partition));
        assert_eq!(qos.reliability, Some(reliability));
        assert_eq!(qos.transport_priority, Some(transport_priority));
        assert_eq!(qos.lifespan, Some(lifespan));
        assert_eq!(qos.destination_order, Some(destination_order));
        assert_eq!(qos.history, Some(history));
        assert_eq!(qos.resource_limits, Some(resource_limits));
        assert_eq!(qos.entity_factory, Some(entity_factory));
        assert_eq!(qos.writer_data_lifecycle, Some(writer_data_lifecycle));
        assert_eq!(qos.reader_data_lifecycle, Some(reader_data_lifecycle));
        assert_eq!(qos.entity_name, Some(entity_name));
    }

    #[test]
    fn test_qos_set_user_data() {
        let user_data = policy::UserData {
            value: Vec::default(),
        };
        let qos = QoS::new().with_user_data(user_data.clone());
        assert_eq!(qos.user_data, Some(user_data));
    }

    #[test]
    fn test_qos_set_topic_data() {
        let topic_data = policy::TopicData {
            value: Vec::default(),
        };
        let qos = QoS::new().with_topic_data(topic_data.clone());
        assert_eq!(qos.topic_data, Some(topic_data));
    }

    #[test]
    fn test_qos_set_group_data() {
        let group_data = policy::GroupData {
            value: Vec::default(),
        };
        let qos = QoS::new().with_group_data(group_data.clone());
        assert_eq!(qos.group_data, Some(group_data));
    }

    #[test]
    fn test_qos_set_durability() {
        let durability = policy::Durability::Volatile;
        let qos = QoS::new().with_durability(durability);
        assert_eq!(qos.durability, Some(durability));
        let durability = policy::Durability::TransientLocal;
        let qos = QoS::new().with_durability(durability);
        assert_eq!(qos.durability, Some(durability));
        let durability = policy::Durability::Transient;
        let qos = QoS::new().with_durability(durability);
        assert_eq!(qos.durability, Some(durability));
        let durability = policy::Durability::Persistent;
        let qos = QoS::new().with_durability(durability);
        assert_eq!(qos.durability, Some(durability));
    }

    #[test]
    fn test_qos_set_durability_service() {
        let durability_service = policy::DurabilityService {
            service_cleanup_delay: crate::Duration::INFINITE,
            history: policy::History::KeepAll,
            resource_limits: policy::ResourceLimits {
                max_samples: policy::ResourceLimit::Limited(1),
                max_instances: policy::ResourceLimit::Limited(1),
                max_samples_per_instance: policy::ResourceLimit::Limited(1),
            },
        };
        let qos = QoS::new().with_durability_service(durability_service);
        assert_eq!(qos.durability_service, Some(durability_service));
    }

    #[test]
    fn test_qos_set_presentation() {
        let coherent_access = true;
        let ordered_access = true;

        let presentation = policy::Presentation::Instance {
            coherent_access,
            ordered_access,
        };
        let qos = QoS::new().with_presentation(presentation);
        assert_eq!(qos.presentation, Some(presentation));

        let presentation = policy::Presentation::Topic {
            coherent_access,
            ordered_access,
        };
        let qos = QoS::new().with_presentation(presentation);
        assert_eq!(qos.presentation, Some(presentation));

        let presentation = policy::Presentation::Group {
            coherent_access,
            ordered_access,
        };
        let qos = QoS::new().with_presentation(presentation);
        assert_eq!(qos.presentation, Some(presentation));
    }

    #[test]
    fn test_qos_set_deadline() {
        let deadline = policy::Deadline {
            period: crate::Duration::INFINITE,
        };
        let qos = QoS::new().with_deadline(deadline);
        assert_eq!(qos.deadline, Some(deadline));
    }

    #[test]
    fn test_qos_set_latency_budget() {
        let latency_budget = policy::LatencyBudget {
            duration: crate::Duration::INFINITE,
        };
        let qos = QoS::new().with_latency_budget(latency_budget);
        assert_eq!(qos.latency_budget, Some(latency_budget));
    }

    #[test]
    fn test_qos_set_ownership() {
        let ownership = policy::Ownership::Shared;
        let qos = QoS::new().with_ownership(ownership);
        assert_eq!(qos.ownership, Some(ownership));

        let ownership = policy::Ownership::Exclusive { strength: 1 };
        let qos = QoS::new().with_ownership(ownership);
        assert_eq!(qos.ownership, Some(ownership));
    }

    #[test]
    fn test_qos_set_liveliness() {
        let lease_duration = crate::Duration::INFINITE;

        let liveliness = policy::Liveliness::Automatic { lease_duration };
        let qos = QoS::new().with_liveliness(liveliness);
        assert_eq!(qos.liveliness, Some(liveliness));

        let liveliness = policy::Liveliness::ManualByParticipant { lease_duration };
        let qos = QoS::new().with_liveliness(liveliness);
        assert_eq!(qos.liveliness, Some(liveliness));

        let liveliness = policy::Liveliness::ManualByTopic { lease_duration };
        let qos = QoS::new().with_liveliness(liveliness);
        assert_eq!(qos.liveliness, Some(liveliness));
    }

    #[test]
    fn test_qos_set_time_based_filter() {
        let time_based_filter = policy::TimeBasedFilter {
            minimum_separation: crate::Duration::from_nanos(1000),
        };
        let qos = QoS::new().with_time_based_filter(time_based_filter);
        assert_eq!(qos.time_based_filter, Some(time_based_filter));
    }

    #[test]
    fn test_qos_set_partition() {
        let partition = policy::Partition {
            partitions: vec!["A".to_string(), "B".to_string()],
        };
        let qos = QoS::new().with_partition(partition.clone());
        assert_eq!(qos.partition, Some(partition));
    }

    #[test]
    fn test_qos_set_reliability() {
        let reliability = policy::Reliability::BestEffort;
        let qos = QoS::new().with_reliability(reliability);
        assert_eq!(qos.reliability, Some(reliability));

        let reliability = policy::Reliability::Reliable {
            max_blocking_time: crate::Duration::INFINITE,
        };
        let qos = QoS::new().with_reliability(reliability);
        assert_eq!(qos.reliability, Some(reliability));
    }

    #[test]
    fn test_qos_set_transport_priority() {
        let transport_priority = policy::TransportPriority { priority: 1 };
        let qos = QoS::new().with_transport_priority(transport_priority);
        assert_eq!(qos.transport_priority, Some(transport_priority));
    }

    #[test]
    fn test_qos_set_lifespan() {
        let lifespan = policy::Lifespan {
            duration: crate::Duration::INFINITE,
        };
        let qos = QoS::new().with_lifespan(lifespan);
        assert_eq!(qos.lifespan, Some(lifespan));
    }

    #[test]
    fn test_qos_set_destination_order() {
        let destination_order = policy::DestinationOrder::ByReceptionTimestamp;
        let qos = QoS::new().with_destination_order(destination_order);
        assert_eq!(qos.destination_order, Some(destination_order));

        let destination_order = policy::DestinationOrder::BySourceTimestamp;
        let qos = QoS::new().with_destination_order(destination_order);
        assert_eq!(qos.destination_order, Some(destination_order));
    }

    #[test]
    fn test_qos_set_history() {
        let history = policy::History::KeepAll;
        let qos = QoS::new().with_history(history);
        assert_eq!(qos.history, Some(history));

        let history = policy::History::KeepLast { depth: 10 };
        let qos = QoS::new().with_history(history);
        assert_eq!(qos.history, Some(history));
    }

    #[test]
    fn test_qos_set_resource_limits() {
        let resource_limits = policy::ResourceLimits {
            max_samples: policy::ResourceLimit::Limited(1),
            max_instances: policy::ResourceLimit::Limited(1),
            max_samples_per_instance: policy::ResourceLimit::Limited(1),
        };
        let qos = QoS::new().with_resource_limits(resource_limits);
        assert_eq!(qos.resource_limits, Some(resource_limits));
    }

    #[test]
    fn test_qos_set_entity_factory() {
        let entity_factory = policy::EntityFactory {
            autoenable_created_entities: true,
        };
        let qos = QoS::new().with_entity_factory(entity_factory);
        assert_eq!(qos.entity_factory, Some(entity_factory));
    }

    #[test]
    fn test_qos_set_writer_data_lifecycle() {
        let writer_data_lifecycle = policy::WriterDataLifecycle {
            autodispose_unregistered_instances: true,
        };
        let qos = QoS::new().with_writer_data_lifecycle(writer_data_lifecycle);
        assert_eq!(qos.writer_data_lifecycle, Some(writer_data_lifecycle));
    }

    #[test]
    fn test_qos_set_reader_data_lifecycle() {
        let reader_data_lifecycle = policy::ReaderDataLifecycle {
            autopurge_nowriter_samples_delay: crate::Duration::from_nanos(10_000),
            autopurge_disposed_samples_delay: crate::Duration::from_nanos(10_000),
        };
        let qos = QoS::new().with_reader_data_lifecycle(reader_data_lifecycle);
        assert_eq!(qos.reader_data_lifecycle, Some(reader_data_lifecycle));
    }

    #[test]
    fn test_qos_set_entity_name() {
        let entity_name = policy::EntityName {
            name: "my_entity".to_string(),
        };
        let qos = QoS::new().with_entity_name(entity_name.clone());
        assert_eq!(qos.entity_name, Some(entity_name));
    }

    #[test]
    #[should_panic = "unable to safely create std::ffi::CString from entity name"]
    fn test_qos_set_entity_name_with_invalid_name() {
        let entity_name = policy::EntityName {
            name: "\0".to_string(),
        };
        let _ = QoS::new().with_entity_name(entity_name);
    }
}
