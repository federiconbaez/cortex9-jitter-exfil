//! Evaluacion de VALOR del detector KS de canal de temporizacion (sistema 06).
//!
//! Metrica: TPR a un FPR fijo (alpha), frente a tres situaciones:
//!   - benigno vs linea base  -> mide la FPR real (deberia ≈ alpha).
//!   - encubierto 1ms/4ms vs linea base -> TPR contra el adversario ingenuo.
//!   - imitador (mimic) vs linea base   -> TPR contra un adversario que mantiene
//!     la distribucion marginal de la linea base.
//!
//! Linea base de comparacion conceptual: "sin detector" (no marca nada -> TPR=0,
//! FPR=0). El detector solo aporta valor si TPR >> FPR para el adversario dado.
//!
//! Corrida reproducible: semillas fijas, TRIALS pares por celda.

use fnbr_jitter::{detect, gen::*};

const TRIALS: usize = 1000; // pares por celda
const WIN: usize = 400; // tamano de ventana observada
const BASE_N: usize = 5000; // tamano de la linea base
const MEDIAN: f64 = 2.0e6; // 2 ms en ns
const SIGMA: f64 = 0.6;

/// Fraccion de ventanas marcadas sospechosas, para una fuente de ventanas dada.
fn rate<F>(seed: u64, alpha: f64, mut make_window: F) -> f64
where
    F: FnMut(&mut Rng) -> Vec<f64>,
{
    let mut rng = Rng::new(seed);
    let base = benign(&mut rng, BASE_N, MEDIAN, SIGMA);
    let hits = (0..TRIALS)
        .filter(|_| {
            let w = make_window(&mut rng);
            detect(&base, &w, alpha).suspicious
        })
        .count();
    hits as f64 / TRIALS as f64
}

fn main() {
    let alphas = [0.001, 0.01, 0.05];

    println!("# Evaluacion de valor — detector 06 (jitter/KS)");
    println!(
        "# TRIALS={TRIALS} pares/celda, WIN={WIN}, base_n={BASE_N}, mediana={}ms, sigma={SIGMA}",
        MEDIAN / 1e6
    );
    println!("# benigno: lognormal(mediana 2ms, sigma 0.6)");
    println!("# encubierto (ingenuo): bits 1ms/4ms + ruido sigma 0.15");
    println!("# imitador (mimic): bits por mitad de la CDF lognormal, marginal identico");
    println!();
    println!(
        "{:>8}  {:>10}  {:>12}  {:>12}",
        "alpha", "FPR", "TPR_naive", "TPR_mimic"
    );
    println!("{:->8}  {:->10}  {:->12}  {:->12}", "", "", "", "");

    for &a in &alphas {
        let fpr = rate(0xBE0_u64.wrapping_add((a * 1e6) as u64), a, |r| {
            benign(r, WIN, MEDIAN, SIGMA)
        });
        let tpr_naive = rate(0xC0_u64.wrapping_add((a * 1e6) as u64), a, |r| {
            covert(r, WIN, 1.0e6, 4.0e6, 0.15)
        });
        let tpr_mimic = rate(0x3113_u64.wrapping_add((a * 1e6) as u64), a, |r| {
            mimic(r, WIN, MEDIAN, SIGMA)
        });
        println!(
            "{:>8.3}  {:>10.4}  {:>12.4}  {:>12.4}",
            a, fpr, tpr_naive, tpr_mimic
        );
    }
}
