use super::prove_and_verify_range_check;

#[test]
fn test_range_check_success() {
    // 测试边界值：2^31 - 1，应该通过
    let val = (1u32 << 31) - 1; // 2^31 - 1 = 2,147,483,647
    assert!(
        prove_and_verify_range_check(val).is_ok(),
        "range check should pass for max valid value {}",
        val
    );
}

#[test]
fn test_range_check_small_values() {
    // 测试小值，应该通过
    let vals = [0u32, 1, 100, 1024];
    for val in vals {
        assert!(
            prove_and_verify_range_check(val).is_ok(),
            "range check should pass for value {}",
            val
        );
    }
}

#[test]
fn test_range_check_fail() {
    // 测试 2^31，超出我们电路允许的 [0, 2^31) 区间，应当失败
    let val = 1u32 << 31; // 2^31 = 2,147,483,648
    assert!(
        prove_and_verify_range_check(val).is_err(),
        "range check should fail for out-of-bounds value {} (2^31)",
        val
    );
}
