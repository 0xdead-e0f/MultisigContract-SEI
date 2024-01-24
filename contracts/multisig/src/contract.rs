#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{PendingTransactions, Transaction, ADMINS, PENDING_TXS, QUORUM, SIGNED_TX};

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
    let pending = PendingTransactions::new(Vec::new());
    PENDING_TXS.save(deps.storage, &pending)?;
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
        ExecuteMsg::CreateTransaction { to, coins } => exec::create_tx(deps, info, to, coins),
        ExecuteMsg::SignTransactions { tx_id } => exec::sign_tx(deps, info, tx_id),
        ExecuteMsg::ExecuteTransaction { tx_id } => exec::execute_tx(deps, tx_id),
    }
}

mod exec {
    use super::*;
    use crate::state::{PENDING_TXS, SIGNED_TX};
    use cosmwasm_std::{Addr, BankMsg, Coin};

    pub fn create_tx(
        deps: DepsMut,
        info: MessageInfo,
        to: Addr,
        coins: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        let admins = ADMINS.load(deps.storage)?;

        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        let mut pending_txs = PENDING_TXS.load(deps.storage)?;
        let next_id = pending_txs.next_id();
        let mut tx = Transaction::new(to, next_id, coins);
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
        tx_id: u32,
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

    pub fn execute_tx(deps: DepsMut, tx_id: u32) -> Result<Response, ContractError> {
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

        let message = BankMsg::Send {
            to_address: tx.to.to_string(),
            amount: tx.coins.clone(),
        };

        Ok(Response::new().add_message(message))
    }

    pub fn is_admin(deps: &DepsMut, info: &MessageInfo) -> Result<(), ContractError> {
        let admins = ADMINS.load(deps.storage)?;
        if !admins.contains(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        Ok(())
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
    }
}

mod query {
    use super::*;
    use crate::{
        msg::{ListAdminsResp, ListPendingResp, ListSignedResp},
        state::{PENDING_TXS, SIGNED_TX},
    };
    use cosmwasm_std::Addr;

    pub fn list_signed(deps: Deps, admin: Addr, tx_id: u32) -> StdResult<ListSignedResp> {
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
}
