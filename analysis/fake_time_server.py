#!/usr/bin/env python3
"""HTTP server that reports a fake Date header based on the URL path.

GET /         -> Date header with real time
GET /3.5      -> Date header offset by +3.5 seconds
GET /3.5?q=xx -> same, query string ignored
"""

import argparse
import time
from email.utils import formatdate
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def handle_request(self):
        path = self.path.split("?")[0].strip("/")
        offset = float(path) if path else 0.0
        self.send_response_only(200)
        self.send_header("Date", formatdate(timeval=time.time() + offset, usegmt=True))
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", "0")
        self.end_headers()

    do_GET = handle_request
    do_HEAD = handle_request

    def log_message(self, fmt, *args):
        pass


def main():
    ap = argparse.ArgumentParser(description="Fake time HTTP server")
    ap.add_argument("--port", type=int, default=8080)
    args = ap.parse_args()
    server = ThreadingHTTPServer(("0.0.0.0", args.port), Handler)
    print(f"Serving on 0.0.0.0:{args.port}")
    server.serve_forever()


if __name__ == "__main__":
    main()
