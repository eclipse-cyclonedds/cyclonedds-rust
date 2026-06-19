# Raw FFI Bindings for Eclipse Cyclone DDS

[![Latest Version][crates.io-shield]][crates.io]
[![Build Status][check-workflow-status-shield]][check-workflow]
[![Community][community-shield]][community]
[![Website][cyclonedds-homepage-shield]][cyclonedds-homepage]

[![Code Coverage][codecov-shield]][codecov]
[![Documentation][docs.rs-shield]][docs.rs]
[![Dependency Status][deps.rs-shield]][deps.rs]

Raw Rust FFI bindings for [Eclipse Cyclone DDS][cyclonedds-github].

This crate is the low-level `bindgen` layer used by
[`eclipse-cyclonedds`][eclipse-cyclonedds]. Most applications should depend on
`eclipse-cyclonedds`, which provides the safe Rust API, topic support, QoS
builders, readers, writers, waitsets, and sample handling.

Depend on `eclipse-cyclonedds-sys` directly only when you need access to the C
API surface itself.

- [Quick Start](#quick-start)
- [Linking Cyclone DDS](#linking-cyclone-dds)
- [Generated Bindings](#generated-bindings)

## Quick Start

For normal DDS applications, add the main binding:

```toml
[dependencies]
eclipse-cyclonedds = "0.0.4"
```

Use this crate directly for raw FFI integration:

```toml
[dependencies]
eclipse-cyclonedds-sys = "0.0.4"
```

The crate exposes the generated bindings under the Rust library name
`cyclonedds_sys`:

```rust
use cyclonedds_sys as sys;

fn main() {
    let domain = sys::DOMAIN_DEFAULT;
    println!("default DDS domain: {domain}");
}
```

Direct use of these bindings follows the safety contract of the Cyclone DDS C
API. Prefer the safe `eclipse-cyclonedds` crate unless you specifically need raw
FFI.

## Linking Cyclone DDS

By default, this crate expects the Cyclone DDS C library and headers to be
available on the system. The build script links against `ddsc` and generates
bindings from `wrapper.h`.

If Cyclone DDS is installed outside the system search paths, set
`CYCLONEDDS_HOME` to the installation prefix:

```sh
CYCLONEDDS_HOME=/opt/cyclonedds cargo build
```

The build script will then look for headers under `$CYCLONEDDS_HOME/include` and
libraries under `$CYCLONEDDS_HOME/lib` or `$CYCLONEDDS_HOME/lib64`.

To build Cyclone DDS from the sources bundled with this crate, enable the
`vendored` feature:

```toml
[dependencies]
eclipse-cyclonedds-sys = { version = "0.0.4", features = ["vendored"] }
```

It requires `cmake` and `libclang` to be available.

## Generated Bindings

Bindings are generated at build time with `bindgen` and included from Cargo's
`OUT_DIR`. It contains raw pointers, C layout types, platform-specific
definitions, and functions that are unsafe to call without satisfying the C API
preconditions.

See the [main crate documentation][eclipse-cyclonedds-docs] for the safe
ergonomic Rust API.

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
[crates.io]: https://crates.io/crates/eclipse-cyclonedds-sys
[crates.io-shield]: https://shieldcn.dev/group/crates/eclipse-cyclonedds-sys+crates/license/eclipse-cyclonedds-sys.svg?no-track&mode=light&size=xs
[cyclonedds-docs]: https://cyclonedds.io/docs
[cyclonedds-github]: https://github.com/eclipse-cyclonedds/cyclonedds
[cyclonedds-homepage]: https://cyclonedds.io
[cyclonedds-homepage-shield]: https://shieldcn.dev/badge/web-cyclonedds.io-blue.svg?no-track&mode=light&logo=lu%3ATornado&size=xs
[dds-spec]: https://www.omg.org/spec/DDS/1.4/About-DDS/
[ddsi-rtps-spec]: https://www.omg.org/spec/DDSI-RTPS/
[deps.rs]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust
[deps.rs-shield]: https://deps.rs/repo/github/eclipse-cyclonedds/cyclonedds-rust/status.svg
[docs.rs]: https://docs.rs/eclipse-cyclonedds-sys
[docs.rs-shield]: https://docs.rs/eclipse-cyclonedds-sys/badge.svg
[eclipse-cyclonedds]: https://crates.io/crates/eclipse-cyclonedds
[eclipse-cyclonedds-docs]: https://docs.rs/eclipse-cyclonedds
[omg-dds-wiki]: https://www.omgwiki.org/ddsf/doku.php?id=ddsf:public:guidebook:01_front:4_toc
