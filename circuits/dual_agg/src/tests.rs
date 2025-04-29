//! 单元测试：调用 prove_and_verify_dual_agg，不允许 panic

// use anyhow::Result;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_dual_agg_small() -> Result<()> {
//         // 简单场景：txs = [1,2,3]
//         // sum = 6, sumsq = 1+4+9 = 14
//         prove_and_verify_dual_agg(&[1, 2, 3])
//     }

//     #[test]
//     fn test_dual_agg_empty() -> Result<()> {
//         // edge case：无交易
//         // sum = 0, sumsq = 0
//         prove_and_verify_dual_agg(&[])
//     }

//     #[test]
//     fn test_dual_agg_longer() -> Result<()> {
//         let txs: Vec<u32> = (0..16).collect();
//         prove_and_verify_dual_agg(&txs)
//     }
// }

// circuits/dual_agg/src/tests.rs

use super::*;

#[test]
fn test_dual_agg_small() -> Result<(), impl Debug> {
    // txs = [1,2,3] → sum=6, sumsq=14
    prove_and_verify_dual_agg(&[1, 2, 3])
}

#[test]
fn test_dual_agg_empty() -> Result<(), impl Debug> {
    prove_and_verify_dual_agg(&[])
}

#[test]
fn test_dual_agg_longer() -> Result<(), impl Debug> {
    let txs: Vec<u32> = (0..16).collect();
    prove_and_verify_dual_agg(&txs)
}
