//! Example DDS publisher using [`Sensor`] as the topic type.
//!
//! Waits for a subscriber to match, then writes 100 [`Sensor`] samples at
//! roughly 100 Hz.

mod common;
use common::Sensor;
use cyclonedds::{Domain, Duration, Participant, QoS, Topic, WaitSet, Writer, qos};

fn main() -> cyclonedds::Result<()> {
    let domain = Domain::default();
    let participant = Participant::new(&domain)?;
    let topic = Topic::<Sensor>::new(&participant, "Sensor")?;

    let qos = QoS::new().with_durability(qos::policy::Durability::TransientLocal);
    let writer = Writer::builder(&topic).with_qos(&qos).build()?;

    let mut waitset = WaitSet::<()>::new(&participant)?;
    waitset.attach(&writer, None)?;

    eprintln!("waiting to match subscriber");
    waitset.wait(Duration::INFINITE)?;

    eprintln!("subscriber matched: started writing");
    for i in 0..100 {
        writer.write(&Sensor {
            id: i,
            message: format!("🦀 |=>sample [{i}] from Rust"),
            #[allow(clippy::cast_precision_loss)]
            data: std::array::from_fn(|reading| reading as f32 + i as f32),
        })?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Ok(())
}
