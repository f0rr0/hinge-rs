#!/usr/bin/env python3
"""Adapt archived documentation to the current, pinned Scalar renderer."""

import json
import re
import sys
from pathlib import Path

site = Path(sys.argv[1])
latest_html = (site / "index.html").read_text()
scalar_url = re.search(
    r'https://cdn\.jsdelivr\.net/npm/@scalar/api-reference@[^"\s]+',
    latest_html,
).group()
style = re.search(r"<style>.*?</style>", latest_html, re.DOTALL).group()
picker = re.search(r'<div class="version-picker">.*?</div>', latest_html, re.DOTALL).group()

for spec_path in site.glob("v/*/openapi.json"):
    spec = json.loads(spec_path.read_text())
    for environment in spec.get("x-scalar-environments", {}).values():
        variables = environment.get("variables")
        if isinstance(variables, dict):
            environment["variables"] = [
                {"name": name, "value": value} for name, value in variables.items()
            ]
    spec_path.write_text(json.dumps(spec, indent=2, ensure_ascii=False) + "\n")
    html_path = spec_path.with_name("index.html")
    html = re.sub(
        r'https://cdn\.jsdelivr\.net/npm/@scalar/api-reference(?:@[^"\s]+)?',
        scalar_url,
        html_path.read_text(),
    )
    html = re.sub(r"<style>.*?</style>", lambda _: style, html, flags=re.DOTALL)
    html = re.sub(r'<div class="version-picker">.*?</div>', lambda _: picker, html, flags=re.DOTALL)
    assert style in html and picker in html, f"Missing themed version picker in {html_path}"
    html_path.write_text(html)
