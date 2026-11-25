use cyclonedds_macros::FixedSize;

// A packed layout is not what a C compiler produces for the equivalent IDL type,
// and it would make the alignment assertion vacuous.
#[derive(FixedSize, Clone, Copy)]
#[repr(C, packed)]
struct Data {
    x: u8,
    y: u64,
}

fn main() {}
