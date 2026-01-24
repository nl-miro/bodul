#!/usr/bin/env python3
"""
Fetch DM product-search JSON similar to the provided curl command.

Install:
  pip install "httpx[brotli,zstd,http2]"

Run:
  python dm_fetch.py

Optional:
  python dm_fetch.py --out data.json
  python dm_fetch.py --category 110105 --pageSize 15 --out data.json
"""

import argparse
import json
import sys
import time
from urllib.parse import urlencode

import httpx


BASE_URL = "https://product-search.services.dmtech.com/hr/search/static"

DEFAULT_PARAMS = {
    "allCategories.id": "110105",
    "pageSize": "100",
    "currentPage": "0",
    "searchType": "editorial-search",
    "sort": "editorial_relevance",
    "type": "search-static",
}

BASE_HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.10 Safari/605.1.1",
    "Accept": "application/json, text/plain, */*",
    "Accept-Language": "hr-HR,hr;q=0.9,en;q=0.8",
    "Accept-Encoding": "gzip, deflate, br, zstd",
    "Referer": "https://www.dm.hr/",
    "x-dm-product-search-tags": "presentation:grid;search-type:editorial;channel:web;editorial-type:category",
    # x-dm-product-search-token is set dynamically per request below (27 * Date.now()).
    "Origin": "https://www.dm.hr",
    "Connection": "keep-alive",
    "Sec-Fetch-Dest": "empty",
    "Sec-Fetch-Mode": "cors",
    "Sec-Fetch-Site": "cross-site",
    "TE": "trailers",
}


def make_token() -> str:
    # Equivalent to JS: 27 * Date.now()
    return str(27 * int(time.time() * 1000))


def build_url(category: str, page_size: int) -> str:
    params = dict(DEFAULT_PARAMS)
    params["allCategories.id"] = str(category)
    params["pageSize"] = str(page_size)
    return f"{BASE_URL}?{urlencode(params)}"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--category", default=DEFAULT_PARAMS["allCategories.id"], help="Category ID (allCategories.id)")
    ap.add_argument("--pageSize", type=int, default=int(DEFAULT_PARAMS["pageSize"]), help="Page size")
    ap.add_argument("--out", help="Write response JSON to a file instead of stdout")
    ap.add_argument("--timeout", type=float, default=30.0)
    args = ap.parse_args()

    url = build_url(args.category, args.pageSize)

    headers = dict(BASE_HEADERS)
    headers["x-dm-product-search-token"] = make_token()

    with httpx.Client(http2=True, headers=headers, timeout=args.timeout, follow_redirects=True) as client:
        r = client.get(url)

    # Diagnostics to stderr (keeps stdout clean JSON)
    print(f"HTTP {r.status_code}", file=sys.stderr)
    for k in ("content-type", "content-encoding", "cache-control"):
        if k in r.headers:
            print(f"{k}: {r.headers[k]}", file=sys.stderr)

    r.raise_for_status()

    data = r.json()

    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
    else:
        json.dump(data, sys.stdout, ensure_ascii=False, indent=2)
        print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
