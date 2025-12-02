//! Common definitions used in examples.

// NOTE: this needs to be a `mod.rs` file to not show up as a top-level example and still be scoped
// to just the `examples` dir.
#![allow(clippy::mod_module_files)]

/// A exemplar topic type representing a sample sensor.
#[derive(cyclonedds::Topicable, serde::Serialize, serde::Deserialize, Default, Clone)]
#[dds(type_name = "common::Sensor")]
pub struct Sensor {
    /// Unique key identifying this sample instance.
    #[dds(key)]
    pub id: u32,
    /// Human-readable message.
    pub message: String,
    /// Sample payload.
    pub data: [f32; 32],
}

impl std::fmt::Debug for Sensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sensor")
            .field("id", &self.id)
            .field("message", &self.message)
            .field(
                "data",
                &format_args!(
                    "[{:?}, {:?}, {:?}, {:?}, ..., {:?}, {:?}, {:?}, {:?}]",
                    self.data[0],
                    self.data[1],
                    self.data[2],
                    self.data[3],
                    self.data[28],
                    self.data[29],
                    self.data[30],
                    self.data[31]
                ),
            )
            .finish()
    }
}
