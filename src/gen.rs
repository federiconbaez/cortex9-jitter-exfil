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

/// Cuantil de la normal estándar (Φ⁻¹), aproximación racional de Acklam.
/// Error absoluto < 1.15e-9 en toda la recta. Se usa para muestrear por CDF.
/// Los coeficientes publicados tienen más dígitos que un f64; se conservan literales.
#[allow(clippy::excessive_precision)]
pub fn inv_normal_cdf(p: f64) -> f64 {
    const A: [f64; 6] = [-3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02,
                          1.383577518672690e+02, -3.066479806614716e+01, 2.506628277459239e+00];
    const B: [f64; 5] = [-5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02,
                          6.680131188771972e+01, -1.328068155288572e+01];
    const C: [f64; 6] = [-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00,
                          -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00];
    const D: [f64; 4] = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00,
                          3.754408661907416e+00];
    let plow = 0.02425;
    let phigh = 1.0 - plow;
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= phigh {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

/// Adversario que IMITA la distribución de la línea base.
///
/// Codifica cada bit eligiendo de qué mitad de la CDF log-normal muestrea el
/// intervalo: bit 0 → mitad izquierda (u ∈ (0, 0.5)), bit 1 → mitad derecha
/// (u ∈ (0.5, 1)). Con bits equilibrados, la mezcla de ambas mitades reconstruye
/// exactamente la log-normal benigna: la distribución marginal es idéntica.
///
/// Por diseño, una prueba de dos muestras sobre la marginal (KS) no puede
/// distinguirlo del tráfico legítimo, aunque transmita bits. Es el caso límite
/// del detector, no un error de medición.
pub fn mimic(rng: &mut Rng, n: usize, median_ns: f64, sigma: f64) -> Vec<f64> {
    (0..n)
        .map(|_| {
            let bit = rng.next_u64() & 1;
            let u = if bit == 0 { 0.5 * rng.unif() } else { 0.5 + 0.5 * rng.unif() };
            let z = inv_normal_cdf(u);
            median_ns * (sigma * z).exp()
        })
        .collect()
}
