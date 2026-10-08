#!/usr/bin/env python3
"""Adapt archived documentation to the current, pinned Scalar renderer."""

import json
import re
import sys
from pathlib import Path

site = Path(sys.argv[1])
scalar_url = re.search(
    r'https://cdn\.jsdelivr\.net/npm/@scalar/api-reference@[^"\s]+',
    (site / "index.html").read_text(),
).group()

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
    html_path.write_text(re.sub(
        r'https://cdn\.jsdelivr\.net/npm/@scalar/api-reference(?:@[^"\s]+)?',
        scalar_url,
        html_path.read_text(),
    ))
