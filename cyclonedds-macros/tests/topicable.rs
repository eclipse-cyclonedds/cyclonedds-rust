#[test]
fn test_topicable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation-failure/*.rs");
}
