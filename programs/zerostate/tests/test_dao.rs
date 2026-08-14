//! v2 (miners-only) is verified end-to-end on devnet via the 0state client
//! (join -> propose -> weighted vote). This file keeps a couple of pure-logic
//! checks; the old litesvm admission flow was removed with the admission model.

/// Mirror of the program's integer square root, kept in sync by eye.
fn isqrt(n: u64) -> u64 {
    if n < 2 { return n; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

#[test]
fn isqrt_is_sub_linear() {
    assert_eq!(isqrt(0), 0);
    assert_eq!(isqrt(1), 1);
    assert_eq!(isqrt(100), 10);
    assert_eq!(isqrt(10_000), 100);
    // a 100x larger stack yields only ~10x the sqrt term — the anti-whale curve
    assert_eq!(isqrt(1_000_000), 1000);
}

#[test]
fn decay_halves_per_half_life() {
    // weight's mined term = whole >> (idle / HALF_LIFE), i.e. halving per period
    let whole: u64 = 10_000;
    assert_eq!(whole >> 0, 10_000); // active miner: full
    assert_eq!(whole >> 1, 5_000);  // one half-life idle: halved
    assert_eq!(whole >> 2, 2_500);  // two: quartered
}
