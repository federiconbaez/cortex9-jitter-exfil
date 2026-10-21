# Valor del detector 06 (jitter / KS)

Dueña: Federica Research Sentinel. Experimento de valor, 21/10/2026.

## Métrica

**TPR a un FPR fijo (alpha)** frente a un adversario que *intenta imitar* la
distribución de retardos de la línea base. La comparación conceptual es
**«sin detector»** (TPR = 0, FPR = 0): el detector aporta valor solo si
`TPR >> FPR` para el adversario dado.

## Línea base a vencer

1. Tráfico **benigno**: intervalos log-normales (mediana 2 ms, σ = 0,6).
2. Adversario **ingenuo** (`covert`): bits 1 ms / 4 ms + ruido log-normal σ = 0,15
   (el canal ya medido en v0.2).
3. Adversario **imitador** (`mimic`): cada bit elige la mitad izquierda o derecha
   de la CDF log-normal de la línea base. Con bits equilibrados, la distribución
   *marginal* es idéntica a la benigna. Es el límite teórico del KS de dos muestras
   (ya advertido en el README: «adversario que imite la distribución»).

## Experimento

- Generadores sintéticos reproducibles en `src/gen.rs` (`benign`, `covert`, `mimic`).
- Evaluación: `cargo run --release --example eval_valor`.
- Por celda: 1000 pares, ventana = 400 intervalos, línea base = 5000; 5 semillas maestras (1..5).
- Alphas: {0,001 · 0,01 · 0,05}.
- Salida cruda: `collector/evidence/eval_valor_semillas.txt` (versionada; solo
  `collector/runs/` está gitignorado).

## Resultados (sintético, 5 semillas, 21/10/2026)

Una semilla fija prueba reproducibilidad, no robustez. Por eso la evaluación
corre ahora 5 semillas maestras (1..5), cada una con su propia línea base y sus
1000 pares por celda (WIN = 400, base = 5000). Rango min–max y media:

| alpha | FPR                    | TPR_naive              | TPR_mimic              |
|------:|:-----------------------|:-----------------------|:-----------------------|
| 0,001 | 0,0000–0,0020 (0,0008) | 1,0000–1,0000 (1,0000) | 0,0000–0,0030 (0,0012) |
| 0,010 | 0,0060–0,0150 (0,0106) | 1,0000–1,0000 (1,0000) | 0,0040–0,0120 (0,0078) |
| 0,050 | 0,0310–0,0510 (0,0398) | 1,0000–1,0000 (1,0000) | 0,0430–0,0590 (0,0498) |

- **TPR_naive = 1 en las 5 semillas** y en los tres alphas.
- **TPR_mimic queda pegado al FPR en todas**: los rangos de TPR_mimic y FPR se
  superponen en cada alpha, y las dos medias quedan cerca del alpha nominal.
  El detector no distingue al imitador en ninguna semilla.

Salida cruda por semilla: `collector/evidence/eval_valor_semillas.txt`.

Reproducir (por defecto semillas 1..5; también `SEMILLAS="7 8"` o argumentos):

```bash
cargo run --release --example eval_valor
cargo run --release --example eval_valor -- 1 2 3
```

La corrida anterior, con una sola semilla fija (commit `d9fe2cb`), queda en
`collector/evidence/eval_valor.txt`. Usaba otro esquema de semillas, así que
sus números no coinciden con los de esta tabla, pero las conclusiones son las mismas.

## Interpretación honesta

- Contra el adversario **ingenuo**, el detector tiene valor claro:
  `TPR_naive = 1,0` con `FPR ≈ alpha`. Esto confirma el resultado de v0.2
  (captura real con uprobe: veredicto estable).
- Contra el adversario **imitador**, `TPR_mimic ≈ FPR` en los tres alphas
  (dentro del ruido de muestreo). **El valor del KS colapsa**: no distingue al
  imitador del tráfico legítimo. No es un fallo de implementación: es el límite
  de cualquier prueba que mire solo la distribución *marginal* de los intervalos.
  El canal sigue transmitiendo bits (mitad izquierda / derecha), pero el detector
  no los ve.
- Veredicto de una frase: **el detector retiene valor contra el canal 1 ms/4 ms;
  contra un adversario que imita la distribución, el valor colapsa a ~FPR.**
  Se sostiene en las 5 semillas.

## Captura real con uprobe (complementaria)

Además del sintético, corrí una captura corta con bpftrace sobre TLS local
(`scripts/captura_mimic.sh`, 300 escrituras baseline + 300 mimic):

```
{"d":0.070234,"p_value":4.387e-1,"suspicious":false}
```

Veredicto: **no sospechoso** a alpha = 0,001 (p ≈ 0,44). Coincide con el
sintético. Evidencia: `collector/evidence/captura_mimic.txt`. El sintético
(5 semillas × 1000 pares/celda) sigue siendo la evidencia principal de tasas; la captura
real confirma que el uprobe no cambia el veredicto.

## Qué haría falta para valor C real

1. **Red-team adaptativo**: un adversario que estime la distribución online y
   ajuste el sesgo (o use dependencias temporales / orden de bits) en vez de
   un `mimic` estático.
2. **Tráfico de red real** hacia Internet (hoy es TLS local); medir FPR bajo
   jitter de cola de red y multipath.
3. **Detector de segundo orden**: mirar no solo la marginal (KS), sino
   autocorrelación, entropía condicional bit-a-bit, o un clasificador entrenado
   contra el imitador. El KS solo no alcanza.
4. Sesión con Federico / agentes reales midiendo si el detector ahorra tiempo
   o atrapa un intento concreto — eso es C de producción, no B de laboratorio.
