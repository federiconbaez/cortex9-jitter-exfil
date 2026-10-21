//! Generadores sintéticos reproducibles (sin dependencias) para evaluar el detector.

/// xorshift64* determinista.
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed.max(1)) }
    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12; self.0 ^= self.0 << 25; self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniforme en (0,1).
    pub fn unif(&mut self) -> f64 { ((self.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64 }
    pub fn normal(&mut self) -> f64 {
        let (u, v) = (self.unif(), self.unif());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

/// Tráfico legítimo: intervalos log-normales (mediana ~ `median_ns`).
pub fn benign(rng: &mut Rng, n: usize, median_ns: f64, sigma: f64) -> Vec<f64> {
    (0..n).map(|_| median_ns * (sigma * rng.normal()).exp()).collect()
}

/// Canal encubierto: cada bit se codifica como un retardo corto o largo,
/// más ruido de red log-normal.
pub fn covert(rng: &mut Rng, n: usize, short_ns: f64, long_ns: f64, noise_sigma: f64) -> Vec<f64> {
    (0..n).map(|_| {
        let base = if rng.next_u64() & 1 == 0 { short_ns } else { long_ns };
        base * (noise_sigma * rng.normal()).exp()
    }).collect()
}
