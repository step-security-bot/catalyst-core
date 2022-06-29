use crate::{
    context::Context,
    jrpc::{
        eth_types::{block_number::BlockNumber, bytes::Bytes, number::Number},
        Error,
    },
};
use chain_evm::ethereum_types::{H160, H256};

pub fn accounts(context: &Context) -> Result<Vec<H160>, Error> {
    Ok(context
        .try_full()?
        .evm_keys
        .iter()
        .map(|secret_key| secret_key.address())
        .collect())
}

pub fn get_transaction_count(
    _address: H160,
    _block_number: BlockNumber,
    _context: &Context,
) -> Result<Number, Error> {
    // TODO implement
    Ok(0.into())
}

pub fn get_balance(
    _address: H160,
    _block_number: BlockNumber,
    _context: &Context,
) -> Result<Number, Error> {
    // TODO implement
    Ok(0.into())
}

pub fn get_code(
    _address: H160,
    _block_number: BlockNumber,
    _context: &Context,
) -> Result<Bytes, Error> {
    // TODO implement
    Ok(Default::default())
}

pub fn get_storage_at(
    _address: H160,
    _key: H256,
    _block_number: BlockNumber,
    _context: &Context,
) -> Result<H256, Error> {
    // TODO implement
    Ok(H256::zero())
}