#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CompletedTransactions, PendingTransactions, Transaction, ADMINS, COMPLETED_TXS, PENDING_TXS, QUORUM, SIGNED_TX, TX_COUNTER};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.owners.len() == 0 {
        return Err(ContractError::ZeroOwners);
    }

    if msg.quorum > msg.owners.len() as u32 {
        return Err(ContractError::WrongQuorum {
            quorum: msg.quorum,
            owners: msg.owners.len() as u32,
        });
    }

    ADMINS.save(deps.storage, &msg.owners)?;
    QUORUM.save(deps.storage, &msg.quorum)?;
    TX_COUNTER.save(deps.storage, &0u128)?;

    let pending_txs = PendingTransactions::new(Vec::new());
    PENDING_TXS.save(deps.storage, &pending_txs)?;

    let completed_txs = CompletedTransactions::new(Vec::new());
    COMPLETED_TXS.save(deps.storage, &completed_txs)?;

    SIGNED_TX.save(deps.storage, (Addr::unchecked("test"), 0), &false)?;

    let events = msg
        .owners
        .into_iter()
        .map(|owner| Event::new("owner-added").add_attribute("addr", owner));

    Ok(Response::new().add_events(events))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    exec::is_admin(&deps, &info)?;
    match msg {
        ExecuteMsg::CreateTransaction { tx_msg } => exec::create_tx(deps, info, tx_msg),
        ExecuteMsg::SignTransactions { tx_id } => exec::sign_tx(deps, info, tx_id),
        ExecuteMsg::ExecuteTransaction { tx_id } => exec::execute_tx(deps, tx_id),
    }
}

mod exec {
    use super::*;
    use crate::state::{TxMsg, PENDING_TXS, SIGNED_TX};
    use cosmwasm_std::{Addr, BankMsg};

    pub fn make_next_id(deps: DepsMut) -> StdResult<u128> {
        let mut tx_counter = TX_COUNTER.load(deps.storage)?;
        tx_counter += 1;
        TX_COUNTER.save(deps.storage, &tx_counter)?;
        Ok(tx_counter)
    }

    pub fn create_tx(
        mut deps: DepsMut,
        info: MessageInfo,
        tx_msg: TxMsg,
    ) -> Result<Response, ContractError> {
        let admins = ADMINS.load(deps.storage)?;

        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        let mut pending_txs = PENDING_TXS.load(deps.storage)?;
        let next_id = make_next_id(deps.branch())?;
        let mut tx = Transaction::new(tx_msg, next_id);

        tx.num_confirmations = 1;
        pending_txs.push(tx.clone());
        PENDING_TXS.save(deps.storage, &pending_txs)?;

        // Since the user proposed the tx he already approves that it will be executed,
        // This way he won't have to approve the transaction again
        SIGNED_TX.save(deps.storage, (info.sender, next_id), &true)?;
        Ok(Response::new().add_event(Event::new("new_tx").add_attribute("tx", tx.to_string())))
    }

    pub fn sign_tx(
        deps: DepsMut,
        info: MessageInfo,
        tx_id: u128,
    ) -> Result<Response, ContractError> {
        if let Ok(signed) = SIGNED_TX.load(deps.storage, (info.sender.clone(), tx_id)) {
            if signed {
                return Err(ContractError::AlreadySigned(tx_id));
            }
        }

        SIGNED_TX.save(deps.storage, (info.sender, tx_id), &true)?;

        let mut pending_txs = PENDING_TXS.load(deps.storage)?;

        let tx = pending_txs
            .find_mut(tx_id)
            .ok_or(ContractError::NonExistentTx(tx_id))?;

        tx.num_confirmations += 1;

        PENDING_TXS.save(deps.storage, &pending_txs)?;

        Ok(Response::new())
    }

    pub fn execute_tx(deps: DepsMut, tx_id: u128) -> Result<Response, ContractError> {
        let pending_txs = PENDING_TXS.load(deps.storage)?;

        let tx = pending_txs
            .find(tx_id)
            .ok_or(ContractError::NonExistentTx(tx_id))?;

        let quorum = QUORUM.load(deps.storage)?;

        if quorum > tx.num_confirmations {
            return Err(ContractError::NotEnoughSignatures {
                quorum,
                num_signed: tx.num_confirmations,
            });
        }

        PENDING_TXS.update(deps.storage, |mut txs|->StdResult<PendingTransactions>{
            txs.remove(&tx.clone());
            Ok(txs)
        })?;

        COMPLETED_TXS.update(deps.storage, |mut txs: CompletedTransactions|-> StdResult<CompletedTransactions>{
            txs.push(tx.clone());
            Ok(txs)
        })?;

        let response = match tx.tx_msg.clone() {
            TxMsg::TxBank { to, coins } => {
                let message = BankMsg::Send {
                    to_address: to.to_string(),
                    amount: coins.clone(),
                };
                Ok(Response::new().add_message(message))
            }
            TxMsg::TxSelf(self_msg) => match self_msg {
                crate::state::SelfTx::AddOwner { owner, quorum } => add_owner(deps, owner, quorum),
                crate::state::SelfTx::RemoveOwner { owner, quorum } => {
                    remove_owner(deps, owner, quorum)
                }
                crate::state::SelfTx::UpdateQuorum { quorum } => update_quorum(deps, quorum),
            },
        };
        
        response
    }

    pub fn is_admin(deps: &DepsMut, info: &MessageInfo) -> Result<(), ContractError> {
        let admins = ADMINS.load(deps.storage)?;
        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        Ok(())
    }

    fn add_owner(
        deps: DepsMut,
        owner: Addr,
        quorum: Option<u32>,
    ) -> Result<Response, ContractError> {
        let mut admins = ADMINS.load(deps.storage)?;
        admins.push(owner);

        if let Some(quorum_val) = quorum {
            if quorum_val > admins.len() as u32 {
                return Err(ContractError::WrongQuorum {
                    quorum: quorum_val,
                    owners: admins.len() as u32,
                });
            }
            QUORUM.save(deps.storage, &quorum_val)?;
        }
        ADMINS.save(deps.storage, &admins)?;

        Ok(Response::new())
    }

    fn remove_owner(
        deps: DepsMut,
        owner: Addr,
        quorum: Option<u32>,
    ) -> Result<Response, ContractError> {
        let mut admins = ADMINS.load(deps.storage)?;
        let index = admins.iter().position(|x| *x == owner).unwrap();
        admins.remove(index);

        if let Some(quorum_val) = quorum {
            if quorum_val > admins.len() as u32 {
                return Err(ContractError::WrongQuorum {
                    quorum: quorum_val,
                    owners: admins.len() as u32,
                });
            }
            QUORUM.save(deps.storage, &quorum_val)?;
        }

        ADMINS.save(deps.storage, &admins)?;
        Ok(Response::new())
    }

    fn update_quorum(deps: DepsMut, quorum: u32) -> Result<Response, ContractError> {
        let admins = ADMINS.load(deps.storage)?;
        if quorum > admins.len() as u32 {
            return Err(ContractError::WrongQuorum {
                quorum: quorum,
                owners: admins.len() as u32,
            });
        }
        QUORUM.save(deps.storage, &quorum)?;
        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ListAdmins {} => to_json_binary(&query::list_admins(deps)?),
        QueryMsg::ListPending {} => to_json_binary(&query::list_pending(deps)?),
        QueryMsg::ListSigned { admin, tx_id } => {
            to_json_binary(&query::list_signed(deps, admin, tx_id)?)
        }
        QueryMsg::ListCompletedTxs { offset, limit } => to_json_binary(&query::list_completed_txs(deps, offset, limit)?),
        QueryMsg::GetQuorum {  } => to_json_binary(&query::get_quorum(deps)?),
    }
}

mod query {
    use super::*;
    use crate::{
        msg::{ListAdminsResp, ListCompletedTxsResp, ListPendingResp, ListSignedResp},
        state::{PENDING_TXS, SIGNED_TX},
    };
    use cosmwasm_std::Addr;

    pub fn list_signed(deps: Deps, admin: Addr, tx_id: u128) -> StdResult<ListSignedResp> {
        let signed = SIGNED_TX.load(deps.storage, (admin, tx_id))?;

        Ok(ListSignedResp { signed })
    }

    pub fn list_admins(deps: Deps) -> StdResult<ListAdminsResp> {
        let admins = ADMINS.load(deps.storage)?;
        Ok(ListAdminsResp { admins })
    }

    pub fn list_pending(deps: Deps) -> StdResult<ListPendingResp> {
        let transactions = PENDING_TXS.load(deps.storage)?;

        Ok(ListPendingResp { transactions })
    }

    pub fn list_completed_txs(deps: Deps, offset: Option<u32>, limit: Option<u32>) ->StdResult<ListCompletedTxsResp> {
        let completed_txs = COMPLETED_TXS.load(deps.storage)?;

        let start_pos = offset.unwrap_or(0) as usize;
        let end_pos = match limit {
            Some(lim) => start_pos + lim as usize,
            None => completed_txs.0.len(), // If no limit is provided, fetch all to the end.
        };

        let end_pos = end_pos.min(completed_txs.0.len());

        let paginated_txs = if start_pos < completed_txs.0.len() {
            &completed_txs.0[start_pos..end_pos]
        } else {
            &[] // If start_pos is out of bounds, return an empty slice.
        };

        let resp = ListCompletedTxsResp {
            transactions: CompletedTransactions(paginated_txs.to_vec()), // This might require adjustment depending on your actual types and structures.
        };

        Ok(resp)
    }

    pub fn get_quorum(deps: Deps) ->StdResult<u32> {
        let quorum = QUORUM.load(deps.storage)?;
        Ok(quorum)
    }
}
