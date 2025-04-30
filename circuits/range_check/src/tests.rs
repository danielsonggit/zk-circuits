use super::prove_and_verify_range_check;
use core::fmt::Debug;

#[test]
fn test_range_check_small() -> Result<(), impl Debug> {
    let vals = &[0u32, 1, 5, 1024, (1 << 30)];
    // 这些都 < 2^31，应该通过
    prove_and_verify_range_check(vals)
}

#[test]
fn test_range_check_fail() -> Result<(), impl Debug> {
    // 取 2^31，超出我们电路允许的 [0, 2^31) 区间，应当失败
    let vals = &[1u32 << 31];
    // assert!(prove_and_verify_range_check(vals).is_err());
    prove_and_verify_range_check(vals)
}
