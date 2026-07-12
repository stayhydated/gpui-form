#[test]
fn select_item_accepts_no_display_and_generic_enums() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/select_item_no_display.rs");
    tests.pass("tests/ui/select_item_generic.rs");
}
