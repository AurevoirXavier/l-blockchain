// use
use std::collections::HashSet;
use std::sync::Mutex;

use reqwest;

use rocket::State;
use rocket_contrib::Json;

use serde_json;


// util fn
fn sha256(input: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    use rustc_serialize::hex::ToHex;

    let mut hasher = Sha256::default();
    hasher.input(input);

    hasher.result().as_slice().to_hex()
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    format!("{}.{}", timestamp.as_secs(), timestamp.subsec_micros())
}


// struct
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    sender: String,
    recipient: String,
    amount: f64,
}

#[derive(Serialize, Deserialize)]
struct Block {
    index: u32,
    timestamp: String,
    transactions: Vec<Transaction>,
    proof: u32,
    previous_hash: String,
}

#[derive(Deserialize)]
pub struct Nodes {
    nodes: Vec<String>
}

#[derive(Deserialize)]
struct Chain {
    chain: Vec<Block>,
    length: u32,
}

pub struct Blockchain {
    chain: Vec<Block>,
    current_transactions: Vec<Transaction>,
    nodes: HashSet<String>,
}


// impl
impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            current_transactions: vec![],
            nodes: HashSet::new(),
        };

        blockchain.new_block(100, Some("1".to_owned()));
        blockchain
    }

    fn register_node(&mut self, node: String) {
        use url::Url;

        let url = Url::parse(&node).unwrap();
        let mut node = url.host_str().unwrap().to_owned();

        if let Some(port) = url.port() { node.push_str(&format!(":{}", port)) }

        self.nodes.insert(node);
    }

    fn valid_chain(&self, chain: &Vec<Block>) -> bool {
        let mut last_block = &chain[0];
        let mut current_index = 1;

        while current_index < chain.len() {
            let block = &chain[current_index];

            if block.previous_hash != sha256(serde_json::to_string(last_block).unwrap().as_bytes()) { return false; }
            if !Blockchain::valid_proof(last_block.proof, block.proof) { return false; }

            last_block = block;
            current_index += 1;
        }

        true
    }

    fn resolve_conflicts(&mut self) -> bool {
        let nodes = &self.nodes;
        let client = reqwest::Client::new();
        let mut right_chain = None;
        let mut chain_len = self.chain.len() as u32;

        for node in nodes {
            if let Ok(mut resp) = client.get(&format!("http://{}/chain", node)).send() {
                if let Ok(Chain { chain, length }) = resp.json() {
                    if chain_len < length && self.valid_chain(&chain) {
                        right_chain = Some(chain);
                        chain_len = length;
                    }
                }
            }
        }

        if let Some(chain) = right_chain {
            self.chain = chain;

            true
        } else { false }
    }

    fn new_block(&mut self, proof: u32, previous_hash: Option<String>) -> &Block {
        use std::mem::replace;

        let block = Block {
            index: self.chain.len() as u32 + 1,
            timestamp: timestamp(),
            transactions: replace(&mut self.current_transactions, vec![]),
            proof,
            previous_hash: if let Some(val) = previous_hash { val } else {
                sha256(serde_json::to_string(self.last_block()).unwrap().as_bytes())
            },
        };

        self.chain.push(block);
        self.last_block()
    }

    fn new_transaction(&mut self, sender: String, recipient: String, amount: f64) -> u32 {
        self.current_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });

        self.last_block().index as u32 + 1
    }

    fn last_block(&self) -> &Block { return self.chain.last().unwrap(); }

    pub fn proof_of_work(&self, last_proof: u32) -> u32 {
        for proof in 0u32.. { if Blockchain::valid_proof(last_proof, proof) { return proof; } }

        unreachable!()
    }

    fn valid_proof(last_proof: u32, proof: u32) -> bool {
        &sha256(format!(
            "{}{}",
            last_proof,
            proof
        ).as_bytes())[0..4] == "0000"
    }
}


// rocket manager
type BcMgr = Mutex<Blockchain>;
type NodeIdentifier = Mutex<String>;

// rocket route
#[post("/transactions/new", format = "application/json", data = "<transaction>")]
pub fn new_transaction(transaction: Json<Transaction>, bc_mgr: State<BcMgr>) -> Json {
    let Transaction {
        sender,
        recipient,
        amount
    } = transaction.0;

    Json(json!({
        "message": format!(
           "Transaction will be added to Block {}",
            bc_mgr
                .lock()
                .unwrap()
                .new_transaction(sender, recipient, amount)
        )
    }))
}

#[get("/mine")]
pub fn mine(bc_mgr: State<BcMgr>, node_identifier: State<NodeIdentifier>) -> Json {
    let mut blockchain = bc_mgr.lock().unwrap();
    let node_identifier = node_identifier.lock().unwrap().to_owned();
    let proof = blockchain.proof_of_work(blockchain.last_block().proof);

    blockchain.new_transaction("0".to_owned(), node_identifier, 1.);

    let block = blockchain.new_block(proof, None);

    Json(json!({
        "message": "New Block forged",
        "index": block.index,
        "transactions": block.transactions,
        "proof": block.proof,
        "previous_hash": block.previous_hash
    }))
}

#[get("/chain")]
pub fn full_chain(bc_mgr: State<BcMgr>) -> Json {
    let chain = &bc_mgr.lock().unwrap().chain;

    Json(json!({
        "chain": chain,
        "length": chain.len()
    }))
}

#[post("/nodes/register", format = "application/json", data = "<nodes>")]
pub fn register_nodes(nodes: Json<Nodes>, bc_mgr: State<BcMgr>) -> Json {
    let Nodes { nodes } = nodes.0;
    let mut blockchain = bc_mgr.lock().unwrap();

    for node in nodes { blockchain.register_node(node); }

    Json(json!({
        "message": "Nodes have been added",
        "total_nodes": blockchain.nodes
    }))
}
