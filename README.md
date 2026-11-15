[![Rust](https://github.com/not-matthias/fcg-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/not-matthias/fcg-rs/actions/workflows/rust.yml)

# fcg-rs
**F**lash**c**ards **G**enerator

## Updating decks

Each note's Anki ID is derived from the deck name (`cards-deck`) and the card front (the heading).
When you re-import a regenerated deck:

- Editing the text below a heading updates the existing card and keeps its review progress.
- Renaming a heading, one of its parent headings, or the deck creates a new card; the old one stays in Anki.
- Two cards with the same front in one deck share an ID, so Anki keeps only one; `fcg` logs a warning.
