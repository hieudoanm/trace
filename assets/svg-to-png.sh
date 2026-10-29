#!/usr/bin/env bash
#
# svg-to-png.sh — Convert SVG files to PNG.
#
# Usage:
#   ./svg-to-png.sh <input.svg> [output.png] [--width N] [--height N]
#   ./svg-to-png.sh <input.svg> [output_dir] --all
#   ./svg-to-png.sh --all [output_dir]
#
# Converts SVG files to PNG using rsvg-convert (preferred) or ImageMagick.
# If no output path is given, replaces the .svg extension with .png.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SVG_DIR="${SCRIPT_DIR}/svg"
IMAGES_DIR="${SCRIPT_DIR}/images"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

log() { printf '%s\n' "$*" >&2; }
die() { log "error: $*"; exit 1; }

# Detect the best available SVG->PNG converter.
detect_converter() {
    if command -v rsvg-convert >/dev/null 2>&1; then
        echo "rsvg"
    elif command -v magick >/dev/null 2>&1; then
        echo "magick"
    elif command -v convert >/dev/null 2>&1; then
        echo "convert"
    else
        die "no SVG-to-PNG converter found (install librsvg or ImageMagick)"
    fi
}

# Convert a single SVG file to PNG.
convert_svg() {
    local converter="$1"
    local input="$2"
    local output="$3"
    local width="${4:-}"
    local height="${5:-}"

    [[ -f "$input" ]] || die "input file not found: $input"

    case "$converter" in
        rsvg)
            local args=(-o "$output")
            [[ -n "$width" ]]  && args+=(-w "$width")
            [[ -n "$height" ]] && args+=(-h "$height")
            rsvg-convert "${args[@]}" "$input"
            ;;
        magick)
            local args=(-background none)
            [[ -n "$width" ]]  && args+=(-resize "${width}x")
            [[ -n "$height" ]] && args+=(-resize "x${height}")
            magick convert "$input" "${args[@]}" "$output"
            ;;
        convert)
            local args=(-background none)
            [[ -n "$width" ]]  && args+=(-resize "${width}x")
            [[ -n "$height" ]] && args+=(-resize "x${height}")
            convert "$input" "${args[@]}" "$output"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

CONVERTER="$(detect_converter)"
INPUT=""
OUTPUT=""
WIDTH=""
HEIGHT=""
ALL=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)
            ALL=true
            shift
            ;;
        --width)
            WIDTH="${2:-}"; shift 2 || die "--width requires a value"
            ;;
        --height)
            HEIGHT="${2:-}"; shift 2 || die "--height requires a value"
            ;;
        --help|-h)
            cat <<EOF
Usage: $0 <input.svg> [output.png] [--width N] [--height N]
       $0 <input.svg> <output_dir> --all
       $0 --all [output_dir]

Converts SVG files to PNG using rsvg-convert or ImageMagick.

Options:
  --width N    Output width in pixels (preserves aspect ratio if height omitted)
  --height N   Output height in pixels (preserves aspect ratio if width omitted)
  --all        Convert all .svg files in the svg/ directory
  --help       Show this help message
EOF
            exit 0
            ;;
        -*)
            die "unknown option: $1"
            ;;
        *)
            if [[ -z "$INPUT" ]]; then
                INPUT="$1"
            elif [[ -z "$OUTPUT" ]]; then
                OUTPUT="$1"
            else
                die "unexpected argument: $1"
            fi
            shift
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if [[ "$ALL" == true ]]; then
    mkdir -p "${OUTPUT:-$IMAGES_DIR}"
    OUT_DIR="${OUTPUT:-$IMAGES_DIR}"
    shopt -s nullglob
    for svg in "${SVG_DIR}"/*.svg; do
        [[ -f "$svg" ]] || continue
        base="$(basename "${svg%.svg}")"
        out="${OUT_DIR}/${base}.png"
        log "converting ${svg} -> ${out}"
        convert_svg "$CONVERTER" "$svg" "$out" "$WIDTH" "$HEIGHT"
    done
    log "done: converted SVGs in ${SVG_DIR} to ${OUT_DIR}"
    exit 0
fi

if [[ -z "$INPUT" ]]; then
    # Default behavior: convert all SVGs in the svg/ directory.
    mkdir -p "$IMAGES_DIR"
    shopt -s nullglob
    for svg in "${SVG_DIR}"/*.svg; do
        [[ -f "$svg" ]] || continue
        base="$(basename "${svg%.svg}")"
        out="${IMAGES_DIR}/${base}.png"
        log "converting ${svg} -> ${out}"
        convert_svg "$CONVERTER" "$svg" "$out" "$WIDTH" "$HEIGHT"
    done
    log "done: converted SVGs in ${SVG_DIR} to ${IMAGES_DIR}"
    exit 0
fi

if [[ -d "$INPUT" ]]; then
    # Treat INPUT as a directory of SVGs.
    mkdir -p "${OUTPUT:-$IMAGES_DIR}"
    OUT_DIR="${OUTPUT:-$IMAGES_DIR}"
    shopt -s nullglob
    for svg in "${INPUT}"/*.svg; do
        [[ -f "$svg" ]] || continue
        base="$(basename "${svg%.svg}")"
        out="${OUT_DIR}/${base}.png"
        log "converting ${svg} -> ${out}"
        convert_svg "$CONVERTER" "$svg" "$out" "$WIDTH" "$HEIGHT"
    done
    log "done: converted SVGs in ${INPUT} to ${OUT_DIR}"
    exit 0
fi

# Single file conversion.
if [[ -z "$OUTPUT" ]]; then
    OUTPUT="${INPUT%.svg}.png"
fi

# If OUTPUT is a directory, place the PNG inside it.
if [[ -d "$OUTPUT" ]]; then
    base="$(basename "${INPUT%.svg}")"
    OUTPUT="${OUTPUT}/${base}.png"
fi

mkdir -p "$(dirname "$OUTPUT")"
log "converting ${INPUT} -> ${OUTPUT}"
convert_svg "$CONVERTER" "$INPUT" "$OUTPUT" "$WIDTH" "$HEIGHT"
log "done"
