//! 双累加器电路：同时跟踪 ∑tx 和 ∑(tx²)
use std::fmt::Debug;

use p3_air::{Air, AirBuilder, BaseAir};
use p3_challenger::HashChallenger;
use p3_challenger::SerializingChallenger32;
use p3_field::Field;
use p3_field::FieldAlgebra;
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use p3_uni_stark::{prove, verify};

use common_utils::{build_stark_config, pad_to_pow2};

/// AIR 定义
pub struct DualAggAir {
    pub num_rows: usize,
    pub expected_sum: u32,
    pub expected_sumsq: u32,
}

impl<F: Field> BaseAir<F> for DualAggAir {
    fn width(&self) -> usize {
        3
    }
}

impl<AB: AirBuilder> Air<AB> for DualAggAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let curr = main.row_slice(0);
        let next = main.row_slice(1);
        let tx = curr[2];

        // 首行：acc1=0, acc2=0
        builder.when_first_row().assert_eq(curr[0], AB::Expr::ZERO);
        builder.when_first_row().assert_eq(curr[1], AB::Expr::ZERO);

        // 转换：sum 更新 & sumsq 更新
        builder.when_transition().assert_eq(next[0], curr[0] + tx);
        builder
            .when_transition()
            .assert_eq(next[1], curr[1] + tx * tx);

        // 最后一行断言
        let sum = AB::Expr::from_canonical_u32(self.expected_sum);
        let sumsq = AB::Expr::from_canonical_u32(self.expected_sumsq);
        builder.when_last_row().assert_eq(curr[0], sum);
        builder.when_last_row().assert_eq(curr[1], sumsq);
    }
}

/// 生成 trace 并 pad 到 2ᵏ 行
pub fn generate_dual_agg_trace<F: Field>(txs: &[u32]) -> (RowMajorMatrix<F>, usize) {
    let mut data = Vec::with_capacity((txs.len() + 1) * 3);
    let mut acc1 = F::ZERO;
    let mut acc2 = F::ZERO;

    // 真正的交易行
    for &amt in txs {
        let famt = F::from_canonical_u32(amt);
        data.push(acc1);
        data.push(acc2);
        data.push(famt);
        acc1 += famt;
        acc2 += famt * famt;
    }
    // 最后一行 (tx = 0) 用于断言
    data.push(acc1);
    data.push(acc2);
    data.push(F::ZERO);

    // pad
    pad_to_pow2(data, 3)
}

/// 对外暴露的 prove & verify 接口，方便 tests 调用
pub fn prove_and_verify_dual_agg(txs: &[u32]) -> Result<(), impl Debug> {
    // 1) 构建 StarkConfig & hasher
    let (config, hasher) = build_stark_config();
    type Challenger = SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>>;

    // 2) 生成 trace
    let (trace, height) = generate_dual_agg_trace::<Mersenne31>(txs);

    // 3) 构造 AIR
    let sum: u32 = txs.iter().sum();
    let sumsq: u32 = txs.iter().map(|&x| x * x).sum();
    let air = DualAggAir {
        num_rows: height,
        expected_sum: sum,
        expected_sumsq: sumsq,
    };

    // 4) Prove & Verify
    let mut prover_ch = Challenger::from_hasher(vec![], hasher);
    let proof = prove(&config, &air, &mut prover_ch, trace.clone(), &vec![]);
    let mut verifier_ch = Challenger::from_hasher(vec![], hasher);
    verify(&config, &air, &mut verifier_ch, &proof, &vec![])
    // Ok(())
}

#[cfg(test)]
mod tests;
