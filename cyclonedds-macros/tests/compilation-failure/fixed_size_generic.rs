use cyclonedds_macros::FixedSize;

// A generic type has no single layout, so there is nothing to promise.
#[derive(FixedSize, Clone, Copy)]
#[repr(C)]
struct Data<T> {
    value: T,
}

fn main() {}
