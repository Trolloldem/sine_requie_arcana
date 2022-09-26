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

fn get_others(states: &mut Vec<UserState>, decks: &Decks) {
    let keys = decks.get_players();
    for key in keys {
        let val = key.to_string();
        if val != states[0].name {
                states.push(UserState { last_drawn: decks.get_last_drawn(&val), name: val} );
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserState {
        name: String,
        last_drawn: Option<u8>,
}

#[get("/")]
fn index(user: User) -> Redirect {
       Redirect::to(format!("/?name={}", user.name)) 
}

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
