#!python3

import http.server
import socketserver

PORT = 8000

if __name__ == "__main__":
    Handler = http.server.SimpleHTTPRequestHandler

    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print("Serving webpage on port", PORT)
        httpd.serve_forever()
