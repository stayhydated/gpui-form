#[test]
fn gpui_form_reports_invalid_component_shape_arguments() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/component_shape_*.rs");
}

#[test]
fn gpui_form_compiles_koruma_builder_attrs_end_to_end() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/koruma_builder_attrs_pass.rs");
}

#[test]
fn component_shape_derive_accepts_constructor_expressions() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/derive_component_shape_constructor_expr_pass.rs");
}
