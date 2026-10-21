//! Detector de exfiltración por canal de temporización.
//!
//! Entrada: marcas de tiempo (ns) de escrituras salientes de un proceso
//! (por ejemplo, llamadas a `SSL_write` capturadas con un uprobe).
//! Salida: un veredicto basado en la prueba de Kolmogórov-Smirnov de dos
//! muestras entre los intervalos observados y una línea base del proceso.

pub mod gen;

/// Intervalos entre salidas consecutivas. Requiere marcas ordenadas.
pub fn intervals(ts_ns: &[u64]) -> Vec<f64> {
    ts_ns.windows(2).map(|w| w[1].saturating_sub(w[0]) as f64).collect()
}

/// Estadístico D de Kolmogórov-Smirnov de dos muestras.
pub fn ks_statistic(a: &[f64], b: &[f64]) -> f64 {
    assert!(!a.is_empty() && !b.is_empty(), "muestras vacías");
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_by(f64::total_cmp);
    b.sort_by(f64::total_cmp);
    let (n, m) = (a.len() as f64, b.len() as f64);
    let (mut i, mut j, mut d) = (0usize, 0usize, 0f64);
    while i < a.len() && j < b.len() {
        let x = a[i].min(b[j]);
        while i < a.len() && a[i] <= x { i += 1; }
        while j < b.len() && b[j] <= x { j += 1; }
        d = d.max((i as f64 / n - j as f64 / m).abs());
    }
    d
}

/// Valor p asintótico de KS (serie de Kolmogórov).
pub fn ks_pvalue(d: f64, n: usize, m: usize) -> f64 {
    let ne = (n * m) as f64 / (n + m) as f64;
    let lambda = (ne.sqrt() + 0.12 + 0.11 / ne.sqrt()) * d;
    if lambda < 1e-3 { return 1.0; }
    let mut sum = 0.0;
    for k in 1..=100 {
        let k = k as f64;
        let term = 2.0 * (-1f64).powf(k - 1.0) * (-2.0 * k * k * lambda * lambda).exp();
        sum += term;
        if term.abs() < 1e-12 { break; }
    }
    sum.clamp(0.0, 1.0)
}

/// Entropía de Shannon (bits) del histograma de intervalos en escala log2.
/// Un canal binario de temporización tiende a concentrar masa en pocos bins.
pub fn log_histogram_entropy(iv: &[f64], bins: usize) -> f64 {
    let logs: Vec<f64> = iv.iter().map(|&x| (x.max(1.0)).log2()).collect();
    let (lo, hi) = logs.iter().fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    if hi - lo < 1e-9 { return 0.0; }
    let mut h = vec![0usize; bins];
    for x in &logs {
        let k = (((x - lo) / (hi - lo)) * bins as f64) as usize;
        h[k.min(bins - 1)] += 1;
    }
    let n = logs.len() as f64;
    h.iter().filter(|&&c| c > 0).map(|&c| { let p = c as f64 / n; -p * p.log2() }).sum()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub d: f64,
    pub p_value: f64,
    pub suspicious: bool,
}

/// Compara una ventana observada contra la línea base del mismo proceso.
pub fn detect(baseline: &[f64], observed: &[f64], alpha: f64) -> Verdict {
    let d = ks_statistic(baseline, observed);
    let p = ks_pvalue(d, baseline.len(), observed.len());
    Verdict { d, p_value: p, suspicious: p < alpha }
}
