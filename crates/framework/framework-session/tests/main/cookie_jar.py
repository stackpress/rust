"""Exercise RFC cookie scope/expiry with Python's installed jar; never open a URL."""

import http.cookiejar
import json
import sys
import urllib.request
from email.message import Message


class Response:
    """Expose only the in-memory response headers consumed by CookieJar."""

    def __init__(self, headers):
        self.headers = Message()
        for header in headers:
            self.headers.add_header("Set-Cookie", header)

    def info(self):
        return self.headers


data = json.load(sys.stdin)
jar = http.cookiejar.CookieJar()
for response in data["responses"]:
    jar.extract_cookies(Response(response["headers"]), urllib.request.Request(response["url"]))
headers = []
for target in data["targets"]:
    request = urllib.request.Request(target)
    jar.add_cookie_header(request)
    headers.append(request.get_header("Cookie", ""))
json.dump(headers, sys.stdout)
