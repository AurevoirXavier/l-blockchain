// marco
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]


// crate
extern crate reqwest;

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

extern crate rustc_serialize;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate sha2;

extern crate url;

extern crate uuid;


// mod
mod l_blockchain;


// use
use std::sync::Mutex;

use uuid::Uuid;


// main
fn main() {
    rocket::ignite()
        .mount("/", routes![
            l_blockchain::new_transaction,
            l_blockchain::mine,
            l_blockchain::full_chain,
            l_blockchain::register_nodes
        ])
        .manage(Mutex::new(l_blockchain::Blockchain::new()))
        .manage(Mutex::new(Uuid::new_v4().to_string().replace("-", "")))
        .launch();
}
