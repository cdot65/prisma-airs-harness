"""Exercise the compiled default HTTP client against inconsistent A/AAAA answers.

QEMU's filesystem prefix gives only the child a test resolv.conf; the host DNS
configuration is never changed. Requires QEMU, OpenSSL and permission to bind an
unused loopback address on UDP port 53 (the build containers allow this).
"""

import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import socket
import ssl
import struct
import subprocess
import tempfile
import threading


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--emulator", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    requests = []

    class Origin(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            requests.append(self.path)
            self.send_response(200)
            self.send_header("Content-Length", "2")
            self.end_headers()
            self.wfile.write(b"ok")

        def log_message(self, *_):
            pass

    with tempfile.TemporaryDirectory(prefix="airs-musl-dns-") as temporary:
        root = Path(temporary)
        prefix = root / "prefix"
        (prefix / "etc").mkdir(parents=True)
        (prefix / "etc/resolv.conf").write_text(
            "nameserver 127.0.0.54\noptions timeout:1 attempts:1\nsearch .\n"
        )
        certificate, key = root / "ca.pem", root / "key.pem"
        subprocess.run(
            [
                "openssl",
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-keyout",
                str(key),
                "-out",
                str(certificate),
                "-days",
                "1",
                "-subj",
                "/CN=issuer.airs.invalid",
                "-addext",
                "subjectAltName=DNS:issuer.airs.invalid",
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        origin = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Origin)
        tls = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        tls.load_cert_chain(certificate, key)
        origin.socket = tls.wrap_socket(origin.socket, server_side=True)
        origin_thread = threading.Thread(target=origin.serve_forever, daemon=True)
        origin_thread.start()
        results = []
        try:
            for case in ["aaaa-nxdomain", "aaaa-nodata", "both-nxdomain"]:
                dns = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
                dns.bind(("127.0.0.54", 53))
                dns.settimeout(0.1)
                stop = threading.Event()
                queries = []

                def answer():
                    while not stop.is_set():
                        try:
                            packet, peer = dns.recvfrom(4096)
                        except socket.timeout:
                            continue
                        offset = 12
                        while packet[offset]:
                            offset += packet[offset] + 1
                        offset += 1
                        kind = struct.unpack("!H", packet[offset : offset + 2])[0]
                        queries.append(kind)
                        missing = case == "both-nxdomain" or (
                            case == "aaaa-nxdomain" and kind == 28
                        )
                        record = b""
                        if kind == 1 and not missing:
                            record = b"\xc0\x0c" + struct.pack("!HHIH", 1, 1, 0, 4)
                            record += socket.inet_aton("127.0.0.1")
                        header = packet[:2] + struct.pack(
                            "!HHHHH",
                            0x8183 if missing else 0x8180,
                            1,
                            int(bool(record)),
                            0,
                            0,
                        )
                        dns.sendto(header + packet[12 : offset + 4] + record, peer)

                thread = threading.Thread(target=answer, daemon=True)
                thread.start()
                env = {
                    k: v
                    for k, v in os.environ.items()
                    if k.upper()
                    not in {
                        "HTTP_PROXY",
                        "HTTPS_PROXY",
                        "ALL_PROXY",
                        "NO_PROXY",
                        "SSL_CERT_FILE",
                        "SSL_CERT_DIR",
                        "CODEX_CA_CERTIFICATE",
                        "CODEX_CUSTOM_CA_PROBE_PROXY",
                        "CODEX_CUSTOM_CA_PROBE_TLS13",
                    }
                }
                env.update(
                    CODEX_CA_CERTIFICATE=str(certificate),
                    CODEX_CUSTOM_CA_PROBE_URL=(
                        f"https://issuer.airs.invalid:{origin.server_port}/discovery"
                    ),
                )
                before = len(requests)
                try:
                    result = subprocess.run(
                        [args.emulator, "-L", str(prefix), str(args.probe.resolve())],
                        env=env,
                        text=True,
                        capture_output=True,
                        timeout=20,
                    )
                finally:
                    stop.set()
                    thread.join(timeout=2)
                    dns.close()
                output = result.stdout + result.stderr
                (args.output / f"{case}.log").write_text(output)
                expected_success = case != "both-nxdomain"
                passed = {1, 28}.issubset(queries)
                if expected_success:
                    passed &= result.returncode == 0 and result.stdout.strip() == "ok"
                    passed &= len(requests) == before + 1
                else:
                    passed &= result.returncode != 0 and "dns" in output.lower()
                    passed &= len(requests) == before
                results.append(
                    {
                        "case": case,
                        "passed": passed,
                        "queries": queries,
                        "exit_code": result.returncode,
                        "https_requests": len(requests) - before,
                    }
                )
        finally:
            origin.shutdown()
            origin.server_close()
            origin_thread.join(timeout=2)
    receipt = {
        "passed": all(case["passed"] for case in results),
        "scope": "emulated musl default-client DNS and verified local HTTPS",
        "probe_sha256": hashlib.file_digest(
            args.probe.open("rb"), "sha256"
        ).hexdigest(),
        "emulator": args.emulator,
        "cases": results,
        "native_host_acceptance": False,
    }
    (args.output / "DNS-REGRESSION.json").write_text(
        json.dumps(receipt, indent=2) + "\n"
    )
    print(json.dumps(receipt))
    raise SystemExit(0 if receipt["passed"] else 1)


if __name__ == "__main__":
    main()
