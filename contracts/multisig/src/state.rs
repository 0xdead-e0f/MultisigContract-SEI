use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Transaction {
    pub tx_msg: TxMsg,
    pub id: u128,
    pub num_confirmations: u32,
}

#[cw_serde]
pub enum TxMsg {
    TxBank { to: Addr, coins: Vec<Coin> },
    TxSelf(SelfTx),
}

#[cw_serde]
pub enum SelfTx {
    AddOwner { owner: Addr, quorum: Option<u32> },
    RemoveOwner { owner: Addr, quorum: Option<u32> },
    UpdateQuorum { quorum: u32 },
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
    pub fn new(tx_msg: TxMsg, id: u128) -> Self {
        Self {
            tx_msg,
            id,
            num_confirmations: 0,
        }
    }
}

impl ToString for Transaction {
    fn to_string(&self) -> String {
        format!("Transaction {{ tx_msg: {}, id: {} }}", self.tx_msg, self.id)
    }
}

impl Display for TxMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxMsg::TxSelf(self_msg) => match self_msg {
                SelfTx::AddOwner { owner, quorum } => write!(
                    f,
                    "SelfTx-AddOwner {{owner: {}, quorum: {:?}}}",
                    owner, quorum
                ),
                SelfTx::RemoveOwner { owner, quorum } => write!(
                    f,
                    "SelfTx-RemoveOwner {{owner: {}, quorum: {:?}}}",
                    owner, quorum
                ),
                SelfTx::UpdateQuorum { quorum } => {
                    write!(f, "SelfTx-UpdateQuorum {{quorum: {}}}", quorum)
                }
            },
            TxMsg::TxBank { to, coins } => {
                write!(f, "BankTx {{to: {}, coin: {}}}", to, coins.to_string())
            }
        }
    }
}

#[cw_serde]
pub struct PendingTransactions(pub Vec<Transaction>);

impl PendingTransactions {
    pub fn new(txs: Vec<Transaction>) -> Self {
        Self(txs)
    }

    pub fn index(&self, index: u128) -> Option<&Transaction> {
        self.0.iter().nth(index as usize)
    }

    pub fn push(&mut self, tx: Transaction) {
        self.0.push(tx);
    }

    pub fn find_mut(&mut self, tx_id: u128) -> Option<&mut Transaction> {
        self.0.iter_mut().find(|tx| tx.id == tx_id)
    }

    pub fn find(&self, tx_id: u128) -> Option<&Transaction> {
        self.0.iter().find(|tx| tx.id == tx_id)
    }

    pub fn remove(&mut self, tx: &Transaction) -> Option<Transaction> {
        let index = self.0.iter().position(|item|item.id==tx.id);
        match index {
            Some(i) => {
                let ret_tx: Transaction = self.0.remove(i);
                Some(ret_tx)
            },
            None => None
        }

    }
}

#[cw_serde]
pub struct CompletedTransactions(pub Vec<Transaction>);

impl CompletedTransactions {
    pub fn new(txs: Vec<Transaction>) -> Self {
        Self(txs)
    }

    pub fn index(&self, index: u128) -> Option<&Transaction> {
        self.0.iter().nth(index as usize)
    }

    pub fn push(&mut self, tx: Transaction) {
        self.0.push(tx);
    }

    pub fn find_mut(&mut self, tx_id: u128) -> Option<&mut Transaction> {
        self.0.iter_mut().find(|tx| tx.id == tx_id)
    }

    pub fn find(&self, tx_id: u128) -> Option<&Transaction> {
        self.0.iter().find(|tx| tx.id == tx_id)
    }
}

pub const ADMINS: Item<Vec<Addr>> = Item::new("admins");
pub const QUORUM: Item<u32> = Item::new("quorum");
pub const TX_COUNTER: Item<u128> = Item::new("tx_counter");
pub const PENDING_TXS: Item<PendingTransactions> = Item::new("pending_txs");
pub const COMPLETED_TXS: Item<CompletedTransactions> = Item::new("completed_txs");
pub const SIGNED_TX: Map<(Addr, u128), bool> = Map::new("signed_tx");
