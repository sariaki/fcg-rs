use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use crate::error::FcgError;
use crate::{card::Card, parser::Parser};
use anki::{Field, Model, Note, Template};
use genanki_rs as anki;

pub struct Deck {
    pub name: String,
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new(file: &str) -> Result<Self, FcgError> {
        let mut parser = Parser::new(file.into());

        let header = parser.parse_yaml().ok_or(FcgError::ParseYaml)?;
        let graph = parser.parse_markdown();

        let cards = graph
            .node_indices()
            .map(|index| Card::new(&graph, index))
            .collect();

        Ok(Self {
            name: header.cards_deck,
            cards,
        })
    }

    pub fn combine(&mut self, other: Deck) {
        self.cards.extend(other.cards);
    }
}

impl Deck {
    fn str_to_id(string: &str) -> i64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        (hasher.finish() % u32::MAX as u64) as i64
    }

    /// Note GUID used by Anki to match re-imported notes with existing ones (and their review
    /// progress). It only depends on the deck name and the front, so editing the back of a card
    /// updates the existing note instead of creating a new one.
    ///
    /// Uses FNV-1a instead of `DefaultHasher`, whose output may change between Rust releases and
    /// would silently reset the progress of every card.
    fn note_guid(deck_name: &str, front: &str) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in deck_name.bytes().chain([0]).chain(front.bytes()) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("fcg-{hash:016x}")
    }

    fn card_model(card: &Card) -> Model {
        let template = Template::new("Card")
            .qfmt("{{Front}}")
            .afmt("{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}");

        Model::new(
            Self::str_to_id(&card.front),
            "Model",
            vec![Field::new("Front"), Field::new("Back")],
            vec![template],
        )
        .css(include_str!("card.css"))
    }

    /// Saves the current deck to disk.
    pub fn save(self, file_path: &str) {
        let mut deck = anki::Deck::new(
            Self::str_to_id(&self.name),
            &self.name,
            "No description available",
        );

        // Add all the notes
        //
        let mut guids = HashSet::new();
        for card in &self.cards {
            let guid = Self::note_guid(&self.name, &card.front);
            if !guids.insert(guid.clone()) {
                log::warn!(
                    "Duplicate card front {:?} in deck {:?}: Anki will only keep one of them",
                    card.front,
                    self.name
                );
            }

            let model = Self::card_model(card);
            let note = Note::new(model, vec![&card.front, &card.back])
                .unwrap()
                .guid(guid);
            deck.add_note(note);
        }

        log::info!("Saving deck to {}", file_path);
        deck.write_to_file(file_path).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_guid_is_stable() {
        // Pinned value: changing it resets the review progress of every imported card.
        assert_eq!(
            Deck::note_guid("deck", "Cell > What does the mitochondria do?"),
            "fcg-2c495b3bf942af89"
        );
        assert_ne!(Deck::note_guid("a", "front"), Deck::note_guid("b", "front"));
        assert_ne!(Deck::note_guid("ab", "c"), Deck::note_guid("a", "bc"));
    }
}
