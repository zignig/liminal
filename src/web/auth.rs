use crate::templates::LoginPageTemplate;
use rocket::response::Responder;

#[get("/login")]
pub fn login<'r>() -> impl Responder<'r, 'static> {
    LoginPageTemplate {
        section: "".to_string(),
    }
}
