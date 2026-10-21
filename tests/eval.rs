use fnbr_jitter::{detect, gen::*, ks_statistic, log_histogram_entropy};

const ALPHA: f64 = 0.001;
const TRIALS: usize = 500;
const WIN: usize = 400;

#[test]
fn ks_identical_is_zero() {
    let a = vec![1.0, 2.0, 3.0];
    assert_eq!(ks_statistic(&a, &a), 0.0);
}

#[test]
fn ks_disjoint_is_one() {
    assert_eq!(ks_statistic(&[1.0, 2.0], &[10.0, 11.0]), 1.0);
}

/// Tasa de falsos positivos: ventanas benignas contra su propia línea base.
#[test]
fn false_positive_rate_below_1pct() {
    let mut rng = Rng::new(42);
    let base = benign(&mut rng, 5000, 2e6, 0.6);
    let fp = (0..TRIALS).filter(|_| detect(&base, &benign(&mut rng, WIN, 2e6, 0.6), ALPHA).suspicious).count();
    let rate = fp as f64 / TRIALS as f64;
    println!("FPR = {rate:.4} ({fp}/{TRIALS})");
    assert!(rate < 0.01, "FPR demasiado alta: {rate}");
}

/// Tasa de detección de un canal binario cuya media imita la línea base.
#[test]
fn detection_rate_above_95pct() {
    let mut rng = Rng::new(7);
    let base = benign(&mut rng, 5000, 2e6, 0.6);
    let tp = (0..TRIALS).filter(|_| detect(&base, &covert(&mut rng, WIN, 1e6, 4e6, 0.15), ALPHA).suspicious).count();
    let rate = tp as f64 / TRIALS as f64;
    println!("TPR = {rate:.4} ({tp}/{TRIALS})");
    assert!(rate > 0.95, "detección demasiado baja: {rate}");
}

#[test]
fn covert_channel_has_lower_entropy() {
    let mut rng = Rng::new(9);
    let b = log_histogram_entropy(&benign(&mut rng, 4000, 2e6, 0.6), 32);
    let c = log_histogram_entropy(&covert(&mut rng, 4000, 1e6, 4e6, 0.15), 32);
    println!("H benigno = {b:.3} bits, H encubierto = {c:.3} bits");
    assert!(c < b);
}

/// El adversario imitador mantiene la distribución marginal: el KS no lo distingue
/// del tráfico benigno. TPR_mimic debe quedar del orden de alpha (≈ FPR).
#[test]
fn mimic_adversary_evades_ks_at_alpha_001() {
    const ALPHA_M: f64 = 0.01;
    let mut rng = Rng::new(11);
    let base = benign(&mut rng, 5000, 2e6, 0.6);
    let tp = (0..TRIALS)
        .filter(|_| detect(&base, &mimic(&mut rng, WIN, 2e6, 0.6), ALPHA_M).suspicious)
        .count();
    let rate = tp as f64 / TRIALS as f64;
    println!("TPR_mimic (alpha={ALPHA_M}) = {rate:.4} ({tp}/{TRIALS})");
    // Con 500 ensayos y alpha=0.01, esperamos ~5 hits. Tolerancia ancha: < 5%.
    // Si el KS distinguiera de verdad, esto saltaría a ~1.0 como el covert.
    assert!(rate < 0.05, "el imitador no debería ser detectado: TPR={rate}");
}

#[test]
fn inv_normal_cdf_symmetry_and_known_points() {
    // Φ⁻¹(0.5) = 0; Φ⁻¹(0.975) ≈ 1.96; simetría.
    let z05 = inv_normal_cdf(0.5);
    assert!(z05.abs() < 1e-9, "Φ⁻¹(0.5) debería ser ~0, got {z05}");
    let z975 = inv_normal_cdf(0.975);
    assert!((z975 - 1.95996398454).abs() < 1e-6, "Φ⁻¹(0.975) ≈ 1.96, got {z975}");
    let z025 = inv_normal_cdf(0.025);
    assert!((z025 + z975).abs() < 1e-9, "simetría rota: {z025} vs -{z975}");
}
