use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use ring::digest::{digest, SHA256};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    sender: String,
    receiver: String,
    amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u32,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to obtain a timestamp.")
            .as_secs();

        let serialized_transactions = serde_json::to_string(&transactions).unwrap();
        let data = format!("{}{}{}", index, timestamp, serialized_transactions);
        let hash = Block::calculate_hash(&data);

        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
        }
    }

    fn calculate_hash(data: &str) -> String {
        let mut context = SHA256.new();
        context.update(data.as_bytes());
        let digest = context.finish();
        hex::encode(digest.as_ref())
    }
}

struct Blockchain {
    chain: Arc<Mutex<Vec<Block>>>,
    wallet_manager: WalletManager,
}

impl Blockchain {
    fn new(wallet_manager: WalletManager) -> Blockchain {
        let genesis_block = Block::new(0, Vec::new(), String::from("0"));
        let chain = vec![genesis_block];

        Blockchain {
            chain: Arc::new(Mutex::new(chain)),
            wallet_manager,
        }
    }

    fn add_transaction(&mut self, transaction: Transaction) {
        let mut chain = self.chain.lock().unwrap();
        let last_block = chain.last().unwrap().clone();
        let index = last_block.index + 1;
        let previous_hash = last_block.hash.clone();

        let block = Block::new(index, vec![transaction], previous_hash);
        chain.push(block);
    }

    fn mine_block(&mut self, miner_address: String) {
        let chain = self.chain.clone();
        let mut chain_lock = chain.lock().unwrap();

        let last_block = chain_lock.last().unwrap().clone();
        let index = last_block.index + 1;
        let previous_hash = last_block.hash.clone();

        let transactions = Vec::new();  // Add transactions here

        let mut block = Block::new(index, transactions, previous_hash);
        block.mine(2);

        chain_lock.push(block);

        // Reward the miner with some amount of cryptocurrency
        let mining_reward = 10.0;  // Arbitrary reward amount
        self.wallet_manager.credit_wallet(&miner_address, mining_reward);
    }
}

impl Block {
    fn mine(&mut self, difficulty: u32) {
        let prefix = "0".repeat(difficulty as usize);
        loop {
            let hash = self.calculate_hash();

            if hash.starts_with(&prefix) {
                self.hash = hash;
                break;
            }

            self.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Failed to obtain a timestamp.")
                .as_secs();
        }
    }
}

struct Wallet {
    address: String,
    balance: f64,
}

impl Wallet {
    fn new(address: String) -> Wallet {
        Wallet {
            address,
            balance: 0.0,
        }
    }
}

struct WalletManager {
    wallets: HashMap<String, Wallet>,
}

impl WalletManager {
    fn new() -> WalletManager {
        WalletManager {
            wallets: HashMap::new(),
        }
    }

    fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new(self.generate_address());
        self.wallets.insert(wallet.address.clone(), wallet);
        wallet.address.clone()
    }

    fn generate_address(&self) -> String {
        // Generate a unique address using cryptography algorithms
        // For simplicity, we'll use a random 8-character string here
        let mut rng = rand::thread_rng();
        let address: String = (0..8)
            .map(|_| rng.sample(rand::distributions::Alphanumeric))
            .collect();
        address
    }

    fn credit_wallet(&mut self, address: &str, amount: f64) {
        if let Some(wallet) = self.wallets.get_mut(address) {
            wallet.balance += amount;
        }
    }

    fn get_balance(&self, address: &str) -> Option<f64> {
        if let Some(wallet) = self.wallets.get(address) {
            Some(wallet.balance)
        } else {
            None
        }
    }
}

fn main() {
    let mut wallet_manager = WalletManager::new();

    // Create wallets
    let wallet1 = wallet_manager.create_wallet();
    let wallet2 = wallet_manager.create_wallet();
    let wallet3 = wallet_manager.create_wallet();

    let mut blockchain = Blockchain::new(wallet_manager);

    // Add initial balances
    blockchain.wallet_manager.credit_wallet(&wallet1, 100.0);
    blockchain.wallet_manager.credit_wallet(&wallet2, 50.0);
    blockchain.wallet_manager.credit_wallet(&wallet3, 25.0);

    // Mine new blocks and update wallet balances
    blockchain.mine_block(wallet1.clone());
    blockchain.mine_block(wallet2.clone());
    blockchain.mine_block(wallet3.clone());

    // Print wallet balances
    println!("Wallet 1: Address={}, Balance={}", wallet1, blockchain.wallet_manager.get_balance(&wallet1).unwrap());
    println!("Wallet 2: Address={}, Balance={}", wallet2, blockchain.wallet_manager.get_balance(&wallet2).unwrap());
    println!("Wallet 3: Address={}, Balance={}", wallet3, blockchain.wallet_manager.get_balance(&wallet3).unwrap());

    // Print the blockchain
    let chain = blockchain.chain.lock().unwrap();
    for block in chain.iter() {
        println!("{:?}", block);
    }
}
