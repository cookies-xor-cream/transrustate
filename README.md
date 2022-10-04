# Transrustate
A terminal-based utility to help with translating and conjugating verbs.

Makes a request to [wordreference](www.wordreference.com) and displays the results in the terminal. Caches all results for future use at `~/.lang_rs.db`.

Built on top of [TUI](https://docs.rs/tui/latest/tui/) and [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/) (a rust interface to [sqlite](https://www.sqlite.org/index.html)).

# Installation
First build the project:
```bash
cargo build --bin transrustate --release
```

Once built the binary is located at `./target/release/transrustate` and can be copied into path.

Once in path the program can be run by typing `transrustate` into the terminal.

# Clearing The Cache
I plan to add a command to wipe the cache in the future, however, currently it must be done manually:
```bash
rm ~/.lang_rs.db
```
