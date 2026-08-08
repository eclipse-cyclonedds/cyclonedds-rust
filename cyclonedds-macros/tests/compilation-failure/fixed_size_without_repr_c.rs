use cyclonedds_macros::FixedSize;

// A `repr(Rust)` type has no guaranteed layout, so its bytes mean nothing to
// another process.
#[derive(FixedSize, Clone, Copy)]
struct Data {
    x: u32,
    y: u32,
}

fn main() {}
