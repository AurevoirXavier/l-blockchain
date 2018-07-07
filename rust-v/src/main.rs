#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

// ---

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

extern crate rustc_serialize;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate sha2;

extern crate uuid;

// ---

mod l_blockchain;

// ---

use std::sync::Mutex;

use uuid::Uuid;

fn main() {
    rocket::ignite()
        .mount("/", routes![
            l_blockchain::new_transaction,
            l_blockchain::mine,
            l_blockchain::full_chain
        ])
        .manage(Mutex::new(l_blockchain::Blockchain::new()))
        .manage(Mutex::new(Uuid::new_v4().to_string().replace("-", "")))
        .launch();
}
