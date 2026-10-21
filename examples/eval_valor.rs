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

/// Corre las tres columnas para una semilla maestra y un alpha.
fn cell(master: u64, a: f64) -> [f64; 3] {
    let off = (a * 1e6) as u64;
    let s = |col: u64| master.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (col << 32) ^ off;
    [
        rate(s(0xBE0), a, |r| benign(r, WIN, MEDIAN, SIGMA)),
        rate(s(0xC0), a, |r| covert(r, WIN, 1.0e6, 4.0e6, 0.15)),
        rate(s(0x3113), a, |r| mimic(r, WIN, MEDIAN, SIGMA)),
    ]
}

/// Uso: `eval_valor [semilla ...]` (o env `SEMILLAS="1 2 3"`). Por defecto 1..=5.
fn main() {
    let alphas = [0.001, 0.01, 0.05];
    let mut seeds: Vec<u64> = std::env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    if seeds.is_empty() {
        if let Ok(v) = std::env::var("SEMILLAS") {
            seeds = v.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        }
    }
    if seeds.is_empty() {
        seeds = (1..=5).collect();
    }

    println!("# Evaluacion de valor — detector 06 (jitter/KS)");
    println!(
        "# TRIALS={TRIALS} pares/celda, WIN={WIN}, base_n={BASE_N}, mediana={}ms, sigma={SIGMA}",
        MEDIAN / 1e6
    );
    println!("# semillas: {seeds:?}");
    println!();
    println!("## Por semilla");
    println!("{:>6}  {:>6}  {:>8}  {:>10}  {:>10}", "seed", "alpha", "FPR", "TPR_naive", "TPR_mimic");
    let mut all: Vec<Vec<[f64; 3]>> = vec![Vec::new(); alphas.len()];
    for &sd in &seeds {
        for (i, &a) in alphas.iter().enumerate() {
            let c = cell(sd, a);
            println!("{:>6}  {:>6.3}  {:>8.4}  {:>10.4}  {:>10.4}", sd, a, c[0], c[1], c[2]);
            all[i].push(c);
        }
    }
    println!();
    println!("## Resumen (min–max, media) sobre {} semillas", seeds.len());
    println!("{:>6}  {:>24}  {:>24}  {:>24}", "alpha", "FPR", "TPR_naive", "TPR_mimic");
    for (i, &a) in alphas.iter().enumerate() {
        let mut cols = Vec::new();
        for k in 0..3 {
            let v: Vec<f64> = all[i].iter().map(|c| c[k]).collect();
            let mn = v.iter().cloned().fold(f64::MAX, f64::min);
            let mx = v.iter().cloned().fold(f64::MIN, f64::max);
            let me = v.iter().sum::<f64>() / v.len() as f64;
            cols.push(format!("{mn:.4}–{mx:.4} ({me:.4})"));
        }
        println!("{:>6.3}  {:>24}  {:>24}  {:>24}", a, cols[0], cols[1], cols[2]);
    }
}
