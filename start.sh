#!/bin/bash
cd "$(dirname "$0")"

exec target/release/mephisto-mm2-emu
