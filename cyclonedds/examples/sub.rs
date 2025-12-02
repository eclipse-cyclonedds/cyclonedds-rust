//! Example DDS subscriber using [`Sensor`] as the topic type.
//!
//! Waits for a publisher to match, then takes samples until 100 have been
//! received.

mod common;
use common::Sensor;
use cyclonedds::sample::View;
use cyclonedds::{Domain, Duration, Participant, QoS, Reader, Topic, WaitSet, qos};

fn main() -> cyclonedds::Result<()> {
    let domain = Domain::default();
    let participant = Participant::new(&domain)?;
    let topic = Topic::<Sensor>::new(&participant, "Sensor")?;
    let qos = QoS::new()
        .with_durability(qos::policy::Durability::TransientLocal)
        .with_history(qos::policy::History::KeepAll);
    let reader = Reader::builder(&topic).with_qos(&qos).build()?;

    let mut waitset = WaitSet::<()>::new(&participant)?;
    waitset.attach(&reader, None)?;

    eprintln!("waiting to match publisher");
    waitset.wait(Duration::INFINITE)?;

    eprintln!("publisher matched: started polling");
    let mut received = 0;
    while received < 100 {
        for sample in reader.take()? {
            match sample.view() {
                View::Sample(sample) => {
                    println!("{received}: sample: {sample:#?}");
                }
                View::Key(key) => {
                    println!("{received}: key: {key:?}");
                }
            }
            received += 1;
        }
    }

    Ok(())
}
