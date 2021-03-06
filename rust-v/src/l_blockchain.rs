// --- use ---
use std::collections::HashSet;
use std::sync::Mutex;

use reqwest;

use rocket::State;
use rocket_contrib::Json;

use serde::Serialize;

// --- global util fn ---
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

fn json_str<T: Serialize>(item: &T) -> String {
    use serde_json;

    serde_json::to_string(item).unwrap()
}

// --- struct ---
#[derive(Serialize, Deserialize)]
pub struct Transaction {
    amount: f64,
    recipient: String,
    sender: String,
}

#[derive(Serialize, Deserialize)]
struct Block {
    index: u32,
    previous_hash: String,
    proof: u64,
    timestamp: String,
    transactions: Vec<Transaction>,
}

#[derive(Deserialize)]
struct Chain {
    chain: Vec<Block>,
    length: u32,
}

#[derive(Deserialize)]
pub struct Nodes { nodes: Vec<String> }

pub struct Blockchain {
    chain: Chain,
    current_transactions: Vec<Transaction>,
    nodes: HashSet<String>,
}


// --- impl ---
impl Blockchain {
    pub fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Chain { chain: vec![], length: 0 },
            current_transactions: vec![],
            nodes: HashSet::new(),
        };

        blockchain.new_block(100, Some("1".to_owned()));
        blockchain
    }

    fn last_block(&self) -> &Block { return self.chain.chain.last().unwrap(); }

    fn new_transaction(&mut self, sender: String, recipient: String, amount: f64) -> u32 {
        self.current_transactions.push(
            Transaction {
                sender,
                recipient,
                amount,
            }
        );

        self.last_block().index as u32 + 1
    }

    fn new_block(&mut self, proof: u64, previous_hash: Option<String>) -> &Block {
        use std::mem::replace;

        let block = Block {
            index: self.chain.length + 1,
            timestamp: timestamp(),
            transactions: replace(&mut self.current_transactions, vec![]),
            proof,
            previous_hash: if let Some(val) = previous_hash { val } else {
                sha256(
                    json_str(self.last_block()
                    ).as_bytes()
                )
            },
        };

        self.chain.chain.push(block);
        self.chain.length += 1;

        self.last_block()
    }

    pub fn proof_of_work(&self, last_proof: u64) -> u64 {
        for proof in 0u64.. { if Blockchain::valid_proof(last_proof, proof) { return proof; } }

        unreachable!()
    }

    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        &sha256(format!(
            "{}{}",
            last_proof,
            proof
        ).as_bytes())[0..4] == "0000"
    }

    fn register_node(&mut self, node: String) {
        use url::Url;

        let url = Url::parse(&node).unwrap();
        let mut node = url.host_str().unwrap().to_owned();

        if let Some(port) = url.port() { node.push_str(&format!(":{}", port)) }

        self.nodes.insert(node);
    }

    fn valid_chain(&self, chain: &Vec<Block>) -> bool {
        let mut prev_block = &chain[0];

        for block in &chain[1..] {
            if block.previous_hash != sha256(json_str(prev_block).as_bytes()) { return false; }
            if !Blockchain::valid_proof(prev_block.proof, block.proof) { return false; }

            prev_block = block
        }

        true
    }

    fn resolve_conflicts(&mut self) -> bool {
        let nodes = &self.nodes;
        let client = reqwest::Client::new();
        let mut max_chain = None;
        let mut max_chain_len = self.chain.length;

        for node in nodes {
            if let Ok(mut resp) = client.get(&format!("http://{}/chain", node)).send() {
                if let Ok(Chain { chain, length }) = resp.json() {
                    if max_chain_len < length && self.valid_chain(&chain) {
                        max_chain = Some(chain);
                        max_chain_len = length;
                    }
                }
            }
        }

        if let Some(max_chain) = max_chain {
            self.chain.chain = max_chain;
            self.chain.length = max_chain_len;

            true
        } else { false }
    }
}


// --- rocket manager ---
type BcMgr = Mutex<Blockchain>;
type NodeIdentifier = Mutex<String>;

// --- rocket get route ---
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
        "chain": chain.chain,
        "length": chain.length
    }))
}

#[get("/nodes/resolve")]
pub fn consensus(bc_mgr: State<BcMgr>) -> Json {
    let response;
    let mut blockchain = bc_mgr.lock().unwrap();

    if blockchain.resolve_conflicts() {
        response = json!({
            "message": "Our chain was replaced",
            "new_chain": blockchain.chain.chain
        })
    } else {
        response = json!({
            "message": "Our chain is authoritative",
            "chain": blockchain.chain.chain
        })
    }

    Json(response)
}

// --- rocket post route ---
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
