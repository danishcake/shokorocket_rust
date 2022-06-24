#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fails_if_name_empty.rs");
    t.compile_fail("tests/fails_if_author_empty.rs");
    t.pass("tests/succeeds_for_plain_map.rs");
    t.pass("tests/succeeds_for_e1m1.rs");
}
