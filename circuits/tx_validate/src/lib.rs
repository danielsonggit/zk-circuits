//! 交易格式验证电路：四列 [sender, receiver, amount, nonce]

use std::fmt::Debug;

use common_utils::{build_stark_config, pad_to_pow2};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_field::Field;
use p3_field::FieldAlgebra;
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use p3_uni_stark::{prove, verify};

/// TxValidateAir：每行 4 列，分别是 (sender, receiver, amount, nonce)
pub struct TxValidateAir {
    pub num_rows: usize,
}

impl<F: Field> BaseAir<F> for TxValidateAir {
    fn width(&self) -> usize {
        5
    }
}

impl<AB: AirBuilder> Air<AB> for TxValidateAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let curr = main.row_slice(0);

        // let sender = curr[0];
        // let receiver = curr[1];
        let amount = curr[2];
        // let nonce = curr[3];
        let inv = curr[4];

        // ── 首行 / 每行都要满足的约束 ──
        builder
            .when_transition()
            .assert_eq(amount * inv, AB::Expr::ONE);
    }
}

/// 构造交易 trace：一行一笔 tx
pub fn generate_tx_validate_trace_with_inv<F: Field>(
    txs: &[[u32; 4]],
) -> (RowMajorMatrix<F>, usize) {
    let mut data = Vec::with_capacity(txs.len() * 5);
    for tx in txs {
        let sender = F::from_canonical_u32(tx[0]);
        let receiver = F::from_canonical_u32(tx[1]);
        let amount = F::from_canonical_u32(tx[2]);
        let nonce = F::from_canonical_u32(tx[3]);
        // 计算逆元：如果 amount != 0，就 invert，否则取 0
        let inv = if tx[2] != 0 {
            // invert() 返回 Option<F>
            amount.inverse()
        } else {
            F::ZERO
        };

        // 推入这一行的 5 个值
        data.push(sender);
        data.push(receiver);
        data.push(amount);
        data.push(nonce);
        data.push(inv);
    }
    // pad 到 2ᵏ 行
    pad_to_pow2(data, 5)
}

/// 测试用 prove & verify 接口
pub fn prove_and_verify_tx_validate(txs: &[[u32; 4]]) -> Result<(), impl Debug> {
    let (config, hasher) = build_stark_config();
    type Challenger = SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>>;

    let (trace, height) = generate_tx_validate_trace_with_inv::<Mersenne31>(txs);
    let air = TxValidateAir { num_rows: height };

    let mut prov_ch = Challenger::from_hasher(vec![], hasher);
    let proof = prove(&config, &air, &mut prov_ch, trace.clone(), &vec![]);
    let mut ver_ch = Challenger::from_hasher(vec![], hasher);
    verify(&config, &air, &mut ver_ch, &proof, &vec![])
    // Ok(())
}

// 在 lib.rs 底部加上：
#[cfg(test)]
mod tests;
