//! Integration tests for shared-memory (PSMX) data transfer.
//!
//! Cyclone DDS off-loads data transfer to a Pub/Sub Message Exchange (PSMX)
//! plugin when a matched reader and writer resolve to the same PSMX Instance.
//! These tests drive that path with the in-process plugin from the
//! `eclipse-cyclonedds-psmx-test` crate, which exercises the same Cyclone DDS
//! machinery as a real shared-memory plugin without needing a daemon in CI.
//!
//! Each pair of domains carries a *different* `Discovery` tag, so the two never
//! discover each other over RTPS and no network path exists between them. A
//! sample that arrives therefore proves the shared-memory path ran.
//!
//! Whether it ran *raw* (zero-copy) or *serialized* is a further question, and
//! the plugin answers it: it records the state Cyclone DDS put in each loan to a
//! trace file, so the assertions look at the transport rather than at anything
//! inside the binding.

// NOTE: active lint levels are defined in the top-level workspace `Cargo.toml`.
// These `allow`s for tests exists for lints that significantly reduce test
// readability or ergonomics.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

use cyclonedds as dds;

mod common;

/// The `type` attribute of the `PubSubMessageExchange` element, which also
/// determines the constructor symbol Cyclone DDS looks up
/// (`rusttest_create_psmx`).
const PSMX_TYPE: &str = "rusttest";

/// How long to wait for a sample before giving up.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Locate the built PSMX plugin.
///
/// It is a `cdylib` from this workspace, named as a dev-dependency of
/// `eclipse-cyclonedds` purely so Cargo has built it by the time this runs. Cargo
/// does not report where it put it, but in the ordinary build layout it sits
/// alongside an ancestor of the test binary. Deliberately no broader search than
/// that — a target directory can hold several builds made with different
/// `RUSTFLAGS`, and loading the wrong one fails confusingly. See the
/// `psmx-tests` feature for which configurations these tests run in.
fn plugin_path() -> std::path::PathBuf {
    let file_name = if cfg!(target_os = "windows") {
        "psmx_rusttest.dll"
    } else if cfg!(target_os = "macos") {
        "libpsmx_rusttest.dylib"
    } else {
        "libpsmx_rusttest.so"
    };

    let executable = std::env::current_exe().expect("could not determine the test executable path");

    executable
        .ancestors()
        .map(|directory| directory.join(file_name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| {
            panic!(
                "could not find the PSMX test plugin ({file_name}) alongside any ancestor of {}",
                executable.display()
            )
        })
}

/// What the PSMX plugin recorded about the samples that crossed it.
///
/// The plugin appends one `<topic> <state>` line per published sample, which is
/// how these tests tell a raw (zero-copy) transfer from a serialized one without
/// instrumenting the binding.
struct Trace {
    path: std::path::PathBuf,
}

impl Trace {
    fn new() -> Self {
        Self {
            path: std::env::temp_dir().join(format!(
                "cyclonedds-psmx-{}.trace",
                common::topic::unique_name()
            )),
        }
    }

    /// The loan states recorded for `topic_name`, in order.
    fn states(&self, topic_name: &str) -> Vec<String> {
        std::fs::read_to_string(&self.path)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| line.strip_prefix(topic_name))
            .map(|state| state.trim().to_string())
            .collect()
    }
}

impl Drop for Trace {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// A domain configuration that loads the test PSMX plugin.
///
/// `seed` becomes the 16 byte PSMX Locator: two domains pair up through a PSMX
/// Instance when they report the same one. `tag` isolates the domain on the
/// network, so domains with different tags never discover each other over RTPS.
fn config(seed: u8, tag: &str, trace: &Trace) -> String {
    let library = plugin_path();
    let library = library.display();
    let locator = format!("{seed:02x}").repeat(16);
    let trace = trace.path.display();

    format!(
        "<Domain>
           <General>
             <Interfaces>
               <NetworkInterface address='127.0.0.1' />
               <PubSubMessageExchange type='{PSMX_TYPE}'
                                      library='{library}'
                                      priority='1000000'
                                      config='LOCATOR={locator};INSTANCE_NAME=shm;TRACE={trace};' />
             </Interfaces>
           </General>
           <Discovery>
             <Tag>{tag}</Tag>
           </Discovery>
         </Domain>"
    )
}

/// Bring up a writer in one domain and a reader in another, both on `$name`.
///
/// A function cannot return these: a `Reader` borrows its `Topic`, which borrows
/// its `Participant`, which borrows its `Domain`. A macro expands the `let`
/// bindings into the caller's own frame, where the borrows are all fine, and
/// macro hygiene keeps the intermediates invisible but alive.
macro_rules! paired_endpoints {
    ($trace:expr, $seed:expr, $tag:literal, $topic:path, $name:expr => $reader:ident, $writer:ident) => {
        paired_endpoints!($trace, $seed, $tag, $topic, $name, None => $reader, $writer);
    };
    ($trace:expr, $seed:expr, $tag:literal, $topic:path, $name:expr, $qos:expr => $reader:ident, $writer:ident) => {
        let writer_domain = dds::Domain::new_with_xml_config(
            common::domain::unique_id(),
            &config($seed, concat!($tag, "-writer"), $trace),
        )?;
        let reader_domain = dds::Domain::new_with_xml_config(
            common::domain::unique_id(),
            &config($seed, concat!($tag, "-reader"), $trace),
        )?;

        let qos: Option<&dds::QoS> = $qos;

        let reader_participant = dds::Participant::new(&reader_domain)?;
        let reader_topic = $topic(&reader_participant, $name)?;
        let $reader = match qos {
            Some(qos) => dds::Reader::builder(&reader_topic).with_qos(qos).build()?,
            None => dds::Reader::new(&reader_topic)?,
        };

        let writer_participant = dds::Participant::new(&writer_domain)?;
        let writer_topic = $topic(&writer_participant, $name)?;
        let $writer = match qos {
            Some(qos) => dds::Writer::builder(&writer_topic).with_qos(qos).build()?,
            None => dds::Writer::new(&writer_topic)?,
        };

        // Both ends should have resolved to the plugin, which advertises shared
        // memory. Without this the rest of the test would be meaningless.
        assert!(
            $writer.is_shared_memory_available(),
            "the writer did not resolve to a shared-memory PSMX instance"
        );
        assert!(
            $reader.is_shared_memory_available(),
            "the reader did not resolve to a shared-memory PSMX instance"
        );
    };
}

/// Wait for exactly one sample and check it, draining the reader.
fn expect_one_sample<T, R>(reader: &dds::Reader<'_, '_, '_, T, R>, expected: &T)
where
    T: dds::Topicable + PartialEq,
    T::Key: cyclonedds::cdr_bounds::CdrBounds,
    R: dds::representation::Representation,
{
    await_sample(|| !reader.peek().expect("failed to peek the reader").is_empty());

    let samples = reader.take().expect("failed to take from the reader");
    assert_eq!(samples.len(), 1, "expected exactly one sample");
    assert!(*samples[0] == *expected, "the sample did not round-trip");
}

/// Wait until `arrived` reports a sample, or give up.
fn await_sample(mut arrived: impl FnMut() -> bool) {
    let deadline = std::time::Instant::now() + TIMEOUT;
    while !arrived() {
        assert!(
            std::time::Instant::now() < deadline,
            "no sample arrived over PSMX within {TIMEOUT:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// Wait for exactly one loaned sample, without draining the reader.
fn await_one_loaned_sample<'r, T, R>(
    reader: &'r dds::Reader<'_, '_, '_, T, R>,
) -> Vec<dds::LoanedSample<'r, T>>
where
    T: dds::Topicable,
    T::Key: cyclonedds::cdr_bounds::CdrBounds,
    R: dds::representation::Representation,
{
    await_sample(|| {
        !reader
            .peek_loaned()
            .expect("failed to peek the reader")
            .is_empty()
    });

    let samples = reader
        .take_loaned()
        .expect("failed to take loaned samples from the reader");
    assert_eq!(samples.len(), 1, "expected exactly one sample");
    samples
}

/// A keyed topic type whose in-memory representation may be shared verbatim.
///
/// `derive(FixedSize)` checks the layout: `repr(C)`, no padding, every field
/// itself shareable, and alignment within what a loan guarantees.
#[derive(
    cyclonedds::Topicable,
    cyclonedds::FixedSize,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
)]
#[repr(C)]
struct Reading {
    #[dds(key)]
    sensor_id: u32,
    sequence: u32,
    value: f64,
}

/// An *unkeyed* zero-copy type, which takes a different path: the sertype reports
/// no key, so the raw key handling never comes into play.
#[derive(
    cyclonedds::Topicable,
    cyclonedds::FixedSize,
    serde::Serialize,
    serde::Deserialize,
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
)]
#[repr(C)]
struct Frame {
    pixels: [u8; 16],
}

/// A zero-copy topic: naming `ZeroCopy` is what makes the sertype memcpy-safe.
fn zero_copy_topic<'d, 'p, T>(
    participant: &'p dds::Participant<'d>,
    topic_name: &str,
) -> dds::Result<dds::Topic<'d, 'p, T, dds::ZeroCopy>>
where
    T: dds::FixedSizeTopic,
{
    dds::builder::TopicBuilder::<T, dds::ZeroCopy>::zero_copy(participant, topic_name).build()
}

/// An ordinary topic, which travels as CDR even over PSMX.
fn plain_topic<'d, 'p>(
    participant: &'p dds::Participant<'d>,
    topic_name: &str,
) -> dds::Result<dds::Topic<'d, 'p, common::topic::Data>> {
    dds::Topic::new(participant, topic_name)
}

/// An ordinary type crosses between domains over PSMX, as CDR.
///
/// The trace confirms the payload was serialized: this type owns a `String`, so
/// it can never be shared as raw memory.
#[test]
fn psmx_delivers_a_serialized_sample() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x01, "serialized", plain_topic, &topic_name => reader, writer);

    let sample = common::topic::Data {
        x: 12,
        y: -35,
        message: format!("shared memory message from {topic_name}"),
    };
    writer.write(&sample)?;
    expect_one_sample(&reader, &sample);

    assert_eq!(trace.states(&topic_name), ["SERIALIZED_DATA"]);
    Ok(())
}

/// A zero-copy reader still accepts what a serialized writer sends.
///
/// Only this direction mixes: a zero-copy writer sends raw memory, which a reader
/// on a serialized topic has no way to interpret.
#[test]
fn psmx_zero_copy_reader_accepts_a_serialized_writer() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    let writer_domain = dds::Domain::new_with_xml_config(
        common::domain::unique_id(),
        &config(0x0f, "mixed-writer", &trace),
    )?;
    let reader_domain = dds::Domain::new_with_xml_config(
        common::domain::unique_id(),
        &config(0x0f, "mixed-reader", &trace),
    )?;

    let reader_participant = dds::Participant::new(&reader_domain)?;
    let reader_topic = zero_copy_topic::<Reading>(&reader_participant, &topic_name)?;
    let reader = dds::Reader::new(&reader_topic)?;

    let writer_participant = dds::Participant::new(&writer_domain)?;
    let writer_topic = dds::Topic::<Reading>::new(&writer_participant, &topic_name)?;
    let writer = dds::Writer::new(&writer_topic)?;

    let expected = Reading {
        sensor_id: 3,
        sequence: 5,
        value: 8.0,
    };
    writer.write(&expected)?;

    expect_one_sample(&reader, &expected);
    assert_eq!(trace.states(&topic_name), ["SERIALIZED_DATA"]);
    Ok(())
}

/// Several samples of differing serialized length arrive, in order.
///
/// Length matters: the writing side splits the four byte encapsulation header off
/// and carries it in the loan metadata, so a bug in reassembling it would show up
/// on samples of some lengths but not others.
#[test]
fn psmx_delivers_multiple_samples() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    let qos = dds::QoS::new().with_history(dds::qos::policy::History::KeepAll);
    paired_endpoints!(&trace, 0x02, "multi", plain_topic, &topic_name, Some(&qos) => reader, writer);

    let expected: Vec<_> = (0..5_u32)
        .map(|index| common::topic::Data {
            x: index,
            y: -index.cast_signed(),
            // Deliberately varies the serialized length between samples.
            message: "y".repeat(index as usize * 7),
        })
        .collect();

    for sample in &expected {
        writer.write(sample)?;
    }

    let deadline = std::time::Instant::now() + TIMEOUT;
    let mut received = Vec::new();
    while received.len() < expected.len() {
        assert!(
            std::time::Instant::now() < deadline,
            "only {} of {} samples arrived over PSMX",
            received.len(),
            expected.len()
        );
        received.extend(
            reader
                .take()?
                .into_iter()
                .filter_map(dds::sample::SampleOrKey::into_sample),
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    assert_eq!(received, expected);
    Ok(())
}

/// A sample built directly in loaned shared memory crosses as raw memory.
///
/// This is zero-copy end to end: the writer fills the sample in place, Cyclone
/// DDS marks the loan as holding raw application data, and the reader rebuilds it
/// through `from_psmx` with no decoding. The trace is what proves the payload was
/// raw rather than serialized — the sample alone would look identical either way.
#[test]
fn psmx_zero_copy_delivers_a_loaned_sample() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x03, "loaned", zero_copy_topic, &topic_name => reader, writer);

    let expected = Reading {
        sensor_id: 7,
        sequence: 42,
        value: -3.5,
    };

    // Fill the sample in the loaned memory itself. For a large payload this is
    // the whole point: the bytes are written once, where the reader reads them.
    let mut loan = writer.loan()?;
    let sample = loan.as_uninit_mut().write(Reading::default());
    sample.sensor_id = expected.sensor_id;
    sample.sequence = expected.sequence;
    sample.value = expected.value;
    writer.write_loaned(loan)?;

    expect_one_sample(&reader, &expected);
    assert_eq!(trace.states(&topic_name), ["RAW_DATA"]);
    Ok(())
}

/// An ordinary write on a zero-copy topic also crosses as raw memory.
///
/// Cyclone DDS copies the sample into a PSMX loan itself here rather than
/// serializing it, which exercises a different path in the C library — and checks
/// that the binding hands over a pointer to a bare `Reading` rather than to its
/// own internal sample representation.
#[test]
fn psmx_zero_copy_delivers_a_plain_write() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x04, "plain", zero_copy_topic, &topic_name => reader, writer);

    let expected = Reading {
        sensor_id: 11,
        sequence: 13,
        value: f64::MIN_POSITIVE,
    };
    writer.write(&expected)?;

    expect_one_sample(&reader, &expected);
    assert_eq!(trace.states(&topic_name), ["RAW_DATA"]);
    Ok(())
}

/// Zero-copy works for an unkeyed topic too.
///
/// An unkeyed sertype reports no key, so this covers a different configuration
/// from the keyed cases above.
#[test]
fn psmx_zero_copy_delivers_an_unkeyed_sample() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x05, "unkeyed", zero_copy_topic, &topic_name => reader, writer);

    let mut expected = Frame::default();
    expected.pixels[0] = 0xff;
    expected.pixels[15] = 0x0f;

    let mut loan = writer.loan()?;
    loan.write(expected);
    writer.write_loaned(loan)?;

    expect_one_sample(&reader, &expected);
    assert_eq!(trace.states(&topic_name), ["RAW_DATA"]);
    Ok(())
}

/// Dropping a loan instead of publishing it delivers nothing, and leaves the
/// writer usable.
#[test]
fn psmx_zero_copy_dropped_loan_publishes_nothing() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x06, "dropped", zero_copy_topic, &topic_name => reader, writer);

    {
        let mut loan = writer.loan()?;
        loan.write(Reading {
            sensor_id: 1,
            sequence: 1,
            value: 1.0,
        });
        // Dropped here without publishing, returning the memory to the writer.
    }

    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        reader.peek()?.is_empty(),
        "a loan that was never published should not deliver a sample"
    );
    assert!(
        trace.states(&topic_name).is_empty(),
        "nothing should have crossed the transport"
    );

    // Taking another loan afterwards still works.
    let expected = Reading {
        sensor_id: 2,
        sequence: 2,
        value: 2.0,
    };
    let mut loan = writer.loan()?;
    loan.write(expected);
    writer.write_loaned(loan)?;
    expect_one_sample(&reader, &expected);

    Ok(())
}

/// An endpoint opted out of PSMX through the `QoS` policy does not use shared
/// memory, even though its domain has a PSMX Instance configured.
#[test]
fn psmx_instances_qos_can_opt_out() -> dds::Result<()> {
    let trace = Trace::new();
    let domain = dds::Domain::new_with_xml_config(
        common::domain::unique_id(),
        &config(0x07, "opt-out", &trace),
    )?;
    let participant = dds::Participant::new(&domain)?;
    let topic = plain_topic(&participant, &common::topic::unique_name())?;

    // An empty instance list keeps the endpoint off PSMX entirely.
    let opted_out = dds::QoS::new().with_psmx_instances(dds::qos::policy::PsmxInstances {
        instances: Vec::new(),
    });
    let writer = dds::Writer::builder(&topic).with_qos(&opted_out).build()?;
    assert!(
        !writer.is_shared_memory_available(),
        "a writer with an empty PSMX instance list should not use shared memory"
    );

    // Naming the configured instance opts back in.
    let opted_in = dds::QoS::new().with_psmx_instances(dds::qos::policy::PsmxInstances {
        instances: vec!["shm".to_string()],
    });
    let writer = dds::Writer::builder(&topic).with_qos(&opted_in).build()?;
    assert!(
        writer.is_shared_memory_available(),
        "a writer naming the configured PSMX instance should use shared memory"
    );

    Ok(())
}

/// Loans work with no PSMX interface configured, falling back to heap memory.
///
/// Cyclone DDS makes that substitution silently, so the borrow-then-publish model
/// has to keep working — it just is not zero-copy. This is also the only path
/// that reaches the `from_loaned_sample` serdata op.
#[test]
fn zero_copy_loan_falls_back_to_the_heap_without_psmx() -> dds::Result<()> {
    // No `PubSubMessageExchange` element at all.
    let domain = dds::Domain::new_with_xml_config(
        common::domain::unique_id(),
        "<Domain>
           <General>
             <Interfaces>
               <NetworkInterface address='127.0.0.1' />
             </Interfaces>
           </General>
         </Domain>",
    )?;

    let participant = dds::Participant::new(&domain)?;
    let topic = zero_copy_topic::<Reading>(&participant, &common::topic::unique_name())?;
    let reader = dds::Reader::new(&topic)?;
    let writer = dds::Writer::new(&topic)?;

    assert!(
        !writer.is_shared_memory_available(),
        "there is no PSMX instance configured, so none should be available"
    );

    let expected = Reading {
        sensor_id: 5,
        sequence: 9,
        value: 1.25,
    };
    let mut loan = writer.loan()?;
    loan.write(expected);
    writer.write_loaned(loan)?;

    expect_one_sample(&reader, &expected);
    Ok(())
}

/// A loan can only be taken on a topic that opted into zero-copy.
///
/// The refusal itself is a compile error rather than a runtime one — see the
/// `compile_fail` example on [`Writer::loan`] — because `loan` exists only on a
/// `Writer<T, ZeroCopy>`. What this checks is the other half: that the opted-in
/// topic really does hand loans out, so the type-level gate is not simply
/// rejecting everything.
#[test]
fn loans_are_available_on_a_zero_copy_topic() -> dds::Result<()> {
    let trace = Trace::new();
    let domain = dds::Domain::new_with_xml_config(
        common::domain::unique_id(),
        &config(0x08, "opt-in", &trace),
    )?;
    let participant = dds::Participant::new(&domain)?;

    let topic = zero_copy_topic::<Reading>(&participant, &common::topic::unique_name())?;
    let writer = dds::Writer::new(&topic)?;
    assert!(writer.loan().is_ok());

    Ok(())
}

/// A borrowing read on a zero-copy topic hands back the writer's own memory.
///
/// This is the end-to-end zero-copy claim, checked directly rather than inferred:
/// the address the writer filled in through its loan is the address the reader
/// derefs to. Nothing copied the payload at any point between them.
#[test]
fn psmx_zero_copy_take_loaned_borrows_the_writers_memory() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x09, "borrowed", zero_copy_topic::<Reading>, &topic_name => reader, writer);

    let expected = Reading {
        sensor_id: 42,
        sequence: 3,
        value: 12.5,
    };

    let mut loan = writer.loan()?;
    let published = loan.write(expected);
    let published_address = std::ptr::from_ref(published).addr();
    writer.write_loaned(loan)?;

    let samples = await_one_loaned_sample(&reader);
    let sample = &samples[0];

    assert_eq!(**sample, expected);
    assert!(!sample.is_key());
    assert_eq!(
        std::ptr::from_ref(&**sample).addr(),
        published_address,
        "the reader should borrow the writer's memory, not a copy of it"
    );
    assert_eq!(trace.states(&topic_name), ["RAW_DATA"]);

    Ok(())
}

/// A borrowing read works on an ordinary topic too, without the zero-copy part.
///
/// The payload was deserialized into the reader's own memory, so there is nothing
/// shared to borrow — but the read still avoids copying it back out again, which
/// is the part that applies to every topic.
#[test]
fn psmx_take_loaned_works_on_a_serialized_topic() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x0a, "borrowed-cdr", plain_topic, &topic_name => reader, writer);

    let expected = common::topic::Data {
        x: 5,
        y: 7,
        message: format!("borrowed from {topic_name}"),
    };
    writer.write(&expected)?;

    let samples = await_one_loaned_sample(&reader);
    assert_eq!(samples[0].x, expected.x);
    assert_eq!(samples[0].y, expected.y);
    assert_eq!(samples[0].message, expected.message);
    assert!(!samples[0].is_key());
    assert_eq!(trace.states(&topic_name), ["SERIALIZED_DATA"]);

    Ok(())
}

/// A disposed instance arrives as a key-only sample, which has no payload to
/// borrow and so is materialized from the key.
#[test]
fn psmx_zero_copy_take_loaned_materializes_a_key_only_sample() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(
        &trace,
        0x0b,
        "borrowed-key",
        zero_copy_topic::<Reading>,
        &topic_name => reader,
        writer
    );

    let sample = Reading {
        sensor_id: 11,
        sequence: 1,
        value: 3.5,
    };
    writer.write(&sample)?;
    let samples = await_one_loaned_sample(&reader);
    assert_eq!(*samples[0], sample);
    drop(samples);

    // Disposing the instance sends the key alone.
    writer.dispose(&dds::Topicable::as_key(&sample))?;

    let samples = await_one_loaned_sample(&reader);
    assert!(
        samples[0].is_key(),
        "a dispose notification carries only the key"
    );
    // The materialized sample is what `Topicable::from_key` builds.
    assert_eq!(samples[0].sensor_id, sample.sensor_id);
    // And the key itself comes back either way.
    assert_eq!(samples[0].key(), dds::Topicable::as_key(&sample));

    assert_eq!(trace.states(&topic_name), ["RAW_DATA", "RAW_KEY"]);

    Ok(())
}

/// Loaned samples go back when they drop, so the writer's pool keeps turning
/// over.
///
/// A retained loan is memory the writer's PSMX Instance cannot reuse. Publishing
/// and borrowing far more samples than the plugin has blocks for only works if
/// each borrow is released on drop.
#[test]
fn psmx_zero_copy_loaned_samples_are_released_on_drop() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x0c, "released", zero_copy_topic::<Reading>, &topic_name => reader, writer);

    for sequence in 0..64 {
        let sample = Reading {
            sensor_id: 1,
            sequence,
            value: f64::from(sequence),
        };
        writer.write(&sample)?;

        let samples = await_one_loaned_sample(&reader);
        assert_eq!(samples[0].sequence, sequence);
        // Dropped here, which is what hands the loan back.
    }

    assert_eq!(trace.states(&topic_name).len(), 64);

    Ok(())
}

/// `read_loaned` leaves the sample in the cache, so the same payload can be
/// borrowed more than once at a time.
///
/// This is the case `take_loaned` cannot reach: two live `LoanedSample`s sharing
/// one serdata. Both must borrow the same memory, and the second must still be
/// valid after the first goes away.
#[test]
fn psmx_zero_copy_read_loaned_can_borrow_one_sample_twice() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x0d, "reread", zero_copy_topic::<Reading>, &topic_name => reader, writer);

    let expected = Reading {
        sensor_id: 7,
        sequence: 2,
        value: 9.5,
    };

    let mut loan = writer.loan()?;
    let published_address = std::ptr::from_ref(loan.write(expected)).addr();
    writer.write_loaned(loan)?;

    // Wait without consuming, so the sample stays available to read again.
    await_sample(|| !reader.peek_loaned().expect("failed to peek").is_empty());

    let first = reader.read_loaned()?;
    let second = reader.read_loaned()?;
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);

    // Both borrows are of the writer's memory, and of the same address.
    assert_eq!(std::ptr::from_ref(&*first[0]).addr(), published_address);
    assert_eq!(std::ptr::from_ref(&*second[0]).addr(), published_address);
    assert_eq!(*first[0], expected);

    // The surviving borrow stays valid once the other is released.
    drop(first);
    assert_eq!(*second[0], expected);
    assert_eq!(std::ptr::from_ref(&*second[0]).addr(), published_address);

    // And a take afterwards still sees it, since reads do not consume.
    let taken = reader.take_loaned()?;
    assert_eq!(taken.len(), 1);
    assert_eq!(*taken[0], expected);

    assert_eq!(trace.states(&topic_name), ["RAW_DATA"]);

    Ok(())
}

/// `peek_loaned` neither consumes the sample nor marks it read.
#[test]
fn psmx_peek_loaned_leaves_the_sample_unread() -> dds::Result<()> {
    let trace = Trace::new();
    let topic_name = common::topic::unique_name();
    paired_endpoints!(&trace, 0x0e, "peeked", plain_topic, &topic_name => reader, writer);

    let expected = common::topic::Data {
        x: 1,
        y: 2,
        message: format!("peeked from {topic_name}"),
    };
    writer.write(&expected)?;

    await_sample(|| !reader.peek_loaned().expect("failed to peek").is_empty());

    // Peeking repeatedly keeps returning it, and leaves it unread.
    for _ in 0..3 {
        let peeked = reader.peek_loaned()?;
        assert_eq!(peeked.len(), 1);
        assert_eq!(peeked[0].message, expected.message);
        assert!(
            peeked[0].info().state.contains(dds::State::SampleFresh),
            "peek must leave the sample unread"
        );
    }

    // Reading marks it read; peeking again shows that.
    assert_eq!(reader.read_loaned()?.len(), 1);
    let peeked = reader.peek_loaned()?;
    assert!(
        peeked[0].info().state.contains(dds::State::SampleStale),
        "the sample should be marked read after a read"
    );

    Ok(())
}
