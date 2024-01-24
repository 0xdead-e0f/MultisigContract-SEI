use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Transaction {
    pub to: Addr,
    pub coins: Vec<Coin>,
    pub id: u32,
    pub num_confirmations: u32,
}

trait ToStr {
    fn to_string(&self) -> String;
}

impl ToStr for Vec<Coin> {
    fn to_string(&self) -> String {
        self.iter().map(|coin| coin.to_string()).collect::<String>()
    }
}

impl Transaction {
    pub fn new(to: Addr, id: u32, coins: Vec<Coin>) -> Self {
        Self {
            to,
            coins,
            id,
            num_confirmations: 0,
        }
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        format!(
            "Transaction {{ to: {}, coin: {}, id: {} }}",
            self.to,
            self.coins.to_string(),
            self.id
        )
    }
}

#[cw_serde]
pub struct PendingTransactions(pub Vec<Transaction>);

impl PendingTransactions {
    pub fn new(txs: Vec<Transaction>) -> Self {
        Self(txs)
    }

    pub fn index(&self, index: u32) -> Option<&Transaction> {
        self.0.iter().nth(index as usize)
    }

    pub fn next_id(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn push(&mut self, tx: Transaction) {
        self.0.push(tx);
    }

    pub fn find_mut(&mut self, tx_id: u32) -> Option<&mut Transaction> {
        self.0.iter_mut().find(|tx| tx.id == tx_id)
    }

    pub fn find(&self, tx_id: u32) -> Option<&Transaction> {
        self.0.iter().find(|tx| tx.id == tx_id)
    }
}

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");
pub const QUORUM: Item<u32> = Item::new("quorum");
pub const PENDING_TXS: Item<PendingTransactions> = Item::new("pending_txs");
pub const SIGNED_TX: Map<(Addr, u32), bool> = Map::new("signed_tx");