#[macro_use] 
extern crate rocket;
use rocket::fs::{FileServer, Options};
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::response::stream::{EventStream, Event};
use rocket::{State, Shutdown};
use rocket_dyn_templates::{Template, context};
use rocket::serde::json::Json;
use rocket::tokio::sync::broadcast::{channel, Sender, error::RecvError};
use rocket::tokio::select;
use rocket::serde::{Serialize, Deserialize};


mod deck;
use deck::{Decks, SharedDecks};

mod comms;
use comms::{DeckResponse, Message, User};

/// Internal function to the player names
///
/// This function returns the [User States][UserState] of all the players.
/// The `states` vector must already contain a `UserState` associated with the
/// player that has requested the endpoint associated with the [new_deck()] function
fn get_others(states: &mut Vec<UserState>, decks: &Decks) {
    let keys = decks.get_players();
    for key in keys {
        let val = key.to_string();
        if val != states[0].name {
                states.push(UserState { last_drawn: decks.get_last_drawn(&val), name: val} );
        }
    }
}

/// Struct to store Deck data used in the Handlebars template
///
/// This struct is used to store the data when the Handlebars template
/// rendered by [new_deck()]. It also handles the serialization and deserialization process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserState {
        name: String,
        last_drawn: Option<u8>,
}

/// Endpoint of the application landing page
///
/// Its behavior is determined by the checks perfomed through the [User][comms::User] request guard.
/// If the request is forwarded, the [Rocket fileserver][rocket::fs::FileServer]. 
/// Otherwise, the request is redirected and handled by [new_deck()]
#[get("/")]
fn index(user: User) -> Redirect {
       Redirect::to(format!("/?name={}", user.name)) 
}

/// Endpoint used to subscribe a client to the `EventStream`
///
/// After a client is subscribed to this `EventStream`, it can send
/// [Messages][comms::Message] to the server that will be broadcasted
/// to the other clients subscribed
#[get("/subscribe")]
fn subscribe(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

/// Main endpoint of the application
///
/// Its behavior is determined by the checks perfomed through the [User][comms::User] request guard.
/// If the guard cheks are successful the GET request to this method, the endpoint will execute one of the two following paths:
/// 1. The player has no deck assigned: a new one is created and all the other players are notified
/// 2. The player already has a deck, so it is simply recovered alongside the last drawn card
/// After the previous step, the last cards drawn by every player is recovered and an `Handlebars` template is 
/// rendered as a response to the request
#[get("/?<name>")]
fn new_deck(_user: User, name: String, shared_decks: &State<SharedDecks>, cookies: &CookieJar, queue: &State<Sender<Message>>) -> Template {
    let mut states;
    let decks = &mut *shared_decks.data.lock().expect("Cannot acquire lock on decks");
    if cookies.get_private("name").is_none() || !decks.has_player(&name) {
        cookies.add_private(Cookie::new("name", name.clone()));
        decks.insert_deck(&name);
        states = vec![UserState { last_drawn: decks.get_last_drawn(&name), name: name} ];
        get_others(&mut states, decks);
        let _res = queue.send(Message { name: states[0].name.clone(), arcana: None, is_shuffle: false, is_last_card: false });
    } else {
        states = vec![UserState{ last_drawn: decks.get_last_drawn(&name), name: name} ];
        get_others(&mut states, decks);
    }
    Template::render("cards", context! { states: &states })
}

/// Endpoint used to draw a card from a player's deck
///
/// If the player that is sending a GET request has a valid cookie, a card is drawn from its [deck][deck::Decks].
/// The server sends to the client a JSON with the structure of a [Deck Response][comms::DeckResponse].
/// There are 3 possible cases: 
/// 1. The card is drawn successfully and its number is sent to the player. If this is the last card, the client is notified
/// 2. The deck is empty. This event is communicated to the client as an error
/// 3. The player does not have a valid cookie. In this case the server assumes that no deck has been created. This event is communicated to the client as an error
#[get("/get")]
fn get_card(shared_decks: &State<SharedDecks>, cookies: &CookieJar, queue: &State<Sender<Message>>) -> Json<DeckResponse>{
    match cookies.get_private("name") {
        Some(cookie) => {
            let decks = &mut *shared_decks.data.lock().expect("Cannot acquire lock on decks");
            return match decks.get_card(&String::from(cookie.value())) {
                Some(card) => {
                        let last_card: bool = decks.is_last_card(&String::from(cookie.value()));
                        let _res = queue.send(Message { name: cookie.value().to_string(), arcana: Some(card.clone()), is_shuffle: false, is_last_card: last_card.clone() }); 
                        Json(DeckResponse { arcana: Some(card), error: None, is_last_card: last_card })
                },
                None => Json(DeckResponse { arcana: None, error: Some(String::from("Deck is empty")), is_last_card: false }),
            };
        },
        None => Json(DeckResponse { arcana: None, error: Some(String::from("Please create a deck first")), is_last_card: false }),
    }
}

/// Endpoint used to shuffle a player's deck
///
/// If the player that is sending a GET request has a valid cookie, its [deck][deck::Decks] is shuffled.
/// On a success, the server sends a message to the client.
#[get("/shuffle")]
fn shuffle_deck(shared_decks: &State<SharedDecks>, cookies: &CookieJar, queue: &State<Sender<Message>>) {
    match cookies.get_private("name") {
        Some(cookie) => {
                let name  = String::from(cookie.value());
                let decks = &mut *shared_decks.data.lock().expect("Cannot acquire lock on decks");
                decks.shuffle_deck(&name);
                let _res = queue.send(Message { name: cookie.value().to_string(), arcana: None, is_shuffle: true, is_last_card: false });
        },
        None => (),
    }
}

/// Application entry point
///
/// The Rocket `launch` macro is responsible of running the function
/// at application startup.This function is responsible of: 
/// 1. Recovering the files necessary to the front-end
/// 2. Expose the applications routes
/// 3. Expose the static files of the application 
#[launch]
fn rocket() -> _ {
    let fe_path = format!("{}/arcana-frontend/static", env!("CARGO_MANIFEST_DIR"));
    let index_path = format!("{}/content/index.html", fe_path);
    rocket::build()
        .manage(SharedDecks::new())
        .manage(channel::<Message>(1024).0)
        .mount("/", routes![
                                index, 
                                new_deck, 
                                shuffle_deck, 
                                get_card, 
                                subscribe
                           ])
        .mount("/", FileServer::new(index_path, Options::IndexFile))
        .mount("/", FileServer::from(fe_path).rank(11))
        .attach(Template::fairing())
}
