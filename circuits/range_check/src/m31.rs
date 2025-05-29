// 小于2^31
use common_utils::{build_stark_config, pad_to_pow2};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_field::{Field, FieldAlgebra, PrimeField32};
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use p3_uni_stark::{prove, verify};
use tracing_forest::util::LevelFilter;
use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

pub struct RangeCheckAir {
    pub value: u32,
}

impl<F> BaseAir<F> for RangeCheckAir {
    fn width(&self) -> usize {
        32
    }
}
impl<AB: AirBuilder> Air<AB> for RangeCheckAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let current_row = main.row_slice(0);
        let next_row = main.row_slice(1);

        // Assert that the most significant bit is zero, only checked when its first row
        builder
            .when_first_row()
            .assert_eq(current_row[0], AB::Expr::ZERO);

        // initializing the `reconstructed_value` and the `next_row_sum`
        let mut reconstructed_value = AB::Expr::ZERO;
        let mut next_row_rowsum = AB::Expr::ZERO;
        for i in 0..32 {
            let bit = current_row[i];
            builder.assert_bool(bit);
            reconstructed_value += AB::Expr::from_wrapped_u32(1 << (31 - i)) * bit; // using `from_wrapped_u32` to make sure the value is in range of 31 bits.
            next_row_rowsum += next_row[i].into(); // converting the input to Expr and adding it to the sum.
        }

        // Assert if the reconstructed value matches the original value, only checked when its first row
        builder
            .when_first_row()
            .assert_eq(AB::Expr::from_wrapped_u32(self.value), reconstructed_value);
        // Assert if the sum of each remaining row is zero in every transition.
        builder
            .when_transition()
            .assert_eq(next_row_rowsum, AB::Expr::ZERO);
    }
}

/// 生成 Trace 并 pad 到 2ᵏ 行
pub fn generate_range_check_trace<F: Field>(value: u32) -> (RowMajorMatrix<F>, usize) {
    let mut data = Vec::with_capacity(32);

    for i in (0..32).rev() {
        if (value & (1 << i)) != 0 {
            data.push(F::ONE);
        } else {
            data.push(F::ZERO);
        }
    }

    pad_to_pow2(data, 32)
}

pub fn prove_and_verify_range_check(value: u32) {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(env_filter)
        .with(ForestLayer::default())
        .init();
    let (config, hasher) = build_stark_config();
    type Challenger = SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>>;

    let (trace, _) = generate_range_check_trace::<Mersenne31>(value);
    #[cfg(debug_assertions)]
    {
        println!("value: {}, Trace 矩阵内容 (Mersenne31 域):", value);
        println!("矩阵维度: {} x {}", trace.height(), trace.width());
        for i in 0..trace.height() {
            let row: Vec<Mersenne31> = trace.row(i).collect();
            for j in 0..32 {
                print!(" {} ", row[j].as_canonical_u32());
            }
            println!();
        }
    }

    let air = RangeCheckAir { value };

    let mut prov_ch = Challenger::from_hasher(vec![], hasher);
    let proof = prove(&config, &air, &mut prov_ch, trace.clone(), &vec![]);
    let mut ver_ch = Challenger::from_hasher(vec![], hasher);

    verify(&config, &air, &mut ver_ch, &proof, &vec![]).expect("range-check verification failed");
}
