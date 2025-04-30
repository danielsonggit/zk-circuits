use std::fmt::Debug;

use common_utils::{build_stark_config, pad_to_pow2};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_field::{Field, FieldAlgebra};
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use p3_uni_stark::{prove, verify};

pub struct RangeCheckAir {
    pub num_rows: usize,
}

impl<F> BaseAir<F> for RangeCheckAir {
    fn width(&self) -> usize {
        1 + 31
    }
}
impl<AB: AirBuilder> Air<AB> for RangeCheckAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let curr = main.row_slice(0);
        let value = curr[0];

        let mut trans = builder.when_transition();
        // 1) 每个 bit_i 都是 0/1： bit_i * (bit_i - 1) == 0
        for i in 1..(1 + 31) {
            let bit = curr[i];
            trans.assert_eq(bit * (bit - AB::Expr::ONE), AB::Expr::ZERO);
        }
        // 重构：sum(bit_i * 2^i) == value  (i=0..30)
        let mut recomposed = AB::Expr::ZERO;
        for i in 0..31 {
            let bit = curr[1 + i];
            let weight = AB::Expr::from_canonical_u32(1u32 << i);
            recomposed = recomposed + bit * weight;
        }
        trans.assert_eq(recomposed, value);
    }
}

/// 生成 Trace 并 pad 到 2ᵏ 行
pub fn generate_range_check_trace<F: Field>(values: &[u32]) -> (RowMajorMatrix<F>, usize) {
    let mut data = Vec::with_capacity(values.len() * 32);

    for &v in values {
        // value
        data.push(F::from_canonical_u32(v));
        // bits
        for i in 0..31 {
            let bit = (v >> i) & 1;
            data.push(F::from_canonical_u32(bit));
        }
    }
    pad_to_pow2(data, 32)
}

pub fn prove_and_verify_range_check(values: &[u32]) -> Result<(), impl Debug> {
    let (config, hasher) = build_stark_config();
    type Challenger = SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>>;

    let (trace, height) = generate_range_check_trace::<Mersenne31>(values);
    let air = RangeCheckAir { num_rows: height };

    let mut prov_ch = Challenger::from_hasher(vec![], hasher);
    let proof = prove(&config, &air, &mut prov_ch, trace.clone(), &vec![]);
    let mut ver_ch = Challenger::from_hasher(vec![], hasher);
    verify(&config, &air, &mut ver_ch, &proof, &vec![])
}

#[cfg(test)]
mod tests;
