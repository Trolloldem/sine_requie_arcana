//! Structs and functions used to handle the communication with clients

use rocket::http::Status;
use rocket::request::{FromRequest, Request, Outcome};
use rocket::serde::{Serialize, Deserialize};

/// Struct used as a Request Guard
pub struct User {
        pub name: String,
}

/// Request guard implementation
///
/// This function checks that the request associated with the functions 
/// [new_deck()][crate::new_deck()] and [index()][crate::index()] 
/// are issued by a client that is represented by one of the following cases:
/// 1. The client has no cookies. In this case, if the request does not provide a `name`, 
/// it is forwarded to the [Rocket Fileserver][rocket::fs::FileServer] to recover the application
/// landing page. If the request has a `name` in its query parameters, the check is successful and 
/// [new_deck()][crate::new_deck()] is executed
/// 2. The client has a valid cookie. In this case, if the request does not provide a `name`, the 
/// [index()][crate::index()] is responsible of forwarding the request to
/// [new_deck()][crate::new_deck()]. If the request has a `name` in its query parameters, 
/// it is checked that it matches the name stored in the signed cookie provided. If the check is 
/// successful, the [new_deck()][crate::new_deck()] function is executed, otherwise 
/// a `BadRequest` error is returned to the user.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, String> {
        if request.cookies().get_private("name").is_none() {
                if request.query_value::<&str>("name").is_none() {
                        return Outcome::Forward(Status::BadRequest)
                } else {
                        return Outcome::Success(User { name: request.query_value::<&str>("name").unwrap().unwrap().to_string() })
                }
        }

        if request.query_value::<&str>("name").is_some() {
                let name =  request.query_value::<&str>("name").unwrap().unwrap();
                if name != request.cookies().get_private("name").unwrap().value() {
                        return Outcome::Error((Status::BadRequest, "You cannot login with multiple usernames".to_string()));
                }
        }
        Outcome::Success(User { name: request.cookies().get_private("name").unwrap().value().to_string() })
    }
}

/// Struct used to send messages through the `EventStream`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub name: String,
    pub arcana: Option<u8>,
    pub is_shuffle: bool,
    pub is_last_card: bool,
}

/// Struct used as a JSON response when a card is drawn
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DeckResponse {
     pub arcana: Option<u8>,
     pub error: Option<String>,
     pub is_last_card: bool,
}

