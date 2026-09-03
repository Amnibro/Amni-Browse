import pathlib, subprocess, sys
root = pathlib.Path(__file__).parent
cert, key = root / "cert.pem", root / "key.pem"
if cert.exists() and key.exists():
    print("cert ok")
    sys.exit(0)
try:
    subprocess.run(
        [
            "openssl",
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-keyout",
            str(key),
            "-out",
            str(cert),
            "-days",
            "365",
            "-nodes",
            "-subj",
            "/CN=localhost",
            "-addext",
            "subjectAltName=DNS:localhost,IP:127.0.0.1",
        ],
        check=True,
        capture_output=True,
    )
    print("cert generated")
except (FileNotFoundError, subprocess.CalledProcessError) as e:
    print("cert fail:", e, file=sys.stderr)
    sys.exit(1)
