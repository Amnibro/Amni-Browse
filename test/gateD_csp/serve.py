import base64, http.server, os, sys
PNG = base64.b64decode(b'iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAAGklEQVR42mM4MdvvPyWYYdSAUQNGDRguBgAAqeuwHwKv5uEAAAAASUVORK5CYII=')
class H(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.0'
    def log_message(self, fmt, *a):
        sys.stderr.write('%s - %s\n' % (self.address_string(), fmt % a)); sys.stderr.flush()
    def do_GET(self):
        body, ctype = (PNG, 'image/png') if self.path.startswith('/favicon.ico') else (b'<!doctype html><html><head><title>csp</title></head><body>connect-src none</body></html>', 'text/html; charset=utf-8')
        self.send_response(200)
        self.send_header('Content-Type', ctype)
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Content-Security-Policy', "default-src 'self'; connect-src 'none'; script-src 'self' 'unsafe-inline'")
        self.end_headers()
        self.wfile.write(body)
if __name__ == '__main__':
    http.server.ThreadingHTTPServer(('127.0.0.1', int(sys.argv[1])), H).serve_forever()
