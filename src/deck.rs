use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::sync::Mutex;
use rand::seq::SliceRandom;
use rand::thread_rng;

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

pub struct Decks {
        pub decks: HashMap<String, Vec<u8>>,
        pub last_drawn: HashMap<String, Option<u8>>,
}

pub struct SharedDecks {
        pub data: Mutex<Decks>
}

impl SharedDecks {

        pub fn new() -> SharedDecks {
                SharedDecks { data: Mutex::new(Decks::new()) }
        }

}

impl Decks {

        pub fn new() -> Decks {
                Decks { decks: HashMap::new(), last_drawn: HashMap::new() }
        }

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

        pub fn has_player(&self, player_name: &String) -> bool {      
                self.decks.contains_key(player_name)
        }

        pub fn get_players(&self) -> Keys<String, Vec<u8>> {
                self.decks.keys()
        }

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

        pub fn get_last_drawn(&self, player_name: &String) -> Option<u8> {
                if self.last_drawn.contains_key(player_name) {
                        return *self.last_drawn.get(player_name).unwrap();
                }
                None
        }

        pub fn is_last_card(&self, player_name: &String) -> bool {
                if self.decks.contains_key(player_name) {
                        let decks: &HashMap<String, Vec<u8>> = &self.decks;
                        let deck = decks.get(player_name).unwrap();
                        return deck.len() == 0;
                }
                false
        }

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
