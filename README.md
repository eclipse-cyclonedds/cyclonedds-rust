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
- [Shared memory (PSMX)](#shared-memory-psmx)
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
eclipse-cyclonedds = "11.1.0"

# Using the `vendored` feature will result in us compiling and linking in a version
# of Cyclone DDS for you.
eclipse-cyclonedds = { version = "11.1.0", features = ["vendored"] }
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

## Shared memory (PSMX)

Cyclone DDS can hand data transfer off to a *Pub/Sub Message Exchange* (PSMX)
plugin instead of sending it over the network, and shared memory is one such
plugin. This is configured on the domain rather than in code, through a
`PubSubMessageExchange` element:

```rust
use cyclonedds::{Domain, Participant, Topic, Writer};

let config = r#"
    <Domain>
      <General>
        <Interfaces>
          <PubSubMessageExchange type="iox" config="INSTANCE_NAME=shm;" />
        </Interfaces>
      </General>
    </Domain>"#;

let domain = Domain::new_with_xml_config(0, config)?;
```

Cyclone DDS looks for the plugin in a library named `psmx_<type>` (overridable
with a `library` attribute) and calls `<type>_create_psmx` in it. The
iceoryx-backed plugin, `psmx_iox`, ships with Cyclone DDS but is built only when
its dependencies are available, so check your installation before relying on it.

A matched reader and writer use PSMX when both resolve to the same PSMX Instance
*and* it reports the same locator at both ends — for shared memory, that means
the same machine. Otherwise they fall back to the network transparently. To check
that the configuration actually took effect, ask the endpoint:

```rust
let writer = Writer::new(&topic)?;

if writer.is_shared_memory_available() {
    // Samples written here can travel through shared memory.
}
```

By default an endpoint may use every PSMX Instance configured on its domain that
supports its topic type and `QoS`. Use the `PsmxInstances` policy to narrow that
down, or to opt an endpoint out entirely with an empty list:

```rust
use cyclonedds::QoS;
use cyclonedds::qos::policy;

let restricted = QoS::new().with_psmx_instances(policy::PsmxInstances {
    instances: vec!["shm".to_string()],
});

let opted_out = QoS::new().with_psmx_instances(policy::PsmxInstances {
    instances: Vec::new(),
});
```

By default this still serializes. Rust topic types are encoded through `serde`,
so a sample is serialized by the writer and deserialized by the reader even when
the route between them is shared memory. What you gain is skipping the network
stack and its copies. To also skip the encoding, see below.

### Zero-copy

A topic can travel as raw memory instead of CDR, which removes the encoding and
the copies entirely: the writer fills a buffer that *is* the shared memory readers
look at.

That requires a type whose in-memory representation can be shared verbatim with
another process. Derive
[`FixedSize`](https://docs.rs/eclipse-cyclonedds/latest/cyclonedds/trait.FixedSize.html)
and the layout is checked for you: `#[repr(C)]`, every field's type shareable in
turn, no padding, and alignment within what a loan guarantees. Break any of those
and it is a compile error rather than a comment.

```rust
use cyclonedds::builder::TopicBuilder;
use cyclonedds::{FixedSize, Topicable, Writer};

#[derive(Topicable, FixedSize, serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default)]
#[repr(C)]
struct Reading {
    #[dds(key)]
    sensor_id: u32,
    sequence: u32,
    value: f64,
}

// Opt the topic in. Without this the topic behaves like any other, because Rust
// cannot tell from a `T: Topicable` bound alone whether `T` is also `FixedSize`.
let topic = TopicBuilder::<Reading>::new(&participant, "Readings")
    .with_zero_copy()
    .build()?;
let writer = Writer::new(&topic)?;

// Borrow shared memory and fill the sample in place.
let mut loan = writer.loan()?;
loan.write(Reading {
    sensor_id: 1,
    sequence: 7,
    value: 21.5,
});
writer.write_loaned(loan)?;
```

For a large payload, prefer `loan.as_uninit_mut()` over `loan.write(..)`: the
former writes the payload straight into shared memory, while the latter builds
the sample first and then moves it in.

Both ends must opt in — a reader only accepts a raw payload on a topic that
agreed to the raw representation. Endpoints that did not opt in still interoperate
normally, exchanging CDR as before, and a zero-copy topic falls back to CDR
whenever PSMX is not usable for a given pair. `Writer::loan` also works with no
PSMX interface configured, borrowing heap memory instead; the programming model is
the same, the benefit is not.

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

### Shared memory silently not being used

Configuring a `PubSubMessageExchange` element is not enough on its own. The plugin
library has to be loadable, both ends have to resolve to the same PSMX Instance,
and its locators have to match. When any of that does not hold, Cyclone DDS falls
back to the network without complaint and the only symptom is that you do not get
the performance you expected. Check
[`is_shared_memory_available`](https://docs.rs/eclipse-cyclonedds/latest/cyclonedds/struct.Writer.html#method.is_shared_memory_available)
on both endpoints rather than assuming.

Note also that shared memory removes network overhead but not serialization unless
the topic opted into zero-copy; see [Shared memory (PSMX)](#shared-memory-psmx).

### Implementing `FixedSize` by hand

Prefer the derive. It checks `repr(C)`, that every field is itself shareable, the
absence of padding, and the alignment bound — the whole contract except that the
type holds no raw pointers, which nothing can check. Asserting it by hand for a
type that does not qualify is undefined behaviour, and it may only show up in
whichever process *reads* the sample.

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
