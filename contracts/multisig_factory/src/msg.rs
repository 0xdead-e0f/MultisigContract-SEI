use cosmwasm_schema::{cw_serde, QueryResponses};
use multisig::msg::InstantiateMsg as MultiSigInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Option<String>,
    pub multisig_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Instantiates a new multisig contract that is funded by a native token.
    InstantiateMultiSigContract {
        instantiate_msg: MultiSigInstantiateMsg,
        label: String,
    },

    /// Callable only by the current owner. Updates the code ID used
    /// while instantiating multisig contracts.
    UpdateCodeId { multisig_code_id: u64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
