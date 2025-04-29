//! 单元测试

use super::prove_and_verify_tx_validate;
// use anyhow::Result;
use std::fmt::Debug;

#[test]
fn test_tx_validate_simple() -> Result<(), impl Debug> {
    // 一笔简单 tx：sender=1, receiver=2, amount=10, nonce=0
    let txs = &[[1, 2, 10, 0]];
    prove_and_verify_tx_validate(txs)
}

#[test]
fn test_tx_validate_multiple() -> Result<(), impl Debug> {
    let txs = &[[0, 1, 5, 0], [1, 2, 3, 1], [2, 0, 1, 2]];
    prove_and_verify_tx_validate(txs)
}
#[test]
fn test_tx_validate_zero() -> Result<(), impl Debug> {
    let txs = &[[0, 1, 0, 0]];
    prove_and_verify_tx_validate(txs)
}
