const N: usize = 624;
const M: usize = 397;
const MATRIX_A: i32 = 0x9908b0dfu32 as i32;
const UPPER_MASK: i32 = 0x80000000u32 as i32;
const LOWER_MASK: i32 = 0x7fffffffi32;
const TEMPERING_MASK_B: i32 = 0x9d2c5680u32 as i32;
const TEMPERING_MASK_C: i32 = 0xefc60000u32 as i32;

pub struct LR2Random {
    mti: usize,
    mt: Vec<i32>,
    mtr: Vec<i32>,
}

impl LR2Random {
    pub fn new() -> Self {
        let mut r = LR2Random {
            mti: 0,
            mt: vec![0i32; N + 1],
            mtr: vec![0i32; N],
        };
        r.set_seed(4357);
        r
    }

    pub fn with_seed(seed: i32) -> Self {
        let mut r = LR2Random {
            mti: 0,
            mt: vec![0i32; N + 1],
            mtr: vec![0i32; N],
        };
        r.set_seed(seed);
        r
    }

    pub fn set_seed(&mut self, seed: i32) {
        let mut seed = seed;
        for i in 0..N {
            self.mt[i] = seed & (0xffff0000u32 as i32);
            seed = seed.wrapping_mul(69069).wrapping_add(1);
            self.mt[i] |= ((seed as u32 & 0xffff0000) >> 16) as i32;
            seed = seed.wrapping_mul(69069).wrapping_add(1);
        }
        self.generate_mt();
    }

    pub fn next_int(&mut self, max: i32) -> i32 {
        let rand_max = max as i64;
        let r = self.rand_mt() as u32 as u64;
        ((r * rand_max as u64) >> 32) as i32
    }

    fn generate_mt(&mut self) {
        let mag01: [i32; 2] = [0, MATRIX_A];
        let mut y: i32;

        for kk in 0..(N - M) {
            y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            self.mt[kk] = self.mt[kk + M] ^ ((y as u32 >> 1) as i32) ^ mag01[(y & 0x1) as usize];
        }

        self.mt[N] = self.mt[0];
        for kk in (N - M)..N {
            y = (self.mt[kk] & UPPER_MASK) | (self.mt[kk + 1] & LOWER_MASK);
            self.mt[kk] =
                self.mt[kk + M - N] ^ ((y as u32 >> 1) as i32) ^ mag01[(y & 0x1) as usize];
        }

        for kk in 0..N {
            y = self.mt[kk];
            y ^= (y as u32 >> 11) as i32;
            y ^= (y << 7) & TEMPERING_MASK_B;
            y ^= (y << 15) & TEMPERING_MASK_C;
            y ^= (y as u32 >> 18) as i32;
            self.mtr[kk] = y;
        }
        self.mti = 0;
    }

    pub fn rand_mt(&mut self) -> i32 {
        if self.mti >= N {
            self.generate_mt();
        }
        let result = self.mtr[self.mti];
        self.mti += 1;
        result
    }
}

impl Default for LR2Random {
    fn default() -> Self {
        Self::new()
    }
}
