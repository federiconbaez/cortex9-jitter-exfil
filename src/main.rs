//! CLI: `fnbr-jitter <linea_base.txt> <observado.txt> [alpha]`
//! Cada archivo: una marca de tiempo en ns por línea (orden ascendente).
use std::{env, fs, process::ExitCode};

fn load(path: &str) -> Result<Vec<u64>, String> {
    let s = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    s.lines().filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().parse::<u64>().map_err(|e| format!("{path}: {e}")))
        .collect()
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: fnbr-jitter <linea_base.txt> <observado.txt> [alpha]");
        return ExitCode::from(2);
    }
    let alpha = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(0.001);
    let (b, o) = match (load(&args[1]), load(&args[2])) {
        (Ok(b), Ok(o)) => (fnbr_jitter::intervals(&b), fnbr_jitter::intervals(&o)),
        (Err(e), _) | (_, Err(e)) => { eprintln!("{e}"); return ExitCode::from(2); }
    };
    if b.is_empty() || o.is_empty() { eprintln!("se necesitan al menos 2 marcas por archivo"); return ExitCode::from(2); }
    let v = fnbr_jitter::detect(&b, &o, alpha);
    println!("{{\"d\":{:.6},\"p_value\":{:.3e},\"suspicious\":{}}}", v.d, v.p_value, v.suspicious);
    if v.suspicious { ExitCode::from(1) } else { ExitCode::SUCCESS }
}
