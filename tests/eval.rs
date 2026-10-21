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
