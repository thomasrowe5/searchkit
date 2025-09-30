#!/usr/bin/env bash
set -euo pipefail
BIN=target/release/searchkit
CORPUS=${1:-data/sample/wiki.txt}
OUT=${2:-data/index.inv}
time $BIN build-inv $CORPUS $OUT
