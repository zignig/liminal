use rocket::fairing::AdHoc;



pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Web interface", |rocket| async {
        rocket.mount(
            "/",
            routes![
            ],
        )
    })
}

