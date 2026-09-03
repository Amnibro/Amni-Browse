import base64, http.server, os, ssl, sys, threading
PNG = base64.b64decode(
    b"iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAGklEQVR42mM4MdvvPyWYYdSAUQNGDRguBgAAqeuwHwKv5uEAAAAASUVORK5CYII="
)
HTTPS_PORT = int(sys.argv[1])
HTTP_PORT = HTTPS_PORT + 1
ROOT = os.path.dirname(os.path.abspath(__file__))
CERT = os.path.join(ROOT, "cert.pem")
KEY = os.path.join(ROOT, "key.pem")


class Cleartext(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"

    def log_message(self, fmt, *a):
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % a))
        sys.stderr.flush()

    def do_GET(self):
        if self.path.startswith("/favicon.ico"):
            body, ctype = PNG, "image/png"
        else:
            body, ctype = b"unexpected", "text/plain"
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


class Secure(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"

    def log_message(self, fmt, *a):
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % a))
        sys.stderr.flush()

    def do_GET(self):
        if self.path.startswith("/favicon.ico"):
            loc = "http://127.0.0.1:%d/favicon.ico" % HTTP_PORT
            self.send_response(302)
            self.send_header("Location", loc)
            self.end_headers()
            return
        body = b"<!doctype html><html><head><title>mixed</title></head><body>no link rel icon</body></html>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def main():
    if not (os.path.isfile(CERT) and os.path.isfile(KEY)):
        print("missing cert.pem/key.pem; run make_fixture.py", file=sys.stderr)
        sys.exit(2)
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", HTTP_PORT), Cleartext)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    httpsd = http.server.ThreadingHTTPServer(("127.0.0.1", HTTPS_PORT), Secure)
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(CERT, KEY)
    httpsd.socket = ctx.wrap_socket(httpsd.socket, server_side=True)
    httpsd.serve_forever()


if __name__ == "__main__":
    main()
