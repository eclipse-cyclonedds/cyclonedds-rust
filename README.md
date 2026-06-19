# Rust Binding for Eclipse Cyclone DDS

[![Latest Version][crates.io-shield]][crates.io]
[![Build Status][check-workflow-status-shield]][check-workflow]
[![Community][community-shield]][community]
[![Website][cyclonedds-homepage-shield]][cyclonedds-homepage]

[![Code Coverage][codecov-shield]][codecov]
[![Documentation][docs.rs-shield]][docs.rs]
[![Dependency Status][deps.rs-shield]][deps.rs]

The official Rust binding for [Eclipse Cyclone DDS][cyclonedds-github].

- [Quick Start](#quick-start)
- [Overview of DDS](#overview-of-dds)
- [Example](#example)
- [Common footguns](#common-footguns)

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
eclipse-cyclonedds = "0.0.4"

# Using the `vendored` feature will result in us compiling and linking in a version
# of Cyclone DDS for you.
eclipse-cyclonedds = { version = "0.0.4", features = ["vendored"] }
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

## Common footguns

### QoS mismatch

A `Writer` and `Reader` only exchange samples after discovery finds matching
topic names, compatible type information, and compatible `QoS` policies. If
samples are not arriving, check the effective `QoS` on both endpoints before
assuming the network or serialization layer is at fault.

### QoS offer/request asymmetry

`QoS` compatibility follows an offer/request model: the writer offers a `QoS`
and the reader requests one. Compatibility is asymmetric. A writer offering
`Reliable` delivery is compatible with a reader requesting `BestEffort`, but not
the inverse. The same asymmetry applies to `Durability`: a writer offering
`TransientLocal` is compatible with a reader requesting `Volatile`, but a reader
requesting `TransientLocal` will not match a writer offering only `Volatile`.

### Durability and late-joining readers

The default durability **does not** make DDS behave like a retained-message
broker. A reader that joins after a writer has already published will not
receive historical samples unless both sides are configured with compatible
durability (`TransientLocal` or stronger) and sufficient history depth.
`Volatile` durability, the default, discards samples the moment no matched
reader exists to receive them.

### Reliability and history depth

Reliable delivery asks the middleware to retransmit missing samples, but does
not provide infinite buffering. The default history policy is `KeepLast` with a
depth of 1, so only the most recent sample is retained per writer instance.
History depth, resource limits, and slow readers all constrain how much data the
middleware retains. A writer paired with a slow reliable reader may block or
drop samples once its send queue is exhausted, depending on the
`max_blocking_time` and resource limit settings in effect.

### Domain ID isolation

Participants on different domain IDs are completely isolated. Discovery will not
cross domain boundaries, so two nodes that are otherwise correctly configured
will be invisible to each other if their domain IDs differ.

### Partition mismatch

Partitions introduce a second matching layer on top of topic name and `QoS`. A
writer and reader on the same topic will not exchange samples unless they share
at least one partition string. Note that the empty string `""` is a valid and
distinct partition.

### Liveliness

`Liveliness` configuration determines when the middleware considers a writer
dead. If the lease duration is shorter than the writer's actual publish rate,
the writer may appear to die and recover under load, causing spurious
matched/unmatched transitions on the reader side.

### `read` vs `take`

`read` and `take` have different cache semantics. `read` leaves matching samples
in the reader cache, so subsequent reads with overlapping state masks may return
the same samples again. `take` removes them, making those samples unavailable to
any later call. Use `take` for queue-like consumption and `read` when you need
to inspect the current state without draining the sample.

Cyclone DDS also includes a `peek` call which reads without updating any sample
or view state, so repeated peeks always see the same samples regardless of state
masks. Use it for non-destructive inspection when state transitions are
undesirable.

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
[check-workflow-status-shield]: https://shieldcn.dev/github/ci/eclipse-cyclonedds/cyclonedds-rust?no-track&mode=light&size=xs
[codecov]: https://codecov.io/github/eclipse-cyclonedds/cyclonedds-rust
[codecov-shield]: https://codecov.io/gh/eclipse-cyclonedds/cyclonedds-rust/graph/badge.svg
[community]: https://discord.gg/4QQvWZrFKF
[community-shield]: https://shieldcn.dev/discord/960814229844291604.svg?no-track&variant=branded&size=xs
[crates.io]: https://crates.io/crates/eclipse-cyclonedds
[crates.io-shield]: https://shieldcn.dev/group/crates/eclipse-cyclonedds+crates/license/eclipse-cyclonedds.svg?no-track&mode=light&size=xs
[cyclonedds-docs]: https://cyclonedds.io/docs
[cyclonedds-github]: https://github.com/eclipse-cyclonedds/cyclonedds
[cyclonedds-homepage]: https://cyclonedds.io
[cyclonedds-homepage-shield]: https://shieldcn.dev/badge/web-cyclonedds.io-blue.svg?no-track&mode=light&logo=lu%3ATornado&size=xs
[dds-spec]: https://www.omg.org/spec/DDS/1.4/About-DDS/
[ddsi-rtps-spec]: https://www.omg.org/spec/DDSI-RTPS/
[deps.rs]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust
[deps.rs-shield]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust/status.svg
[docs.rs]: https://docs.rs/eclipse-cyclonedds
[docs.rs-shield]: https://docs.rs/eclipse-cyclonedds/badge.svg
[omg-dds-wiki]: https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:01_front:4_toc
