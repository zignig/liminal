use rocket::fairing::AdHoc;
use crate::{
    store::{FileSet, RenderType},
    templates::ReplicaTemplate, web::auth::User,
};
use rocket::response::Responder;

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("File Browser", |rocket| async {
        rocket.mount("/", routes![replicate])
    })
}

#[get("/replica")]
pub async fn replicate<'r>(_user: User) -> impl Responder<'r, 'static> {
    ReplicaTemplate {
        section: "replica".to_string()
    }
}