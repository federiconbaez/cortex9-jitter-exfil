#!/usr/bin/env bash
# Captura corta: baseline benigno + ventana mimic, evalúa con fnbr-jitter.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/fnbr-jitter"
[[ -x "$BIN" ]] || cargo build --release -q
OUT="${ROOT}/collector/runs"
mkdir -p "$OUT"
BT_LOG="$OUT/bt_mimic.tsv"
: > "$BT_LOG"

sudo -n bpftrace -o "$BT_LOG" collector/ssl_write.bt &
BTPID=$!
sleep 1
cleanup() { sudo -n kill "$BTPID" 2>/dev/null || true; wait "$BTPID" 2>/dev/null || true; }
trap cleanup EXIT

python3 scripts/generar_trafico.py --mode benign --n 300 --seed 11 --port 19543
python3 scripts/generar_trafico.py --mode mimic  --n 300 --seed 12 --port 19544
sleep 0.5
cleanup
trap - EXIT

python3 - <<'PY'
from collections import defaultdict
from pathlib import Path
rows = Path("collector/runs/bt_mimic.tsv").read_text().strip().splitlines()
by = defaultdict(list)
for line in rows:
    parts = line.split("\t")
    if len(parts) < 2: continue
    ts, pid = int(parts[0]), parts[1]
    by[pid].append(ts)
ranked = sorted(by.items(), key=lambda kv: -len(kv[1]))
print(f"pids: {[(p,len(v)) for p,v in ranked[:6]]}")
cands = [(p,v) for p,v in ranked if len(v) >= 50][:2]
cands.sort(key=lambda kv: kv[1][0])
names = ["baseline_mimic", "mimic_obs"]
for name, (pid, ts) in zip(names, cands):
    ts = sorted(ts)
    Path(f"collector/runs/{name}.txt").write_text("\n".join(map(str, ts)) + "\n")
    print(name, "pid", pid, "n", len(ts))
if len(cands) < 2:
    raise SystemExit(f"faltan capturas: {len(cands)}")
PY

echo "=== mimic vs baseline (alpha=0.001) ==="
rc=0
out=$("$BIN" collector/runs/baseline_mimic.txt collector/runs/mimic_obs.txt 0.001) || rc=$?
echo "$out"
echo "detector_exit=$rc (0 = no sospechoso, esperado para mimic)"
{
  echo "# Captura real mimic (uprobe SSL_write)"
  echo "# fecha: $(date -Iseconds)"
  echo "$out"
  echo "detector_exit=$rc"
} | tee collector/evidence/captura_mimic.txt
exit 0
