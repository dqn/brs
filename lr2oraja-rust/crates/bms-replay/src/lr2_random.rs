// LR2Random — Mersenne Twister MT19937 variant.
//
// This is NOT java.util.Random (which uses LCG). It matches the MT19937
// implementation in Java `LR2Random.java` exactly, including the non-standard
// seeding algorithm.

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908b0df;
const UPPER_MASK: u32 = 0x80000000;
const LOWER_MASK: u32 = 0x7fffffff;
const TEMPERING_MASK_B: u32 = 0x9d2c5680;
const TEMPERING_MASK_C: u32 = 0xefc60000;

/// MT19937 variant used by LR2 for ghost data lane shuffle.
pub struct LR2Random {
    mt: [u32; N + 1],
    mtr: [u32; N],
    mti: usize,
}

impl LR2Random {
    pub fn new(seed: i32) -> Self {
        let mut rng = Self {
            mt: [0u32; N + 1],
            mtr: [0u32; N],
            mti: 0,
        };
        rng.set_seed(seed);
        rng
    }

    /// Initialize the generator with a seed.
    ///
    /// Uses a non-standard seeding algorithm: `69069 * seed + 1` (i32 wrapping).
    /// Each mt[i] is constructed from the upper 16 bits of two consecutive LCG outputs.
    pub fn set_seed(&mut self, seed: i32) {
        // Java uses int (i32) arithmetic with wrapping
        let mut s = seed;
        for i in 0..N {
            // mt[i] = seed & 0xffff0000
            self.mt[i] = (s as u32) & 0xffff0000;
            s = s.wrapping_mul(69069).wrapping_add(1);
            // mt[i] |= (seed & 0xffff0000) >>> 16
            self.mt[i] |= ((s as u32) & 0xffff0000) >> 16;
            s = s.wrapping_mul(69069).wrapping_add(1);
        }
        self.generate_mt();
    }

    fn generate_mt(&mut self) {
        let mag01: [u32; 2] = [0, MATRIX_A];

        // First loop: kk in 0..N-M
        for kk in 0..(N - M) {
            let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            self.mt[kk] = self.mt[kk + M] ^ (y >> 1) ^ mag01[(y & 0x1) as usize];
        }

        // mt[N] = mt[0] (wrap-around)
        self.mt[N] = self.mt[0];

        // Second loop: kk in N-M..N
        // Java: mt[kk + (M - N)] where M - N is negative → equivalent to mt[kk - (N - M)]
        for kk in (N - M)..N {
            let y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            self.mt[kk] = self.mt[kk - (N - M)] ^ (y >> 1) ^ mag01[(y & 0x1) as usize];
        }

        // Tempering
        for kk in 0..N {
            let mut y = self.mt[kk];
            y ^= y >> 11;
            y ^= (y << 7) & TEMPERING_MASK_B;
            y ^= (y << 15) & TEMPERING_MASK_C;
            y ^= y >> 18;
            self.mtr[kk] = y;
        }

        self.mti = 0;
    }

    /// Generate a raw 32-bit MT value.
    pub fn rand_mt(&mut self) -> u32 {
        if self.mti >= N {
            self.generate_mt();
        }
        let value = self.mtr[self.mti];
        self.mti += 1;
        value
    }

    /// Generate a random integer in [0, max).
    ///
    /// Uses the same scaling as Java: `(unsigned_rand * max) >>> 32`.
    pub fn next_int(&mut self, max: i32) -> i32 {
        let rand_val = self.rand_mt() as u64;
        let max_val = max as u64;
        ((rand_val * max_val) >> 32) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_seed() {
        // Default seed in Java is 4357
        let mut rng = LR2Random::new(4357);
        // Just verify it produces deterministic output
        let v1 = rng.rand_mt();
        let mut rng2 = LR2Random::new(4357);
        let v2 = rng2.rand_mt();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_next_int_range() {
        let mut rng = LR2Random::new(12345);
        for _ in 0..1000 {
            let val = rng.next_int(10);
            assert!(val >= 0 && val < 10);
        }
    }

    #[test]
    fn test_deterministic_sequence() {
        let mut rng = LR2Random::new(42);
        let seq: Vec<u32> = (0..10).map(|_| rng.rand_mt()).collect();
        // Verify same seed produces same sequence
        let mut rng2 = LR2Random::new(42);
        let seq2: Vec<u32> = (0..10).map(|_| rng2.rand_mt()).collect();
        assert_eq!(seq, seq2);
    }

    #[test]
    fn test_reseed() {
        let mut rng = LR2Random::new(100);
        let v1 = rng.rand_mt();
        rng.set_seed(100);
        let v2 = rng.rand_mt();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_exhausts_buffer_and_regenerates() {
        let mut rng = LR2Random::new(1);
        // Generate N+1 values to trigger regeneration
        for _ in 0..=N {
            rng.rand_mt();
        }
        // Should still work after regeneration
        let val = rng.next_int(100);
        assert!(val >= 0 && val < 100);
    }
}
