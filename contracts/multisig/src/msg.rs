use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{CompletedTransactions, PendingTransactions, TxMsg};

#[cw_serde]
pub struct InstantiateMsg {
    /// Owners that need to sign transactions
    pub owners: Vec<Addr>,
    /// Quorum must be < owners.len()
    pub quorum: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateTransaction { tx_msg: TxMsg },
    SignTransactions { tx_id: u128 },
    ExecuteTransaction { tx_id: u128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ListPendingResp)]
    ListPending {},

    #[returns(ListPendingResp)]
    ListCompletedTxs {offset: Option<u32>, limit: Option<u32>},

    #[returns(ListAdminsResp)]
    ListAdmins {},

    #[returns(ListSignedResp)]
    ListSigned { admin: Addr, tx_id: u128 },

    #[returns(u32)]
    GetQuorum {},
}

#[cw_serde]
pub struct ListSignedResp {
    pub signed: bool,
}

#[cw_serde]
pub struct ListAdminsResp {
    pub admins: Vec<Addr>,
}

#[cw_serde]
pub struct ListPendingResp {
    pub transactions: PendingTransactions,
}

#[cw_serde]
pub struct ListCompletedTxsResp {
    pub transactions: CompletedTransactions,
}

