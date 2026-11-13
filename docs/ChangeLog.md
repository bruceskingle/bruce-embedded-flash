# ChangeLog

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
