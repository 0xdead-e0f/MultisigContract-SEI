use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Quorum: {quorum} is more than the number of owners: {owners}")]
    WrongQuorum { quorum: u32, owners: u32 },

    #[error("Number of owners can't be 0")]
    ZeroOwners,

    #[error("Transaction with tx_id: {0}, doesn't exist")]
    NonExistentTx(u128),

    #[error("You already signed transaction with id: {0}")]
    AlreadySigned(u128),

    #[error("Not enough admins signed this transaction, the quorum is {quorum} and only {num_signed} signed the transaction")]
    NotEnoughSignatures { quorum: u32, num_signed: u32 },
}
