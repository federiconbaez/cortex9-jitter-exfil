#!/usr/bin/env bash
# Captura SSL_write con bpftrace y evalúa con fnbr-jitter.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/fnbr-jitter"
[[ -x "$BIN" ]] || cargo build --release -q
OUT="${ROOT}/collector/runs"
mkdir -p "$OUT"
BT_LOG="$OUT/bt.tsv"
: > "$BT_LOG"

sudo bpftrace -o "$BT_LOG" collector/ssl_write.bt &
BTPID=$!
sleep 1
cleanup() { sudo kill "$BTPID" 2>/dev/null || true; wait "$BTPID" 2>/dev/null || true; }
trap cleanup EXIT

# línea base benigna
python3 scripts/generar_trafico.py --mode benign --n 500 --seed 1 --port 19443
# ventana observada benigna (para FPR)
python3 scripts/generar_trafico.py --mode benign --n 400 --seed 2 --port 19444
# ventana covert
python3 scripts/generar_trafico.py --mode covert --n 400 --seed 3 --port 19445
sleep 0.5
cleanup
trap - EXIT

# Separar por PID: tomar los 3 PIDs más activos (más líneas)
python3 - <<'PY'
from collections import Counter, defaultdict
from pathlib import Path
rows = Path("collector/runs/bt.tsv").read_text().strip().splitlines()
by = defaultdict(list)
for line in rows:
    parts = line.split("\t")
    if len(parts) < 2: continue
    ts, pid = int(parts[0]), parts[1]
    by[pid].append(ts)
# ordenar pids por cantidad
ranked = sorted(by.items(), key=lambda kv: -len(kv[1]))
print(f"pids capturados: {[(p,len(v)) for p,v in ranked[:6]]}")
# esperamos al menos 3 procesos python con SSL_write
cands = [(p,v) for p,v in ranked if len(v) >= 50][:3]
# ordenar por primer timestamp = orden de generación
cands.sort(key=lambda kv: kv[1][0])
names = ["baseline", "benign_obs", "covert"]
for name, (pid, ts) in zip(names, cands):
    ts = sorted(ts)
    Path(f"collector/runs/{name}.txt").write_text("\n".join(map(str, ts)) + "\n")
    print(name, "pid", pid, "n", len(ts))
if len(cands) < 3:
    raise SystemExit(f"faltan capturas: solo {len(cands)} procesos con >=50 writes")
PY

# El detector devuelve !=0 cuando marca sospechoso; eso es un RESULTADO, no un fallo
# del script, as\u00ed que capturamos el c\u00f3digo sin que set -e aborte.
echo "=== FPR (benign vs baseline) ==="
rc_fpr=0; "$BIN" collector/runs/baseline.txt collector/runs/benign_obs.txt 0.001 || rc_fpr=$?
echo "detector_fpr_exit=$rc_fpr (0 = no sospechoso, esperado)"
echo "=== TPR (covert vs baseline) ==="
rc_tpr=0; "$BIN" collector/runs/baseline.txt collector/runs/covert.txt 0.001 || rc_tpr=$?
echo "detector_tpr_exit=$rc_tpr (!=0 = canal detectado, esperado)"
# El script termina OK si corri\u00f3 bien, con independencia del veredicto del detector.
exit 0
