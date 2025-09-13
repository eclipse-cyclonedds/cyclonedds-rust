//! Integration tests for DDS read/write scenarios.

// NOTE: active lint levels are defined in the top-level workspace `Cargo.toml`.
// These `allow`s for tests exists for lints that significantly reduce test
// readability or ergonomics.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

use cyclonedds as dds;
use cyclonedds::cdr_bounds::CdrBounds;
use cyclonedds::entity::Entity;

mod common;

/// Verify round-trip read/write with keyhash generation enabled across two
/// externally matched domains.
#[test]
fn read_write_generate_keyhash() -> dds::Result<()> {
    let domain_id_01 = common::domain::unique_id();
    let domain_id_02 = common::domain::unique_id();
    let external_domain_id = domain_id_02;
    let topic_name = common::topic::unique_name();

    let config = &format!(
        "<Domain>
           <General>
             <Interfaces>
               <NetworkInterface address='127.0.0.1' />
             </Interfaces>
           </General>
           <Discovery>
             <ExternalDomainId>
               {external_domain_id}
             </ExternalDomainId>
           </Discovery>
           <Internal>
             <GenerateKeyhash>
               true
             </GenerateKeyhash>
           </Internal>
           <Tracing>
             <Verbosity>
               finest
             </Verbosity>
             <OutputFile>
               stderr
             </OutputFile>
           </Tracing>
         </Domain>"
    );

    let qos = dds::QoS::new()
        .with_history(dds::qos::policy::History::KeepAll)
        .with_durability(dds::qos::policy::Durability::TransientLocal);

    let domain = dds::Domain::new_with_xml_config(domain_id_01, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::<common::topic::Data>::new(&participant, &topic_name)?;
    let reader = dds::Reader::new(&topic)?;

    let domain = dds::Domain::new_with_xml_config(domain_id_02, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::<common::topic::Data>::new(&participant, &topic_name)?;
    let writer = dds::Writer::new(&topic)?;

    writer.set_status_mask(dds::Status::PublicationMatched)?;
    let mut waitset = dds::WaitSet::<()>::new(&participant)?;
    waitset.attach(&writer, None)?;
    waitset.wait(std::time::Duration::from_secs(1).try_into()?)?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let sample = common::topic::Data {
        x: 12,
        y: 35,
        message: format!("message from {topic_name}"),
    };
    writer.write(&sample)?;

    while reader.peek()?.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let samples = reader.read()?;
    assert_eq!(samples.len(), 1);
    assert_eq!(*samples[0], sample);
    Ok(())
}

/// Verify that two distinct Rust types (with distinct backing sertypes) that
/// share the same CDR memory layout and DDS type name can interoperate on the
/// same topic.
#[test]
fn read_write_same_type_shape_different_sertype() -> dds::Result<()> {
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
    struct Data01 {
        x: u32,
        y: u32,
        z: u32,
    }

    impl dds::Topicable for Data01 {
        type Key = ();

        fn from_key((): &Self::Key) -> Self {
            Self::default()
        }

        fn as_key(&self) -> Self::Key {}

        fn dds_type_name() -> impl AsRef<str> {
            "Data"
        }
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
    struct Data02 {
        x: u32,
        y: u32,
        z: u32,
    }

    impl dds::Topicable for Data02 {
        type Key = ();

        fn from_key((): &Self::Key) -> Self {
            Self::default()
        }

        fn as_key(&self) -> Self::Key {}

        fn dds_type_name() -> impl AsRef<str> {
            "Data"
        }
    }

    impl std::cmp::PartialEq<Data02> for Data01 {
        fn eq(&self, other: &Data02) -> bool {
            self.x == other.x && self.y == other.y && self.z == other.z
        }
    }

    let domain_id = common::domain::unique_id();
    let topic_name = common::topic::unique_name();
    let domain = dds::Domain::new(domain_id)?;
    let participant = dds::Participant::new(&domain)?;
    let topic01 = dds::Topic::<Data01>::new(&participant, &topic_name)?;
    let topic02 = dds::Topic::<Data02>::new(&participant, &topic_name)?;

    let writer = dds::Writer::new(&topic01)?;
    let reader = dds::Reader::new(&topic02)?;

    let sample = Data01 {
        x: 1234,
        y: 5678,
        z: 9_101_112,
    };

    writer.write(&sample)?;

    let samples = reader.take()?;

    assert_eq!(samples.len(), 1);
    assert_eq!(sample, *samples[0]);

    Ok(())
}

/// Verify single-sample and multi-sample read/write scenarios across two
/// externally matched domains.
#[test]
fn read_write() -> dds::Result<()> {
    let domain_id_01 = common::domain::unique_id();
    let domain_id_02 = common::domain::unique_id();
    let external_domain_id = domain_id_02;
    let topic_name = common::topic::unique_name();

    let config = &format!(
        "<Domain>
          <General>
            <Interfaces>
              <NetworkInterface address='127.0.0.1' />
            </Interfaces>
          </General>
          <Discovery>
            <ExternalDomainId>
              {external_domain_id}
            </ExternalDomainId>
          </Discovery>
        </Domain>"
    );

    let qos = dds::QoS::new()
        .with_history(dds::qos::policy::History::KeepAll)
        .with_durability(dds::qos::policy::Durability::TransientLocal);

    let domain = dds::Domain::new_with_xml_config(domain_id_01, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::<common::topic::Data>::new(&participant, &topic_name)?;
    let reader = dds::Reader::new(&topic)?;

    let domain = dds::Domain::new_with_xml_config(domain_id_02, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::new(&participant, &topic_name)?;
    let writer = dds::Writer::new(&topic)?;

    writer.set_status_mask(dds::Status::PublicationMatched)?;
    let mut waitset = dds::WaitSet::<()>::new(&participant)?;
    waitset.attach(&writer, None)?;
    waitset.wait(std::time::Duration::from_secs(1).try_into()?)?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let sample = common::topic::Data {
        x: 12,
        y: 35,
        message: format!("message from {topic_name}"),
    };
    writer.write(&sample)?;

    while reader.peek()?.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let samples = reader.take()?;
    assert_eq!(samples.len(), 1);
    assert_eq!(*samples[0], sample);

    let samples = reader.take()?;
    assert!(samples.is_empty());

    let sample_01 = common::topic::Data::default();
    let sample_02 = common::topic::Data {
        x: 100,
        y: 200,
        message: "This is a sample: 🐋鯨❤️".to_string(),
    };

    writer.write(&sample_01)?;
    writer.write(&sample_02)?;

    while reader.peek()?.len() < 2 {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let samples = reader.read()?;

    assert_eq!(samples.len(), 2);
    assert_eq!(*samples[0], sample_01);
    assert_eq!(*samples[1], sample_02);

    let sample_03 = common::topic::Data {
        x: 101,
        y: 202,
        message: "🐋鯨❤️".to_string(),
    };
    writer.write(&sample_03)?;

    while reader.peek()?.len() < 3 {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let samples = reader.take()?;

    assert_eq!(samples.len(), 3);
    assert_eq!(*samples[0], sample_01);
    assert_eq!(*samples[1], sample_02);
    assert_eq!(*samples[2], sample_03);

    let samples = reader.read()?;
    assert_eq!(samples.len(), 0);

    Ok(())
}

/// Verify round-trip read/write scenarios on keyed data where the type has a
/// key containing unbounded fields.
///
/// The backing key is mocked so that the implementation of [`CdrBounds`] is
/// runtime configurable so we can hit edgecases.
#[test]
fn read_write_generate_keyhash_unbounded_keyed_data() -> dds::Result<()> {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default)]
    pub struct UnboundedKeyedData {
        pub x: u32,
        pub y: i32,
        pub tags: Vec<u32>,
        pub message: String,
    }

    static MOCKED_MAX_SERIALIZED_CDR_SIZE: std::sync::Mutex<usize> =
        std::sync::Mutex::new(std::mem::size_of::<MockedKey>());

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq, Default, Hash)]
    pub struct MockedKey(u32, i32, Vec<u128>);

    impl CdrBounds for MockedKey {
        fn max_serialized_cdr_size() -> dds::cdr_bounds::CdrSize {
            let size = *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap();
            dds::cdr_bounds::CdrSize::Bounded(size)
        }

        fn alignment() -> usize {
            8
        }
    }

    impl dds::Topicable for UnboundedKeyedData {
        type Key = MockedKey;

        fn from_key(key: &Self::Key) -> Self {
            Self {
                x: key.0,
                y: key.1,
                ..Default::default()
            }
        }

        fn as_key(&self) -> Self::Key {
            MockedKey(self.x, self.y, vec![])
        }
    }

    let domain_id_01 = common::domain::unique_id();
    let domain_id_02 = common::domain::unique_id();
    let external_domain_id = domain_id_02;
    let topic_name = common::topic::unique_name();

    let config = &format!(
        "<Domain>
           <General>
             <Interfaces>
               <NetworkInterface address='127.0.0.1' />
             </Interfaces>
           </General>
           <Discovery>
             <ExternalDomainId>
               {external_domain_id}
             </ExternalDomainId>
           </Discovery>
           <Internal>
             <GenerateKeyhash>
               true
             </GenerateKeyhash>
           </Internal>
         </Domain>"
    );

    let qos = dds::QoS::new()
        .with_history(dds::qos::policy::History::KeepAll)
        .with_durability(dds::qos::policy::Durability::TransientLocal);

    let domain = dds::Domain::new_with_xml_config(domain_id_01, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::<UnboundedKeyedData>::new(&participant, &topic_name)?;
    let reader = dds::Reader::new(&topic)?;

    let domain = dds::Domain::new_with_xml_config(domain_id_02, config)?;
    let participant = dds::Participant::builder(&domain).with_qos(&qos).build()?;
    let topic = dds::Topic::<UnboundedKeyedData>::new(&participant, &topic_name)?;
    let writer = dds::Writer::new(&topic)?;

    writer.set_status_mask(dds::Status::PublicationMatched)?;
    let mut waitset = dds::WaitSet::<()>::new(&participant)?;
    waitset.attach(&writer, None)?;
    waitset.wait(std::time::Duration::from_secs(1).try_into()?)?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let sample = UnboundedKeyedData {
        x: 12,
        y: 35,
        tags: vec![5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5],
        message: "This is a sample: 🐋鯨❤️".to_string(),
    };
    writer.write(&sample)?;

    while reader.peek()?.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let samples = reader.read()?;
    assert_eq!(samples.len(), 1);
    assert_eq!(*samples[0], sample);

    *MOCKED_MAX_SERIALIZED_CDR_SIZE.lock().unwrap() = 32;
    let sample = UnboundedKeyedData {
        x: 12,
        y: 35,
        tags: vec![5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5],
        message: "This is a sample: 🐋鯨❤️".to_string(),
    };
    writer.write(&sample)?;

    while reader.peek()?.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let samples = reader.read()?;
    assert_eq!(samples.len(), 1);
    assert_eq!(*samples[0], sample);

    Ok(())
}
