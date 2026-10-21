#!/usr/bin/env python3
"""Genera escrituras SSL con intervalos benignos o de canal encubierto.
Usa un servidor TLS local (mismo proceso no: cliente contra localhost).
"""
import argparse, os, socket, ssl, threading, time, random, math

def server(sock_path_or_port, stop):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    # certs created by caller
    ctx.load_cert_chain("collector/cert.pem", "collector/key.pem")
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("127.0.0.1", sock_path_or_port))
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

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["benign", "covert"], required=True)
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
            for i in range(args.n):
                if args.mode == "benign":
                    delay = lognormal(rng, 0.002, 0.6)  # ~2 ms mediana
                else:
                    base = 0.001 if rng.randrange(2) == 0 else 0.004
                    delay = base * math.exp(0.15 * rng.gauss(0, 1))
                time.sleep(max(delay, 0.00005))
                ss.sendall(b"x" * 16)
    stop.set()

if __name__ == "__main__":
    main()
