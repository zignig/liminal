use crate::templates::LoginPageTemplate;

use rocket::{
    Request,
    form::Form,
    http::{CookieJar, Status},
    request::{FromRequest, Outcome},
    response::{Redirect, Responder},
};

#[get("/login")]
pub fn login<'r>() -> impl Responder<'r, 'static> {
    LoginPageTemplate {
        section: "".to_string(),
    }
}

// The node form data
#[derive(FromForm, Debug)]
pub struct LoginForm<'v> {
    user: &'v str,
    pass: &'v str,
}

// TODO fix to get user data out of the config
#[post("/login", data = "<login_data>")]
pub async fn login_post<'r>(
    login_data: Form<LoginForm<'_>>,
    cookies: &CookieJar<'_>,
) -> impl Responder<'r, 'static> {
    println!("{:#?}", &login_data.user);
    let _ = &login_data.pass;
    cookies.add_private(("user_id", "overlord"));
    Redirect::to(uri!(crate::web::index()))
}

pub struct User {
    pub id: u64,
}

// TODO fix
// from https://api.rocket.rs/v0.5/rocket/request/trait.FromRequest
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<User, ()> {
        let val = request.cookies().get_private("user_id");
        match val {
            Some(_) => Outcome::Success(User { id: 0 }),
            None => Outcome::Forward(Status::Unauthorized),
        }
    }
}

#[catch(401)]
pub fn unauthorized(_req:  &Request) -> Redirect { 
    Redirect::to(uri!(login))
}