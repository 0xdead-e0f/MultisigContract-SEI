#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, WasmMsg
};
use multisig::msg::InstantiateMsg as MultiSigInstantiateMsg;

use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::MULTISIG_CODE_ID;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
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
    instantiate_contract(deps, info.sender, Some(info.funds), instantiate_msg, label)
}

pub fn instantiate_contract(
    deps: DepsMut,
    sender: Addr,
    funds: Option<Vec<Coin>>,
    instantiate_msg: MultiSigInstantiateMsg,
    label: String,
) -> Result<Response, ContractError> {
    // Check sender is contract owner if set
    // let ownership = cw_ownable::get_ownership(deps.storage)?;
    // if ownership
    //     .owner
    //     .as_ref()
    //     .map_or(false, |owner| *owner != sender)
    // {
    //     return Err(ContractError::Unauthorized {});
    // }

    let code_id = MULTISIG_CODE_ID.load(deps.storage)?;

    // Instantiate the specified contract with owner as the admin.
    let instantiate = WasmMsg::Instantiate {
        admin: Some(sender.to_string()),
        code_id,
        msg: to_json_binary(&instantiate_msg)?,
        funds: funds.unwrap_or_default(),
        label,
    };

    let msg = SubMsg::reply_on_success(instantiate, INSTANTIATE_CONTRACT_REPLY_ID);

    Ok(Response::default()
        .add_attribute("action", "instantiate_cw_multisig")
        .add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        INSTANTIATE_CONTRACT_REPLY_ID => {
            let res = parse_reply_instantiate_data(msg)?;
            let contract_addr = deps.api.addr_validate(&res.contract_address)?;
            // Make the contract its own admin.
            let msg = WasmMsg::UpdateAdmin {
                contract_addr: contract_addr.to_string(),
                admin: contract_addr.to_string(),
            };

            Ok(Response::default()
                .add_attribute("new_multisig_wallet", contract_addr)
                .add_message(msg))
        }
        _ => Err(ContractError::UnknownReplyID {}),
    }
}