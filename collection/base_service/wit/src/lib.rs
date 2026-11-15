pub mod bindings {
    wit_bindgen::generate!({
        world: "service",
    });
}

use crate::bindings::export;
use crate::bindings::exports::liminal::controller::todo::Guest;

pub struct Test;

impl Guest for Test {
    async fn fetch(host: String) -> String {
        println!("{}", host);
        host
    }
}

export!(Test with_types_in bindings);
