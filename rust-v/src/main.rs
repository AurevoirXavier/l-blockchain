#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rustc_serialize;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate sha2;

mod l_blockchain;

fn main() {
    use l_blockchain;
    let mut blockchain = l_blockchain::Blockchain::new();

    rocket::ignite().mount("/", routes![
       l_blockchain::mine
    ]).launch();
}
