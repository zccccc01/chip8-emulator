const DEFAULT_SEED: u16 = 888;
const LCG_A: u16 = 75;
const LCG_C: u16 = 74;

pub struct LinearCongruentialGenerator {
    pub seed: u16,
}

impl Default for LinearCongruentialGenerator {
    fn default() -> Self {
        Self { seed: DEFAULT_SEED }
    }
}

impl Iterator for LinearCongruentialGenerator {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        // X(n+1) = (A * Xn + C) mod 2^16
        // u16 类型溢出会自动完成 mod 65536
        self.seed = LCG_A.wrapping_mul(self.seed).wrapping_add(LCG_C);
        Some(self.seed)
    }
}
