use std::fmt;
use std::time::SystemTime;

//use serde_json;

//#[derive(Serialize, Deserialize)]
struct Sender {}

//#[derive(Serialize, Deserialize)]
struct Recipient {}

//#[derive(Serialize, Deserialize)]
struct Transaction {
    sender: Sender,
    recipient: Recipient,
    amount: f64,
}

//#[derive(Serialize, Deserialize)]
struct Block {
    index: u32,
    timestamp: SystemTime,
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
            timestamp: SystemTime::now(),
            transactions: replace(&mut blockchain.current_transactions, vec![]),
            proof,
            previous_hash: if let Some(val) = previous_hash { val } else {
                Block::hash(blockchain.last_block())
            },
        };

        blockchain.chain.push(block);
        blockchain.last_block()
    }

    fn hash(block: &Block) -> String {
        use sha2::{Sha256, Digest};
        use rustc_serialize::hex::ToHex;

        let mut hasher = Sha256::default();
        hasher.input(
            block.to_string().as_bytes()
        );

        hasher.result().as_slice().to_hex()
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Block {
            index,
            timestamp,
            transactions,
            proof,
            previous_hash
        } = self;

        writeln!(
            f,
            r#"{{"index": {}, "previous_hash": {}, "proof": {}, "timestamp": {}, "transactions": {}}}"#,
            index,
            previous_hash,
            proof,
            1,
            1
        )
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
