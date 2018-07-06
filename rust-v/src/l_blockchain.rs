use serde_json;

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

#[derive(Serialize, Deserialize)]
struct Sender {}

#[derive(Serialize, Deserialize)]
struct Recipient {}

#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: Sender,
    recipient: Recipient,
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

pub struct Blockchain {
    chain: Vec<Block>,
    current_transactions: Vec<Transaction>,
}

impl Block {
    fn new(blockchain: &mut Blockchain, proof: u32, previous_hash: Option<String>) -> &Block {
        use std::mem::replace;



        let block = Block {
            index: blockchain.chain.len() as u32 + 1,
            timestamp: timestamp(),
            transactions: replace(&mut blockchain.current_transactions, vec![]),
            proof,
            previous_hash: if let Some(val) = previous_hash { val } else {
                sha256(serde_json::to_string(
                    blockchain.last_block()
                ).unwrap().as_bytes())
            },
        };

        blockchain.chain.push(block);
        blockchain.last_block()
    }
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain: vec![],
            current_transactions: vec![],
        }
    }

    fn last_block(&self) -> &Block { return self.chain.last().unwrap(); }

    pub fn proof_of_work(&self, last_proof: u32) -> u32 {
        for proof in 0u32.. {
            if Blockchain::valid_proof(last_proof, proof) { println!("{}", proof);return proof; }
        }

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

impl Transaction {
    fn new(blockchain: &mut Blockchain, sender: Sender, recipient: Recipient, amount: f64) -> u32 {
        blockchain.current_transactions.push(Transaction {
            sender,
            recipient,
            amount,
        });

        blockchain.last_block().index as u32 + 1
    }
}

#[get("/mine")]
pub fn mine() -> &'static str {
    "We'll mine a block"
}
