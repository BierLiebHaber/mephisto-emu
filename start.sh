#!/bin/bash
cd "$(dirname "$0")"
export RUST_BACKTRACE=full
target/release/mephisto-mm2-emu 2>&1 | tee uci.log
