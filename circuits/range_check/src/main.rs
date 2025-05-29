use range_check::m31;
use std::fmt::Debug;
fn main() -> Result<(), Box<dyn Debug>> {
    let val = 1u32 << 31;
    m31::prove_and_verify_range_check(val);

    Ok(())
}
