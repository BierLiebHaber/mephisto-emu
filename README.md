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

## Known Bugs/Limitations
* using the engine with a fixed movetime needs quite a big margin (> 1sec) since we need to extract the info for the best current move
* all other go modifiers except movetime are unsupported
* while we do return a ponder move there is no way to actually use this information in the engine
* the first move in a new game (or movestack) can not be canceled by movetime or the stop command