use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const MULTISIG_CODE_ID: Item<u64> = Item::new("multisig_code_id");
pub const NFT_ADDRESS: Item<Addr> = Item::new("nft_address");
pub const MINTED_MULTISIG_WALLETS: Item<Vec<String>> = Item::new("minted_wallets");
