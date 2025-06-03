// Example 17: Blockchain Integration Implementation
//
// This example demonstrates blockchain concepts including blocks,
// transactions, hashing, and a simple proof-of-work system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::info;

// Struct: Transaction
//
// Represents a transaction in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    from: String,
    to: String,
    amount: f64,
    timestamp: DateTime<Utc>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64) -> Self {
        Self {
            from,
            to,
            amount,
            timestamp: Utc::now(),
        }
    }
}

// Struct: Block
//
// Represents a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    transactions: Vec<Transaction>,
    previous_hash: String,
    nonce: u64,
    hash: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Self {
            index,
            timestamp: Utc::now(),
            transactions,
            previous_hash,
            nonce: 0,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp.to_rfc3339(),
            serde_json::to_string(&self.transactions).unwrap(),
            self.previous_hash,
            self.nonce
        );

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);

        info!("Mining block {}...", self.index);
        let start_time = std::time::Instant::now();

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        let duration = start_time.elapsed();
        info!(
            "Block {} mined in {:?} with nonce {}",
            self.index, duration, self.nonce
        );
    }
}

// Struct: Blockchain
//
// Represents the main blockchain data structure.
pub struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
    pending_transactions: Vec<Transaction>,
    mining_reward: f64,
    balances: HashMap<String, f64>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Self {
            chain: Vec::new(),
            difficulty: 3,
            pending_transactions: Vec::new(),
            mining_reward: 10.0,
            balances: HashMap::new(),
        };

        // Create genesis block
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, Vec::new(), "0".to_string());
        self.chain.push(genesis_block);
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        // Simple validation
        if transaction.from != "system" && self.get_balance(&transaction.from) < transaction.amount
        {
            info!(
                "Transaction rejected: insufficient funds for {}",
                transaction.from
            );
            return;
        }

        self.pending_transactions.push(transaction);
        info!("Transaction added to pending pool");
    }

    pub fn mine_pending_transactions(&mut self, mining_reward_address: String) {
        // Add mining reward transaction
        let reward_transaction = Transaction::new(
            "system".to_string(),
            mining_reward_address.clone(),
            self.mining_reward,
        );
        self.pending_transactions.push(reward_transaction);

        let mut block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            self.get_latest_block().hash.clone(),
        );

        block.mine_block(self.difficulty);
        self.chain.push(block);

        // Update balances
        for transaction in &self.pending_transactions {
            if transaction.from != "system" {
                *self.balances.entry(transaction.from.clone()).or_insert(0.0) -= transaction.amount;
            }
            *self.balances.entry(transaction.to.clone()).or_insert(0.0) += transaction.amount;
        }

        self.pending_transactions.clear();
        info!("Block mined and added to blockchain");
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        *self.balances.get(address).unwrap_or(&0.0)
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }

    pub fn get_chain_info(&self) -> (usize, bool) {
        (self.chain.len(), self.is_chain_valid())
    }
}

// Function: demo_blockchain
//
// Demonstrates blockchain functionality.
fn demo_blockchain() -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Creating Blockchain ===");
    let mut blockchain = Blockchain::new();

    // Give Alice some initial coins
    blockchain.balances.insert("Alice".to_string(), 100.0);
    blockchain.balances.insert("Bob".to_string(), 50.0);

    info!("=== Adding Transactions ===");
    blockchain.add_transaction(Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        25.0,
    ));

    blockchain.add_transaction(Transaction::new(
        "Bob".to_string(),
        "Alice".to_string(),
        10.0,
    ));

    info!("=== Mining Block ===");
    blockchain.mine_pending_transactions("Miner1".to_string());

    info!("=== Balances After Mining ===");
    info!("Alice: {}", blockchain.get_balance("Alice"));
    info!("Bob: {}", blockchain.get_balance("Bob"));
    info!("Miner1: {}", blockchain.get_balance("Miner1"));

    // Add more transactions
    blockchain.add_transaction(Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        20.0,
    ));

    blockchain.mine_pending_transactions("Miner2".to_string());

    info!("=== Final Balances ===");
    info!("Alice: {}", blockchain.get_balance("Alice"));
    info!("Bob: {}", blockchain.get_balance("Bob"));
    info!("Miner1: {}", blockchain.get_balance("Miner1"));
    info!("Miner2: {}", blockchain.get_balance("Miner2"));

    let (chain_length, is_valid) = blockchain.get_chain_info();
    info!("=== Blockchain Statistics ===");
    info!("Chain length: {}", chain_length);
    info!("Chain valid: {}", is_valid);

    Ok(())
}

// Function: main
//
// Entry point demonstrating blockchain implementation.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting Blockchain Integration Example");
    demo_blockchain()?;
    info!("Blockchain Integration Example completed successfully");

    Ok(())
}
