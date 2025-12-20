use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
    response::Responder,
};

use crate::templates::{HomePageTemplate, SearchFragmentTemplate};

#[get("/search?<query>", rank = 1)]
pub async fn searcher<'r>(query: String, up: UpTarget<'_>) -> impl Responder<'r, 'static> {
    println!("{} --  {:?} ", query, up.0);
    SearchFragmentTemplate {
        items: vec![query, "fnord".to_string(), "three".to_string()],
    }
}

#[get("/search", rank = 2)]
pub async fn base_search<'r>() -> impl Responder<'r, 'static> {
    HomePageTemplate {
        section: "".to_string(),
    }
}

#[get("/search?<query>", rank = 3)]
pub async fn search_page<'r>(query: String) -> impl Responder<'r, 'static> {
    println!("{} - ", query);
    HomePageTemplate {
        section: "".to_string(),
    }
}

#[derive(Debug)]
pub struct UpTarget<'r>(&'r str);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UpTarget<'r> {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // println!("{:#?}", req.headers());
        match req.headers().get_one("X-Up-Target") {
            Some(val) => Outcome::Success(UpTarget(val)),
            None => Outcome::Forward(Status::Accepted), // None => Outcome::Error((Status::BadRequest, ())),
        }
    }
}
