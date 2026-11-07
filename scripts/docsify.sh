#!/usr/bin/env bash
#
# docsify.sh — Copy tracers graph outputs into docs/ for GitHub Pages hosting.
#
# Copies .tracers/graph.html and .tracers/graph.json into docs/ and creates a
# .nojekyll file so GitHub Pages serves them as-is.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRACERS_DIR="${REPO_ROOT}/.tracers"
DOCS_DIR="${REPO_ROOT}/docs"

mkdir -p "${DOCS_DIR}"

if [[ -f "${TRACERS_DIR}/graph.html" ]]; then
    cp "${TRACERS_DIR}/graph.html" "${DOCS_DIR}/graph.html"
    mv "${DOCS_DIR}/graph.html" "${DOCS_DIR}/index.html"
fi

if [[ -f "${TRACERS_DIR}/graph.json" ]]; then
    cp "${TRACERS_DIR}/graph.json" "${DOCS_DIR}/graph.json"
fi

touch "${DOCS_DIR}/.nojekyll"

echo "docsify: copied graph outputs to docs/"
