# 06 — Detector de exfiltración por jitter

Dueña: Federica Research Sentinel. Pedido de Federico, 20/10/2026.

## Qué hace (v0.2)
1. **Colector eBPF real**: `collector/ssl_write.bt` engancha `SSL_write` /
   `SSL_write_ex` de `libssl.so.3` con bpftrace y escribe marcas de tiempo (ns).
2. **Detector**: Kolmogórov-Smirnov de dos muestras (alpha = 0.001) de los
   intervalos contra la línea base del proceso.
3. CLI: `fnbr-jitter linea_base.txt observado.txt [alpha]`.

## Estado verificado
- `cargo build --release` + `cargo clippy -- -D warnings`: limpios.
- `cargo test --release`: 5/5 (sintético: FPR 0,2 %, TPR 100 %).
- **Captura real con uprobe** (`scripts/captura-real.sh`, 20/10/2026):
  - 500 + 400 + 400 llamadas a `SSL_write` capturadas en `libssl.so.3`.
  - Benigno vs línea base: `suspicious=false` (p ≈ 3,6e-2) → sin falsa alarma.
  - Canal encubierto 1 ms/4 ms vs línea base: `suspicious=true` (p ≈ 2,8e-18).

## Lo que NO está verificado
- Tráfico de red real hacia Internet (la prueba usa TLS local).
- Adversario que imite la distribución de la línea base (límite del KS).
- `libcrypto` por separado; hoy solo `libssl` write path.
- MCP/gRPC sobre stdio no pasa por SSL: este detector no los ve.

## Cómo reproducir
```bash
cargo test --release
sudo apt-get install -y bpftrace   # si falta
sudo ./scripts/captura-real.sh
```
