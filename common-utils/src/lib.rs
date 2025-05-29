use p3_challenger::{HashChallenger, SerializingChallenger32};
use p3_circle::CirclePcs;
use p3_commit::ExtensionMmcs;
use p3_field::extension::BinomialExtensionField;
use p3_field::Field;
use p3_fri::FriConfig;
use p3_keccak::Keccak256Hash;
use p3_matrix::dense::RowMajorMatrix;
use p3_merkle_tree::MerkleTreeMmcs;
use p3_mersenne_31::Mersenne31;
use p3_symmetric::{CompressionFunctionFromHasher, SerializingHasher32};
use p3_uni_stark::StarkConfig;
use std::marker::PhantomData;

/// Pads `data` (flat row-major) to the next power-of-two number of rows.
/// The matrix has width `width`, so the number of rows = data.len() / width.
/// Returns the padded matrix and the new row count.
pub fn pad_to_pow2<F: Field>(mut data: Vec<F>, width: usize) -> (RowMajorMatrix<F>, usize) {
    let height = data.len() / width;
    let mut padded = height.next_power_of_two();
    if padded < 4 {
        padded = 4;
    }
    // Extend with zeros until padded length
    let padding_rows = padded - height;
    let padding_elements = padding_rows * width;
    data.resize(data.len() + padding_elements, F::ZERO);

    (RowMajorMatrix::new(data, width), padded)
}

/// Builds a default StarkConfig<Pcs, Challenge, Challenger> paired with a byte hasher.
/// Base field = Mersenne31; extension = BinomialExtensionField<_,3>
pub fn build_stark_config() -> (
    StarkConfig<
        CirclePcs<
            Mersenne31,
            MerkleTreeMmcs<
                Mersenne31,
                u8,
                SerializingHasher32<Keccak256Hash>,
                CompressionFunctionFromHasher<Keccak256Hash, 2, 32>,
                32,
            >,
            ExtensionMmcs<
                Mersenne31,
                BinomialExtensionField<Mersenne31, 3>,
                MerkleTreeMmcs<
                    Mersenne31,
                    u8,
                    SerializingHasher32<Keccak256Hash>,
                    CompressionFunctionFromHasher<Keccak256Hash, 2, 32>,
                    32,
                >,
            >,
        >,
        BinomialExtensionField<Mersenne31, 3>,
        SerializingChallenger32<Mersenne31, HashChallenger<u8, Keccak256Hash, 32>>,
    >,
    Keccak256Hash,
) {
    type Val = Mersenne31;
    type Challenge = BinomialExtensionField<Val, 3>;
    type ByteHash = Keccak256Hash;
    type FieldHash = SerializingHasher32<ByteHash>;
    type MyCompress = CompressionFunctionFromHasher<ByteHash, 2, 32>;
    type ValMmcs = MerkleTreeMmcs<Val, u8, FieldHash, MyCompress, 32>;

    // Instantiate hashers and MMCS
    let byte_hasher = ByteHash {};
    let field_hasher = FieldHash::new(byte_hasher);
    let compress = MyCompress::new(byte_hasher);
    let val_mmcs = ValMmcs::new(field_hasher, compress);
    let ext_mmcs = ExtensionMmcs::<Val, Challenge, ValMmcs>::new(val_mmcs.clone());

    // FRI configuration
    let fri_config = FriConfig {
        log_blowup: 1,
        num_queries: 32,
        proof_of_work_bits: 16,
        mmcs: ext_mmcs.clone(),
    };

    // PCS instantiate
    let pcs = CirclePcs::<Val, ValMmcs, ExtensionMmcs<Val, Challenge, ValMmcs>> {
        mmcs: val_mmcs.clone(),
        fri_config,
        _phantom: PhantomData,
    };

    // StarkConfig
    let config = StarkConfig::<
        _, // Pcs
        Challenge,
        SerializingChallenger32<Val, HashChallenger<u8, ByteHash, 32>>,
    >::new(pcs);

    (config, byte_hasher)
}
