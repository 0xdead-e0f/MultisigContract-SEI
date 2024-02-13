use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const MULTISIG_CODE_ID: Item<u64> = Item::new("multisig_code_id");
pub const NFT_ADDRESS: Item<Addr> = Item::new("nft_address");
pub const MULTISIG_WALLET_MAP: Map<Addr, Vec<String>> = Map::new("multisig_map");
pub const TEMP_WALLET_OWNER: Item<Addr> = Item::new("wallet_owner_temp");
