# Mephisto-emu
An experimental emulator for the Mephisto MM2 chess computer, providing a UCI compatible interface.
Maybe other boards in the Mephisto Modular series will be added later.

## Quickstart

`git clone https://github.com/BierLiebHaber/mephisto-emu`

`cd mephisto-emu`

Get the MM2 rom file (I used version 400, tho others should also work) from somewhere and rename it to `MM2.rom`.
Get the `hg240.rom` rom file from somewhere.
Put both into the `mephisto-emu` folder.

`cargo build -r`

`./start.sh`
