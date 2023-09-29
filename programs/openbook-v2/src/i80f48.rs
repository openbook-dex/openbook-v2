// regression test for https://gitlab.com/tspiteri/fixed/-/issues/57
// see https://github.com/blockworks-foundation/fixed/issues/1
#[test]
fn bug_fixed_comparison_u64() {
    use fixed::types::I80F48;

    let a: u64 = 66000;
    let b: u64 = 1000;
    assert!(I80F48::from(a) > b); // fails!
}
