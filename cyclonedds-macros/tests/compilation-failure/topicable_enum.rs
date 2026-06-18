use cyclonedds_macros::Topicable;

#[derive(Topicable)]
enum Data {
    Variant1 { x: i32, y: i32 },
    Variant2(i32, u32),
}

fn main() {}
