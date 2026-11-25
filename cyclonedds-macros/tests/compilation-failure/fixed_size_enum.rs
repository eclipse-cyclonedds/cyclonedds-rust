use cyclonedds_macros::FixedSize;

// Only listed discriminants are valid values, so bytes from another process
// cannot be reinterpreted as one.
#[derive(FixedSize, Clone, Copy)]
#[repr(C)]
enum Data {
    First,
    Second,
}

fn main() {}
