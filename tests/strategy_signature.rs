#[test]
fn strategy_rejects_incorrect_signatures() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/strategy_wrong_signature.rs");
}
