//! Struct and methods used to handle players' decks

use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::sync::Mutex;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Enum that represents the status of a player's deck after an operation
pub enum DeckStatus {
        PlayerExists,
        InsertOk
}

impl std::fmt::Display for DeckStatus {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                        DeckStatus::PlayerExists => write!(f, "Player already exists"),
                        DeckStatus::InsertOk => write!(f, "New deck created"),
                }
        }
}

/// Type of the data used to store the association between players and card decks
pub struct Decks {
        pub decks: HashMap<String, Vec<u8>>,
        pub last_drawn: HashMap<String, Option<u8>>,
}

/// Thread-safe wrapper around [Decks]
///
/// Rocket state must be thread-safe to be passed as arguments to routes functions 
pub struct SharedDecks {
        pub data: Mutex<Decks>
}

impl SharedDecks {

        pub fn new() -> SharedDecks {
                SharedDecks { data: Mutex::new(Decks::new()) }
        }

}

impl Decks {

        /// Creates a new empty set of card decks
        pub fn new() -> Decks {
                Decks { decks: HashMap::new(), last_drawn: HashMap::new() }
        }

        /// Shuffle the the deck associated with a player
        ///
        /// This method check that the player with name `player_name` has a deck.
        /// If a deck is found, the last card drawn is set to `None` and the 
        /// deck is shuffled
        pub fn shuffle_deck(&mut self, player_name: &String) {
                let decks: &mut HashMap<String, Vec<u8>> = &mut self.decks;
                let deck_opt = decks.get_mut(player_name);
                match deck_opt {
                   Some(deck) => { 
                        let last_drawn = self.last_drawn.get_mut(player_name).unwrap();
                        *last_drawn = None;
                        deck.clear();
                        for tarot in 0..22 {
                                deck.push(tarot);
                        }
                        deck.shuffle(&mut thread_rng());
                   },
                   None => (),
                };
        }

        /// Check if a player is associated with a deck
        ///
        /// This method checks if the player with name `player_name` has a deck
        pub fn has_player(&self, player_name: &String) -> bool {      
                self.decks.contains_key(player_name)
        }

        /// Retrieve the names of all the player with a deck
        pub fn get_players(&self) -> Keys<String, Vec<u8>> {
                self.decks.keys()
        }

        /// Draw a card from a player's deck
        ///
        /// This method check that the player with name `player_name` has a deck.
        /// If a deck is found, a card is drawn and the last card drawn by the 
        /// player is updated.
        /// This method return `None` if the player's deck is empty, while `Some`
        /// with the card values inside otherwise 
        pub fn get_card(&mut self, player_name: &String) -> Option<u8> {
                if self.decks.contains_key(player_name) {
                        let decks: &mut HashMap<String, Vec<u8>> = &mut self.decks;
                        let deck = decks.get_mut(player_name).unwrap();
                        let last_drawn = self.last_drawn.get_mut(player_name).unwrap();
                        *last_drawn = deck.pop();
                        return *last_drawn;
                }
                None
        }

        /// Recover the last card drawn from a player's deck
        ///
        /// This method return `None` if the player's deck has never drawn a card before, 
        ///while `Some` with the card values inside otherwise 
        pub fn get_last_drawn(&self, player_name: &String) -> Option<u8> {
                if self.last_drawn.contains_key(player_name) {
                        return *self.last_drawn.get(player_name).unwrap();
                }
                None
        }

        /// Check if the deck is empty
        pub fn is_last_card(&self, player_name: &String) -> bool {
                if self.decks.contains_key(player_name) {
                        let decks: &HashMap<String, Vec<u8>> = &self.decks;
                        let deck = decks.get(player_name).unwrap();
                        return deck.len() == 0;
                }
                false
        }

        /// Inser a new deck with a player with `player_name`
        ///
        /// If the Decks
        pub fn insert_deck(&mut self, player_name: &String) -> DeckStatus {
                if !self.decks.contains_key(player_name) {
                        self.decks.insert(player_name.to_string(), Vec::<u8>::new());
                        self.last_drawn.insert(player_name.to_string(), None);
                        self.shuffle_deck(player_name);
                        return DeckStatus::InsertOk;
                }
                DeckStatus::PlayerExists
        }
}
