use rocket::http::Status;
use rocket::request::{FromRequest, Request, Outcome};
use rocket::serde::{Serialize, Deserialize};


pub struct User {
        pub name: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> Outcome<User, String> {
        if request.cookies().get_private("name").is_none() {
                if request.query_value::<&str>("name").is_none() {
                        return Outcome::Forward(())
                } else {
                        return Outcome::Success(User { name: request.query_value::<&str>("name").unwrap().unwrap().to_string() })
                }
        }

        if request.query_value::<&str>("name").is_some() {
                let name =  request.query_value::<&str>("name").unwrap().unwrap();
                if name != request.cookies().get_private("name").unwrap().value() {
                        return Outcome::Failure((Status::BadRequest, "You cannot login with multiple usernames".to_string()));
                }
        }
        Outcome::Success(User { name: request.cookies().get_private("name").unwrap().value().to_string() })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Message {
    pub name: String,
    pub arcana: Option<u8>,
    pub is_shuffle: bool,
    pub is_last_card: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DeckResponse {
     pub arcana: Option<u8>,
     pub error: Option<String>,
     pub is_last_card: bool,
}

