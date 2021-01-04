# Quota Control

*On hiatus until further notice*

Quota control is a cli gui tool. The tool is in active development at the moment and only features a simple overview of groups. Progress for this application is [public](https://github.com/Chaostheorie/quota-control/projects/1?fullscreen=true).

> There's only support for \*nix systems

You can build the full tool with `cargo build --release`

## Testing

You will require to have a folder in `/home/quotas/`. You may also need a file such as the supplied `/assets/beispiel.quota` (need to be in `/home/quotas/`). You may also require to bei either part of the group root or of a group matching the regular expression `[bghz]z.*`.

## Demo

![Screenshot](/assets/screenshot.png)

## Attributions

- [csv](https://crates.io/crates/csv) (1.1.3) by [Andrew Gallant](jamslam@gmail.com) under Unlicense/MIT
- [ansi_term](https://crates.io/crates/ansi_term) (0.12.1) by [Josh Triplett](josh@joshtriplett.org), ogham@bsago.me and [Ryan Scheel (Havvy)](ryan.havvy@gmail.com) under MIT
- [serde](https://crates.io/crates/serde) (1.0) by [David Tolnay](dtolnay@gmail.com) and [Erick Tryzelaar](erick.tryzelaar@gmail.com) under MIT OR Apache-2.0
- [regex](https://crates.io/crates/regex) (1.3.9) by the Rust Project Developers under MIT OR Apache-2.0
- [users](https://crates.io/crates/users) (0.10.0) by [Benjamin Sago](ogham@bsago.me) under MIT
- [tui](https://crates.io/crates/tui) (0.11.0) by [Florian Dehau](work@fdehau.com) under MIT
- [termion](https://gitlab.redox-os.org/redox-os/termion) (1.5.5) by [Ticki](Ticki@users.noreply.github.com) under MIT
