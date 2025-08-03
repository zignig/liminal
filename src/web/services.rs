//! This _will_ hold the external services
//! Blender, render GLTF and icons
//! Perhaps ....
//! Freecad , Kicad  ... remains to be seen.

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

