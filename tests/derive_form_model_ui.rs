#[test]
fn form_model_derive_ui() {
    let testcases = trybuild::TestCases::new();
    testcases.pass("tests/ui/form_model/pass.rs");
    testcases.compile_fail("tests/ui/form_model/fail_generic.rs");
    testcases.compile_fail("tests/ui/form_model/fail_tuple.rs");
    testcases.compile_fail("tests/ui/form_model/fail_enum.rs");
}
