#!/usr/bin/env bash
#
# docsify.sh — Copy trace graph outputs into docs/ for GitHub Pages hosting.
#
# Copies .trace/graph.html and .trace/graph.json into docs/ and creates a
# .nojekyll file so GitHub Pages serves them as-is.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRACE_DIR="${REPO_ROOT}/.trace"
DOCS_DIR="${REPO_ROOT}/docs"

mkdir -p "${DOCS_DIR}"

if [[ -f "${TRACE_DIR}/graph.html" ]]; then
    cp "${TRACE_DIR}/graph.html" "${DOCS_DIR}/graph.html"
    mv "${DOCS_DIR}/graph.html" "${DOCS_DIR}/index.html"
fi

if [[ -f "${TRACE_DIR}/graph.json" ]]; then
    cp "${TRACE_DIR}/graph.json" "${DOCS_DIR}/graph.json"
fi

touch "${DOCS_DIR}/.nojekyll"

echo "docsify: copied graph outputs to docs/"
