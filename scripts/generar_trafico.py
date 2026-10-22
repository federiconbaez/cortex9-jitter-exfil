#!/usr/bin/env python3
"""Genera escrituras SSL con intervalos benignos, de canal encubierto o de imitador.

Modos:
  benign  — lognormal(mediana ~2 ms, sigma 0.6)
  covert  — canal ingenuo 1 ms / 4 ms + ruido
  mimic   — adversario que imita la distribución: codifica bits eligiendo
            la mitad izquierda/derecha de la CDF lognormal. La distribución
            marginal es idéntica a la benigna; el KS no puede distinguirlo.
"""
import argparse, math, os, random, socket, ssl, subprocess, threading, time


def asegurar_cert():
    """Genera un par autofirmado de prueba si no existe (no se versiona)."""
    if not (os.path.exists("collector/cert.pem") and os.path.exists("collector/key.pem")):
        subprocess.run(["openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "30",
                        "-subj", "/CN=localhost", "-keyout", "collector/key.pem",
                        "-out", "collector/cert.pem"], check=True, capture_output=True)


def server(port, stop):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    asegurar_cert()
    ctx.load_cert_chain("collector/cert.pem", "collector/key.pem")
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("127.0.0.1", port))
        s.listen(1)
        s.settimeout(0.5)
        while not stop.is_set():
            try:
                conn, _ = s.accept()
            except socket.timeout:
                continue
            with ctx.wrap_socket(conn, server_side=True) as ss:
                try:
                    while True:
                        data = ss.recv(65536)
                        if not data:
                            break
                except Exception:
                    pass


def lognormal(rng, median, sigma):
    return median * math.exp(sigma * rng.gauss(0, 1))


def inv_normal_cdf(p):
    """Cuantil de la normal estándar (Acklam). Misma aproximación que en gen.rs."""
    a = (-3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02,
         1.383577518672690e+02, -3.066479806614716e+01, 2.506628277459239e+00)
    b = (-5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02,
         6.680131188771972e+01, -1.328068155288572e+01)
    c = (-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00,
         -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00)
    d = (7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00,
         3.754408661907416e+00)
    plow, phigh = 0.02425, 1.0 - 0.02425
    if p < plow:
        q = math.sqrt(-2.0 * math.log(p))
        return (((((c[0]*q + c[1])*q + c[2])*q + c[3])*q + c[4])*q + c[5]) / \
               ((((d[0]*q + d[1])*q + d[2])*q + d[3])*q + 1.0)
    if p <= phigh:
        q = p - 0.5
        r = q * q
        return (((((a[0]*r + a[1])*r + a[2])*r + a[3])*r + a[4])*r + a[5]) * q / \
               (((((b[0]*r + b[1])*r + b[2])*r + b[3])*r + b[4])*r + 1.0)
    q = math.sqrt(-2.0 * math.log(1.0 - p))
    return -(((((c[0]*q + c[1])*q + c[2])*q + c[3])*q + c[4])*q + c[5]) / \
            ((((d[0]*q + d[1])*q + d[2])*q + d[3])*q + 1.0)


def mimic_delay(rng, median, sigma):
    bit = rng.randrange(2)
    u = 0.5 * rng.random() if bit == 0 else 0.5 + 0.5 * rng.random()
    # evitar extremos exactos 0/1
    u = min(max(u, 1e-12), 1.0 - 1e-12)
    z = inv_normal_cdf(u)
    return median * math.exp(sigma * z)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["benign", "covert", "mimic"], required=True)
    ap.add_argument("--n", type=int, default=400)
    ap.add_argument("--port", type=int, default=19443)
    ap.add_argument("--seed", type=int, default=1)
    args = ap.parse_args()
    rng = random.Random(args.seed)
    stop = threading.Event()
    t = threading.Thread(target=server, args=(args.port, stop), daemon=True)
    t.start()
    time.sleep(0.2)
    ctx = ssl.create_default_context()
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE
    with socket.create_connection(("127.0.0.1", args.port), timeout=2) as raw:
        with ctx.wrap_socket(raw, server_hostname="localhost") as ss:
            for _ in range(args.n):
                if args.mode == "benign":
                    delay = lognormal(rng, 0.002, 0.6)
                elif args.mode == "covert":
                    base = 0.001 if rng.randrange(2) == 0 else 0.004
                    delay = base * math.exp(0.15 * rng.gauss(0, 1))
                else:  # mimic
                    delay = mimic_delay(rng, 0.002, 0.6)
                time.sleep(max(delay, 0.00005))
                ss.sendall(b"x" * 16)
    stop.set()


if __name__ == "__main__":
    main()
