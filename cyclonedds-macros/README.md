# Procedural Macros for Eclipse Cyclone DDS

[![Latest Version][crates.io-shield]][crates.io]
[![Build Status][check-workflow-status-shield]][check-workflow]
[![Community][community-shield]][community]
[![Website][cyclonedds-homepage-shield]][cyclonedds-homepage]

[![Code Coverage][codecov-shield]][codecov]
[![Documentation][docs.rs-shield]][docs.rs]
[![Dependency Status][deps.rs-shield]][deps.rs]

Procedural macros for the official Rust binding for
[Eclipse Cyclone DDS][cyclonedds-github].

This crate currently provides the `Topicable` derive macro used by
[`eclipse-cyclonedds`][eclipse-cyclonedds]. Most users should depend on
`eclipse-cyclonedds` and use its re-export:

```rust
#[derive(cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
struct Sensor {
    #[dds(key)]
    id: u32,
    temperature: f32,
}
```

Depend on `eclipse-cyclonedds-macros` directly only when you need the proc macro
crate itself.

- [Quick Start](#quick-start)
- [Topicable Derive](#topicable-derive)
- [Generated Implementation](#generated-implementation)

## Quick Start

For normal DDS applications, add the main binding:

```toml
[dependencies]
eclipse-cyclonedds = "0.0.3"
serde = { version = "1", features = ["derive"] }
```

Then derive `Topicable` on a named-field struct:

```rust
use cyclonedds::{Domain, Participant, Topic, Topicable};

#[derive(Topicable, serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
#[dds(type_name = "Sensor")]
struct Sensor {
    #[dds(key)]
    id: u32,
    temperature: f32,
}

fn main() -> cyclonedds::Result<()> {
    let domain = Domain::default();
    let participant = Participant::new(&domain)?;
    let topic = Topic::<Sensor>::new(&participant, "Sensor")?;

    Ok(())
}
```

See the [main crate documentation][eclipse-cyclonedds-docs] for actually
interacting with DDS.

## Topicable Derive

`#[derive(Topicable)]` implements `cyclonedds::Topicable` for a named-field
struct. The payload type must also satisfy the trait bounds required by
`Topicable`, including `serde::Serialize`, `serde::Deserialize`, `Clone`, and
`Debug`.

Supported attributes:

- `#[dds(key)]` on fields that make up the DDS instance key.
- `#[dds(type_name = "...")]` on the struct to override the DDS type name used
  for topic matching.

For now the derive macro rejects enums, unions, and tuple structs.

## Generated Implementation

For keyed topics, fields marked with `#[dds(key)]` are collected into a hidden
generated key type. That type implements the serialization and keyhash support
needed for keys in DDS. You can refer to the key type created by this macro
through the main crate's `cyclonedds::Key<T>` type alias:

```rust
use cyclonedds::{Key, Topicable};

#[derive(Topicable, serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
struct Position {
    #[dds(key)]
    x: i32,
    #[dds(key)]
    y: i32,
    label: String,
}

let position = Position {
    x: 1,
    y: 2,
    label: "origin".to_string(),
};

// Access the generated Key type via `Key<Position>`.
let key: Key<Position> = position.as_key();
assert_eq!(key, Key::<Position> { x: 1, y: 2 });
```

For unkeyed topics, the generated implementation uses `()` as the key type.
`from_key` returns `Default::default()`, so unkeyed derived types must implement
`Default`.

## Minimum Supported Rust Version

For now, the MSRV is the latest stable Rust version at the time of release.

## References

- [Rust binding for Eclipse Cyclone DDS][eclipse-cyclonedds]
- [Cyclone DDS][cyclonedds-github]
- [Cyclone DDS Documentation][cyclonedds-docs]
- [OMG DDS Specification][dds-spec]
- [OMG DDSI-RTPS specification][ddsi-rtps-spec]
- [OMG DDS Wiki][omg-dds-wiki]
- [Contributing](../CONTRIBUTING.md)
- [Security Policy](../SECURITY.md)

[check-workflow]: https://github.com/eclipse-cyclonedds/cyclonedds-rust/actions/workflows/check.yml
[check-workflow-status-shield]: https://shieldcn.dev/github/ci/eclipse-cyclonedds/cyclonedds-rust?no-track&mode=light&size=xs
[codecov]: https://codecov.io/github/eclipse-cyclonedds/cyclonedds-rust
[codecov-shield]: https://codecov.io/gh/eclipse-cyclonedds/cyclonedds-rust/graph/badge.svg
[community]: https://discord.gg/4QQvWZrFKF
[community-shield]: https://shieldcn.dev/discord/960814229844291604.svg?no-track&variant=branded&size=xs
[crates.io]: https://crates.io/crates/eclipse-cyclonedds-macros
[crates.io-shield]: https://shieldcn.dev/group/crates/eclipse-cyclonedds-macros+crates/license/eclipse-cyclonedds-macros.svg?no-track&mode=light&size=xs
[cyclonedds-docs]: https://cyclonedds.io/docs
[cyclonedds-github]: https://github.com/eclipse-cyclonedds/cyclonedds
[cyclonedds-homepage]: https://cyclonedds.io
[cyclonedds-homepage-shield]: https://shieldcn.dev/badge/web-cyclonedds.io-blue.svg?no-track&mode=light&logo=lu%3ATornado&size=xs
[dds-spec]: https://www.omg.org/spec/DDS/1.4/About-DDS/
[ddsi-rtps-spec]: https://www.omg.org/spec/DDSI-RTPS/
[deps.rs]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust
[deps.rs-shield]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust/status.svg
[docs.rs]: https://docs.rs/eclipse-cyclonedds-macros
[docs.rs-shield]: https://docs.rs/eclipse-cyclonedds-macros/badge.svg
[eclipse-cyclonedds]: https://crates.io/crates/eclipse-cyclonedds
[eclipse-cyclonedds-docs]: https://docs.rs/eclipse-cyclonedds
[omg-dds-wiki]: https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:01_front:4_toc
