//! Tests that verify RandomizerBase uses JavaRandom (LCG) instead of StdRng (ChaCha20).
//!
//! These tests enforce replay/pattern reproducibility by asserting that
//! set_random_seed() produces the exact same sequence as java.util.Random.

use beatoraja_pattern::java_random::JavaRandom;
use beatoraja_pattern::randomizer::RandomizerBase;

/// After set_random_seed(42), the internal JavaRandom must produce
/// the same sequence as `new java.util.Random(42)`.
/// Java verification: new Random(42).nextInt(10) -> 0, 3, 8, 4, 0
#[test]
fn randomizer_base_uses_java_random_sequence() {
    let mut base = RandomizerBase::new();
    base.set_random_seed(42);

    // The random field must be JavaRandom, and calling next_int_bounded
    // must match the Java LCG sequence.
    assert_eq!(base.random.next_int_bounded(10), 0);
    assert_eq!(base.random.next_int_bounded(10), 3);
    assert_eq!(base.random.next_int_bounded(10), 8);
    assert_eq!(base.random.next_int_bounded(10), 4);
    assert_eq!(base.random.next_int_bounded(10), 0);
}

/// Two RandomizerBase instances seeded with the same value must produce
/// identical sequences (determinism).
#[test]
fn randomizer_base_same_seed_same_sequence() {
    let mut base1 = RandomizerBase::new();
    let mut base2 = RandomizerBase::new();
    base1.set_random_seed(123);
    base2.set_random_seed(123);

    for _ in 0..20 {
        assert_eq!(
            base1.random.next_int_bounded(100),
            base2.random.next_int_bounded(100),
        );
    }
}

/// JavaRandom must have next_double() (port of java.util.Random.nextDouble).
/// Java verification: new Random(0).nextDouble() == 0.730967787376657
#[test]
fn java_random_next_double_exists_and_matches_java() {
    let mut rng = JavaRandom::new(0);
    let val = rng.next_double();
    // Java: new Random(0).nextDouble() = 0.730967787376657
    let expected = 0.730967787376657f64;
    assert!(
        (val - expected).abs() < 1e-15,
        "next_double() mismatch: got {}, expected {}",
        val,
        expected
    );
}

/// Verify next_double() sequence for seed 42.
/// Java verification:
///   Random r = new Random(42);
///   r.nextDouble() -> 0.7275636800328681
///   r.nextDouble() -> 0.6832234717598454
#[test]
fn java_random_next_double_sequence() {
    let mut rng = JavaRandom::new(42);
    let v1 = rng.next_double();
    let v2 = rng.next_double();

    assert!(
        (v1 - 0.7275636800328681f64).abs() < 1e-15,
        "1st next_double() mismatch: got {}",
        v1
    );
    assert!(
        (v2 - 0.6832234717598454f64).abs() < 1e-15,
        "2nd next_double() mismatch: got {}",
        v2
    );
}

/// set_random_seed with negative value should be ignored (no-op).
#[test]
fn randomizer_base_negative_seed_ignored() {
    let mut base = RandomizerBase::new();
    base.set_random_seed(42);
    // Consume one value to advance the state
    let _v1 = base.random.next_int_bounded(100);

    // Negative seed should not reset the RNG
    base.set_random_seed(-1);

    // The next value should continue from where seed=42 left off
    // (not reset to any other state)
    let mut reference = JavaRandom::new(42);
    let _ref_v1 = reference.next_int_bounded(100); // skip first
    let ref_v2 = reference.next_int_bounded(100);

    assert_eq!(base.random.next_int_bounded(100), ref_v2);
}
