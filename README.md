# 06 — Detector de exfiltración por jitter

Dueña: Federica Research Sentinel. Pedido de Federico, 20/10/2026.

## Qué hace (v0.1)
Compara la distribución de intervalos entre escrituras salientes de un proceso
contra su propia línea base (Kolmogórov-Smirnov de dos muestras, alpha = 0.001)
y marca ventanas sospechosas. Incluye entropía del histograma log2 como señal secundaria.

CLI: `fnbr-jitter linea_base.txt observado.txt [alpha]` (una marca en ns por línea).
Sale con 1 si es sospechoso, 0 si no, 2 si hay error.

## Estado verificado
- `cargo build --release`: limpio.
- `cargo clippy --all-targets -- -D warnings`: limpio.
- `cargo test --release`: 5/5 en verde.
- Métricas medidas, **solo sobre datos sintéticos** (500 ventanas de 400 intervalos):
  falsos positivos 0,2 % (1/500); detección 100 % (500/500) de un canal binario 1 ms / 4 ms con ruido log-normal.

## Lo que NO está verificado
- **Captura real**: el uprobe sobre `SSL_write` (libssl) todavía no existe. El kernel de la
  computadora compartida acepta eBPF como root, pero XDP/uprobes no se probaron.
- **Tráfico real**: la línea base log-normal es un supuesto; el tráfico real puede ser multimodal
  y disparar falsos positivos.
- **Adversario adaptativo**: un canal que imite la distribución de la línea base (por ejemplo, con
  muestreo por rechazo) no lo detecta KS. Es un límite conocido de este enfoque, no un bug.
- La entropía separa poco (4,12 vs 4,02 bits en la prueba): señal débil, no usar sola.

## Próximo paso
Colector eBPF (uprobe en `SSL_write`, marca de tiempo por PID) que escriba el formato del CLI,
y evaluación con tráfico real grabado.
