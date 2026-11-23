# ChangeLog

# Release 0.2.0

## New

- `discovery`, finding the boards on the network by mDNS, as bruce-embedded-std 0.3.0's firmware advertises them,
  `_bruce-device._tcp` and `_http._tcp`, with each one's ID, board, firmware and its version, address, and whether it
  is adopted. It asks the boards to answer straight back, so that it works when they are on another network than the
  computer, e.g. an IoT VLAN. Apps use it too, e.g. Bruce's Smart Home, to find their devices without their addresses
  being typed in.
- `flash --erase-settings` erases the board's saved settings, its NVS partition, after the checks and before
  flashing, so that it starts as a new board, e.g. one which crashes too soon to use its own reset.

## Improved

- A board on USB which is also on the network is described, by `list` and in the window, by what it says there: its
  board, the firmware it runs now and its version, and its address, rather than what it ran when it was last
  identified. The window looks at the network every 15 seconds.
- Flashing checks the network too: a board which says there that it is another board than the firmware was built for
  is not flashed, unless `--confirm-board` says it is, and one whose firmware has no record, but which says there that
  it is the firmware's board, is flashed without asking which board it is.
- What is flashed to a board is remembered, so that it is no longer listed as running the firmware it had before.

## Fixed

- Flashing a firmware file from the shared firmware folder with `--keep-as` emptied it, as it was copied onto itself.

# Release 0.1.0

## New

- `bruce-embedded-flash`, the library and the command, flashes firmware built on bruce-embedded-std to a board only if
  it was built for that board, by the record each firmware image carries, read from the ELF file and from the board
  over its ROM bootloader. If the firmware on the board has no record, it asks which board it is; if it was built
  for another board, or the firmware is for another chip, it refuses; and it asks before replacing other firmware,
  e.g. a capture device's.
- `list`, `boards`, `restart`, `identify` and `flash` commands, the boards seen on each USB port remembered, and
  `flash --keep-as` to keep what was flashed in the shared firmware folder. It flashes with espflash's library, so it
  needs no other tools, and is the cargo runner of bruce-esp-idf's board crates.
- `bruce-embedded-firmware`, the shared folder firmware is kept in on a computer, by repository and version, and
  finding repositories with firmware on GitHub, listing their releases and downloading their ELF files, the
  `firmware` command.
- `bruce-embedded-flash-ui`, a Dioxus component for flashing boards on USB, which apps such as Bruce's Smart Home use,
  and a desktop app built with it.
