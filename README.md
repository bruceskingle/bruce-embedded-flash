# bruce-embedded-flash

Bruce's Embedded Flash: flashing firmware to ESP32 boards, built on
[bruce-esp-idf](https://github.com/bruceskingle/bruce-esp-idf), only to the board it was built for, by the record each
firmware image carries, [bruce-embedded-firmware-record](https://github.com/bruceskingle/bruce-embedded-firmware-record).

| Crate | What it is |
|---|---|
| [bruce-embedded-flash](bruce-embedded-flash) | The library, and the `bruce-embedded-flash` command, which flashes a board, after checking the firmware on it, see its [README](bruce-embedded-flash/README.md). |
| [bruce-embedded-firmware](bruce-embedded-firmware) | The firmware on a computer: the shared folder it is kept in, by repository and version, and downloading it from GitHub releases. |
| [bruce-embedded-flash-ui](bruce-embedded-flash-ui) | A Dioxus component for flashing boards, which apps use, e.g. Bruce's Smart Home's Devices page, and a desktop app built with it, `cargo run --features desktop` in its directory. |

The crates are released together, with the same version.

License
=======

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
