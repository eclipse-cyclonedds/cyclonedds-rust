//! Internal implementation details exposed for advanced use-cases.
//!
//! This module exposes low-level APIs and internal data structures used to wrap
//! the [Cyclone DDS core library](http://github.com/eclipse-cyclonedds/cyclonedds).
//! It is intended **only** for power users who understand the internal workings
//! of the library and need access to functionality beyond the public API.
//!
//! This module is only available when the `internal` feature is enabled.
//!
//! ---
//! > ⚠️ **Warning:** Everything in this module is considered
//! > unstable and may change, break, or be removed without notice between
//! > versions.
//! >
//! > Do not rely on this API unless you are prepared to deal with breakage.
//! ---
//!
//! **Use at your own risk.**

pub mod traits;
