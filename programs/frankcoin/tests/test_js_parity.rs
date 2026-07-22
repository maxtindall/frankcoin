// Cross-check: the browser miner and the on-chain program must compute the
// SAME hash and the same leading-zero-bit count. If they ever diverge, the
// browser produces proofs the program rejects - a silent, baffling failure.
// Vector below was produced by the JS grinder in mint-src.
use solana_keccak_hasher::hashv;

fn leading_zero_bits(h: &[u8; 32]) -> u32 {
    let mut c = 0u32;
    for &b in h.iter() {
        if b == 0 { c += 8; } else { c += b.leading_zeros(); break; }
    }
    c
}

#[test]
fn js_and_rust_agree_on_the_same_proof() {
    let challenge = [3u8; 32];
    let miner = [7u8; 32];
    let nonce: u64 = 138922;          // found by the JS grinder at difficulty 16

    let h = hashv(&[&challenge, &miner, &nonce.to_le_bytes()]).to_bytes();
    let bits = leading_zero_bits(&h);

    println!("rust bits = {}", bits);
    assert!(bits >= 16, "rust disagrees with the browser miner: got {} bits", bits);
}
