# Rust Binding for Eclipse Cyclone DDS

[![License][epl-license-shield]][epl-license]
[![License][edl-license-shield]][edl-license]
[![Latest Version][crates.io-shield]][crates.io]
[![Documentation][docs.rs-shield]][docs.rs]
[![Build Status][check-workflow-status-shield]][check-workflow]
[![Website][cyclonedds-homepage-shield]][cyclonedds-homepage]
[![Dependency Status][deps.rs-shield]][deps.rs]
[![Community][community-shield]][community]

The official Rust binding for [Eclipse Cyclone DDS][cyclonedds-github].

- [Quick Start](#quick-start)
- [Overview of DDS](#overview-of-dds)
- [Example](#example)

## Quick Start

Install the C library and headers to your system path or use the bundled-sources
via the `vendored` feature. Then add the package to your `Cargo.toml` via:

```toml
[dependencies]
# This expects you to have an installation of the Cyclone DDS C library available
# which allows you to tailor the underlying C library as you wish.
#
# NOTE: you can also point to Cyclone DDS using the `CYCLONEDDS_HOME` variable if
# performing a full installation is undesirable.
eclipse-cyclonedds = "0.0.1"

# Using the `vendored` feature will result in us compiling and linking in a version
# of Cyclone DDS for you.
eclipse-cyclonedds = { version = "0.0.1", features = ["vendored"] }
```

Then see [the example](#example) and [the docs][docs.rs] to get started.

## Overview of DDS

The [Data Distribution Service][dds-spec] (DDS) is a publish-subscribe
middleware standard for real-time, data-centric communication. It is used in a
variety of mission critical applications in domains such as aerospace, defense,
autonomous systems (e.g. vehicles, robotics), industrial control, smart energy
grids, transportation, simulation, and medical devices.

[`Participants`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:domain_participant)
within a specific
[`Domain`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:domain_dds)
discover each other automatically via the DDSI/RTPS discovery protocol. Once two
endpoints sharing the same topic name, type information, and compatible
[`Quality of Service` (`QoS`)](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:q:quality_of_service_qos_policies)
discover each other, the middleware establishes a connection between them.
[`Publishers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:p:publisher)
and
[`Subscribers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:s:subscriber)
allow you to group
[`Writers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:data_writer)
and
[`Readers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:data_reader)
respectively to allow you to set their collective behavior. These
[`Writers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:data_writer)
and
[`Readers`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:d:data_reader)
exchange typed samples via
[`Topics`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:t:topic).

```text
                            DOMAIN
                               │
            ┌──────────────────┴──────────────────┐
            │                                     │
       PARTICIPANT                           PARTICIPANT
            │      T ≡ struct Position {x, y}     │
       ┌────┴────┐                           ┌────┴────┐
       │         │                           │         │
  PUBLISHER   TOPIC<T> ═══════════════════ TOPIC<T>  SUBSCRIBER
       │         ║                           ║         │
       │     "Position"                 "Position"     │
       │         ║                           ║         │
    WRITER<T> ═══╝                           ╚═══ READER<T>
         ╰───────── matched via Topic<T> ─────────╯
         Node 01                               Node 02
        ─────────                             ─────────
```

Data delivery characteristics, such as how samples are buffered, retransmitted,
and received, are controlled via
[`Quality of Service`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:02_quality_of_service:start),
a collection of
[`QoS policies`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:q:quality_of_service_qos_policies)
that configure characteristics such as:

- [`durability`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:02_quality_of_service:durability)
  (whether late-joining readers receive historical samples)
- [`reliability`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:02_quality_of_service:reliability)
  (best-effort vs reliable delivery)
- [`history depth`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:02_quality_of_service:history)
  (the number of samples to store in history)
- [`deadline`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:02_quality_of_service:deadline)
  (whether a signal should be generated when a sample is not received within a
  specified period) Policies are set independently on the writer and reader
  side, and compatibility is checked at discovery time. A writer's offered `QoS`
  must be compatible with a reader's requested `QoS` for the two endpoints to
  match.

There are a variety of other elements to the DDS API such as:

- [`WaitSets`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:w:waitset):
  to allow you to block until a particular status occurs on a DDS entity.
- [`Listeners`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:l:listener&s%5B%5D=listener):
  to notify applications of a change in the status of a particular entity.
- [`GuardConditions`, `StatusConditions`, `ReadConditions`, and `QueryConditions`](https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:06_append:glossary:c:condition&s%5B%5D=guard&s%5B%5D=condition):
  Mechanisms to trigger the condition associated with a waitset.

See the [DDS Specification][dds-spec] and the [OMG DDS Wiki][omg-dds-wiki] for
these other elements and see the [Rust Documentation][docs.rs] for what is
supported by this API.

<!-- TODO add details on the footguns (default QoS, read and take semantics) -->

## Example

```rust
use cyclonedds::QoS;
use cyclonedds::qos::policy;
use cyclonedds::sample::View;
use cyclonedds::{Domain, Duration, Participant, Reader, Topic, Writer, Topicable};

#[derive(Topicable, serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
struct Sensor {
    #[dds(key)]
    id: u32,
    temperature: f32,
}

fn main() -> cyclonedds::Result<()> {
    let domain = Domain::default();
    let participant = Participant::new(&domain)?;
    let topic = Topic::<Sensor>::new(&participant, "Sensor")?;

    let qos = QoS::new()
        .with_durability(policy::Durability::TransientLocal)
        .with_history(policy::History::KeepLast { depth: 10 })
        .with_reliability(policy::Reliability::Reliable {
            max_blocking_time: Duration::from_millis(100),
        });

    let reader = Reader::builder(&topic).with_qos(&qos).build()?;
    let writer = Writer::builder(&topic).with_qos(&qos).build()?;

    writer.write(&Sensor {
        id: 1,
        temperature: 21.5,
    })?;

    writer.write(&Sensor {
        id: 2,
        temperature: 35.5,
    })?;

    // Get available samples from the reader history (draining them from the
    // reader).
    //
    // NOTE: use reader.read() to leave them in place for future reads.
    for sample in reader.take()? {
        // `sample` is a SampleOrKey<Sensor> which allows you to:
        //    - distinguish between the sample and key-only cases
        //    - access fields directly (with non-key fields default-initialized
        //      for key-only samples)
        //    - access sample info via `.info()`

        // You can precisely match on the result of `.view()` to handle
        // either case explicitly. This replaces checking the sample info for the
        // `valid_data` field (which does not exist in the Rust API).
        match sample.view() {
            View::Sample(sample) if sample.temperature > 30.0 => {
                println!("sample[{}] is hot: {}°C", sample.id, sample.temperature)
            }
            View::Sample(sample) => {
                println!("sample[{}] is cool: {}°C", sample.id, sample.temperature)
            }
            View::Key(key) => println!(
                "received a key-only sample due to an unregister or dispose: {key:?}"
            ),
        }

        // Alternatively, you can directly access the fields with key-only samples
        // having their non-key fields default-initialized via
        // `Topicable::from_key`.
        if sample.temperature > 30.0 {
            println!("sample[{}] is hot: {}°C", sample.id, sample.temperature);
        }

        // Access the sample info.
        println!(
            "sample[{}] was produced at: {:?}",
            sample.id,
            sample.info().source_timestamp
        );
    }
    Ok(())
}
```

## Minimum supported Rust version (MSRV)

For now, the MSRV is the latest stable Rust version at the time of release.

## References

- [Cyclone DDS][cyclonedds-github]
- [Cyclone DDS Documentation][cyclonedds-docs]
- [OMG DDS Specification][dds-spec]
- [OMG DDSI-RTPS specification][ddsi-rtps-spec]
- [OMG DDS Wiki][omg-dds-wiki]
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

[check-workflow]: https://github.com/eclipse-cyclonedds/cyclonedds-rust/actions/workflows/check.yml
[check-workflow-status-shield]: https://img.shields.io/github/actions/workflow/status/eclipse-cyclonedds/cyclonedds-rust/check.yml
[community]: https://discord.gg/4QQvWZrFKF
[community-shield]: https://img.shields.io/discord/960814229844291604.svg?logo=discord
[crates.io]: https://crates.io/crates/eclipse-cyclonedds
[crates.io-shield]: https://img.shields.io/crates/v/eclipse-cyclonedds.svg
[cyclonedds-docs]: https://cyclonedds.io/docs
[cyclonedds-github]: https://github.com/eclipse-cyclonedds/cyclonedds
[cyclonedds-homepage]: https://cyclonedds.io
[cyclonedds-homepage-shield]: https://img.shields.io/badge/web-cyclonedds.io-blue
[dds-spec]: https://www.omg.org/spec/DDS/1.4/About-DDS/
[ddsi-rtps-spec]: https://www.omg.org/spec/DDSI-RTPS/
[deps.rs]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust
[deps.rs-shield]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust/status.svg
[docs.rs]: https://docs.rs/eclipse-cyclonedds
[docs.rs-shield]: https://docs.rs/eclipse-cyclonedds/badge.svg
[edl-license]: https://choosealicense.com/licenses/edl-1.0/
[edl-license-shield]: https://img.shields.io/badge/license-EDL%201.0-blue
[epl-license]: https://choosealicense.com/licenses/epl-2.0/
[epl-license-shield]: https://img.shields.io/badge/license-EPL%202.0-blue
[omg-dds-wiki]: https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:01_front:4_toc
