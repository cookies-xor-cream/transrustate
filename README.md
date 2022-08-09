# Transrustate
A simple terminal-based tool in order to help out with translating and conjugating verbs.

Makes a request to [wordreference](www.wordreference.com) and scrapes the results in order display the answer in the terminal. Inserts the results into a database in order to speed up repeat queries.

Uses [TUI](https://docs.rs/tui/latest/tui/) for the interface and [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/) (a rust interface to [sqlite](https://www.sqlite.org/index.html)) to manage the data.
