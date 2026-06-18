//! Integration tests validating the `[Topicable]` derive macro.

use cyclonedds as dds;
use dds::Topicable;

#[test]
fn test_topicable_on_empty_struct() -> dds::Result<()> {
    #[derive(Topicable, Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
    struct Data {}

    let domain = dds::Domain::default();
    let participant = dds::Participant::new(&domain)?;
    let topic = dds::Topic::<Data>::new(&participant, "data")?;
    let reader = dds::Reader::new(&topic)?;
    let writer = dds::Writer::new(&topic)?;

    let sample = Data {};
    writer.write(&sample).unwrap();

    let samples = reader.read()?;
    assert_eq!(*samples[0], sample);

    Ok(())
}

#[test]
fn test_topicable_on_unkeyed_data() -> dds::Result<()> {
    #[derive(Topicable, Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
    struct Data {
        x: u32,
        y: i32,
    }

    let domain = dds::Domain::default();
    let participant = dds::Participant::new(&domain)?;
    let topic = dds::Topic::<Data>::new(&participant, "data")?;
    let reader = dds::Reader::new(&topic)?;
    let writer = dds::Writer::new(&topic)?;

    let sample = Data { x: 1, y: 2 };
    writer.write(&sample)?;

    let samples = reader.read()?;
    assert_eq!(*samples[0], sample);

    Ok(())
}

#[test]
fn test_topicable_with_topic_name() -> dds::Result<()> {
    #[derive(Topicable, Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
    #[dds(type_name = "custom::Data")]
    struct Data {
        x: u32,
        y: i32,
    }

    let domain = dds::Domain::default();
    let participant = dds::Participant::new(&domain)?;
    let topic = dds::Topic::<Data>::new(&participant, "data")?;
    let reader = dds::Reader::new(&topic)?;
    let writer = dds::Writer::new(&topic)?;

    let sample = Data { x: 1, y: 2 };
    writer.write(&sample)?;

    let samples = reader.read()?;
    assert_eq!(*samples[0], sample);

    Ok(())
}

#[test]
fn test_topicable_with_invalid_topic_name() -> dds::Result<()> {
    #[derive(Topicable, Debug, Clone, serde::Serialize, serde::Deserialize, Default, PartialEq)]
    #[dds(type_name = "custom::Data\0")]
    struct Data {
        x: u32,
        y: i32,
    }

    let domain = dds::Domain::default();
    let participant = dds::Participant::new(&domain)?;
    let error = dds::Topic::<Data>::new(&participant, "data").unwrap_err();
    assert_eq!(error, dds::Error::BadParameter);

    Ok(())
}
