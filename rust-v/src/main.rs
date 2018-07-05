extern crate rustc_serialize;

//extern crate serde;
//#[macro_use]
//extern crate serde_derive;
//extern crate serde_json;

extern crate sha2;

mod l_blockchain;

fn main() {
    let blockchain = l_blockchain::Blockchain::new();
}
