#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult, SubMsg, WasmMsg,
};
use multisig::msg::InstantiateMsg as MultiSigInstantiateMsg;

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MultisigWallets, QueryMsg};
use crate::state::{
    MINTED_MULTISIG_WALLETS, MULTISIG_CODE_ID, MULTISIG_WALLET_MAP, TEMP_WALLET_OWNER,
};

pub(crate) const CONTRACT_NAME: &str = "crates.io:cw-multisig-factory";
pub(crate) const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const INSTANTIATE_CONTRACT_REPLY_ID: u64 = 0;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw_ownable::initialize_owner(deps.storage, deps.api, msg.owner.as_deref())?;
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    MULTISIG_CODE_ID.save(deps.storage, &msg.multisig_code_id)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InstantiateMultiSigContract {
            instantiate_msg,
            label,
        } => execute_instantiate_multisig_contract(deps, info, instantiate_msg, label),
        ExecuteMsg::UpdateCodeId { multisig_code_id } => {
            execute_update_code_id(deps, info, multisig_code_id)
        }
    }
}

pub fn execute_update_code_id(
    deps: DepsMut,
    info: MessageInfo,
    multisig_code_id: u64,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    MULTISIG_CODE_ID.save(deps.storage, &multisig_code_id)?;
    Ok(Response::default()
        .add_attribute("action", "update_code_id")
        .add_attribute("multisig_code_id", multisig_code_id.to_string()))
}

pub fn execute_instantiate_multisig_contract(
    deps: DepsMut,
    info: MessageInfo,
    instantiate_msg: MultiSigInstantiateMsg,
    label: String,
) -> Result<Response, ContractError> {
    instantiate_contract(deps, info.sender, instantiate_msg, label)
}

pub fn instantiate_contract(
    deps: DepsMut,
    sender: Addr,
    instantiate_msg: MultiSigInstantiateMsg,
    label: String,
) -> Result<Response, ContractError> {
    let code_id = MULTISIG_CODE_ID.load(deps.storage)?;

    // Instantiate the specified contract with owner as the admin.
    let instantiate = WasmMsg::Instantiate {
        admin: Some(sender.to_string()),
        code_id,
        msg: to_json_binary(&instantiate_msg)?,
        funds: vec![],
        label,
    };

    TEMP_WALLET_OWNER.save(deps.storage, &sender)?;

    let msg = SubMsg::reply_on_success(instantiate, INSTANTIATE_CONTRACT_REPLY_ID);

    Ok(Response::default()
        .add_attribute("action", "instantiate_cw_multisig")
        .add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_CONTRACT_REPLY_ID => {
            let result = msg.result.into_result().map_err(StdError::generic_err)?;
            let event = result
                .events
                .iter()
                .find(|event| event.ty == "instantiate")
                .ok_or_else(|| StdError::generic_err("cannot find `instantiate_contract` event"))?;

            let contract_address = &event
                .attributes
                .iter()
                .find(|attr| attr.key == "_contract_address")
                .ok_or_else(|| StdError::generic_err("cannot find `contract_address` attribute"))?
                .value;

            MINTED_MULTISIG_WALLETS.update(
                deps.storage,
                |mut wallets| -> StdResult<Vec<String>> {
                    wallets.push(contract_address.to_string());
                    Ok(wallets)
                },
            )?;

            let owner = TEMP_WALLET_OWNER.load(deps.storage)?;
            let mut owner_wallets = MULTISIG_WALLET_MAP
                .load(deps.storage, owner.clone())
                .unwrap_or(Vec::new());
            owner_wallets.push(contract_address.clone());
            MULTISIG_WALLET_MAP.save(deps.storage, owner, &owner_wallets)?;

            Ok(Response::new()
                .add_attribute("method", "handle_instantiate_reply")
                .add_attribute("contract_address", contract_address))
        }
        _ => Err(ContractError::UnknownReplyID {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAllMultisigWallets {} => to_json_binary(&query_multisig_wallets(deps)?),
        QueryMsg::GetMultisigWalletsByOwner { owner } => {
            to_json_binary(&query_multisig_wallets_by_owner(deps, owner)?)
        }
    }
}

fn query_multisig_wallets(deps: Deps) -> StdResult<MultisigWallets> {
    Ok(MultisigWallets {
        wallets: MINTED_MULTISIG_WALLETS.load(deps.storage)?,
    })
}

fn query_multisig_wallets_by_owner(deps: Deps, owner: Addr) -> StdResult<MultisigWallets> {
    let wallets = MULTISIG_WALLET_MAP
        .load(deps.storage, owner)
        .unwrap_or_default();
    Ok(MultisigWallets { wallets })
}
