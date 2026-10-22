# 06 — Detector de exfiltración por jitter

> **ES:** Detector de exfiltración por canal encubierto de tiempos (jitter) en escrituras SSL. Compilar/probar: `cargo build --release && cargo test --release`. El certificado TLS de prueba se genera solo al correr `scripts/generar_trafico.py` (no se versiona).
> **EN:** Detector for timing-jitter covert-channel exfiltration over SSL writes. Build/test: `cargo build --release && cargo test --release`. The test TLS cert is generated at run time by `scripts/generar_trafico.py` (not versioned).
>
> **Estado / Status:** resultados en VALOR.md; **no auditado** por un tercero / **not independently audited**. Licencia / License: Apache-2.0.


Dueña: Federica Research Sentinel. Pedido de Federico, 20/10/2026.

## Qué hace (v0.2)
1. **Colector eBPF real**: `collector/ssl_write.bt` engancha `SSL_write` /
   `SSL_write_ex` de `libssl.so.3` con bpftrace y escribe marcas de tiempo (ns).
2. **Detector**: Kolmogórov-Smirnov de dos muestras (alpha = 0.001) de los
   intervalos contra la línea base del proceso.
3. CLI: `fnbr-jitter linea_base.txt observado.txt [alpha]`.

## Estado verificado
- `cargo build --release` + `cargo clippy -- -D warnings`: limpios.
- `cargo test --release`: 7/7 (sintético: FPR≈alpha, TPR_naive=100 %, TPR_mimic≈FPR).
- **Captura real con uprobe** (`scripts/captura-real.sh`, 20/10/2026):
  - 500 + 400 + 400 llamadas a `SSL_write` capturadas en `libssl.so.3`.
  - Benigno vs línea base: `suspicious=false` → sin falsa alarma.
  - Canal encubierto 1 ms/4 ms vs línea base: `suspicious=true`.
  - Los valores-p varían entre corridas (muestras y PIDs distintos). En mediciones
    independientes el benigno dio p ≈ 1,6e-2 … 1,9e-1 (siempre por encima del umbral,
    no sospechoso) y el canal encubierto p ≈ 10⁻¹⁵ … 10⁻¹⁸ (siempre muy por debajo).
    Lo estable es el **veredicto**, no el p exacto; para reproducir, correr el script,
    no copiar un número.

## Lo que NO está verificado
- Tráfico de red real hacia Internet (la prueba usa TLS local).
- ~~Adversario que imite la distribución~~ → medido en VALOR.md: el KS no lo distingue.
- `libcrypto` por separado; hoy solo `libssl` write path.
- MCP/gRPC sobre stdio no pasa por SSL: este detector no los ve.


## Experimento de valor (adversario imitador)

Ver **[VALOR.md](VALOR.md)**: métrica TPR@FPR fijo contra un adversario que imita
la distribución de la línea base (`mimic`), comparado con el canal ingenuo 1 ms/4 ms
y con «sin detector». Resultado: el KS retiene valor contra el ingenuo
(`TPR_naive = 1`); contra el imitador el valor **colapsa** (`TPR_mimic ≈ FPR`).
Reproducir: `cargo run --release --example eval_valor`.

## Cómo reproducir
```bash
cargo test --release
sudo apt-get install -y bpftrace   # si falta
sudo ./scripts/captura-real.sh
```
