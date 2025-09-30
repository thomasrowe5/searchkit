#!/usr/bin/env bash
set -euo pipefail
BIN=target/release/searchkit
INDEX=${1:-data/index.inv}
Q=${2:-"hello world"}
K=${3:-10}
time $BIN query-inv $INDEX "$Q" $K
