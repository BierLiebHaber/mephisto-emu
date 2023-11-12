#!/bin/bash
cd "$(dirname "$0")"

tee in.log | target/release/mephisto-mm2-emu 2>&1 | tee uci.log
