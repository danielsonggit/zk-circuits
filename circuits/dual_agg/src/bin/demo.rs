use dual_agg::{generate_dual_agg_trace, prove_and_verify_dual_agg};
use p3_field::PrimeField32;
use p3_matrix::Matrix;
use p3_mersenne_31::Mersenne31;
use std::fmt::Debug;

use tracing_forest::ForestLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
fn main() -> Result<(), impl Debug> {
    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(ForestLayer::default())
        .init();

    let txs: Vec<u32> = (0..16).collect();
    // 先生成并打印 trace
    let (trace, height) = generate_dual_agg_trace::<Mersenne31>(&txs);
    println!("Trace (rows = {}, width = {}):", height, trace.width());
    for i in 0..trace.height() {
        let row: Vec<Mersenne31> = trace.row(i).collect();
        let acc = row[0].as_canonical_u32();
        let acc2 = row[1].as_canonical_u32();
        let tx = row[2].as_canonical_u32();
        println!("第{}行: ({}, {}, {})", i, acc, acc2, tx);
    }
    prove_and_verify_dual_agg(&txs)
}
