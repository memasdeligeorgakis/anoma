use borsh::{BorshDeserialize, BorshSerialize, BorshSchema};

use super::storage as parameters_storage;
use crate::ledger::storage::types::{encode, decode, self};
use crate::ledger::storage::{self, Storage};
use crate::types::storage::Key;
use crate::types::time::DurationSecs;

use thiserror::Error;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Storage error: {0}")]
    StorageError(storage::Error),
    #[error("Storage type error: {0}")]
    StorageTypeError(types::Error),
    #[error("Protocol parameters are missing, they must be always set")]
    ParametersMissing,
}

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum WriteError {
    #[error("Storage error: {0}")]
    StorageError(storage::Error),
    #[error("Serialize error: {0}")]
    SerializeError(String),
}

/// Protocol parameters
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
)]
pub struct Parameters {
    /// Epoch duration
    pub epoch_duration: EpochDuration,
    /// Maximum expected time per block
    pub max_expected_time_per_block: DurationSecs,
    /// Whitelisted validity predicate hashes
    pub vp_whitelist: Vec<String>,
    /// Whitelisted tx hashes
    pub tx_whitelist: Vec<String>,
}

/// Epoch duration. A new epoch begins as soon as both the `min_num_of_blocks`
/// and `min_duration` have passed since the beginning of the current epoch.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    BorshSerialize,
    BorshDeserialize,
    BorshSchema,
)]
pub struct EpochDuration {
    /// Minimum number of blocks in an epoch
    pub min_num_of_blocks: u64,
    /// Minimum duration of an epoch
    pub min_duration: DurationSecs,
}

impl Parameters {
    /// Initialize parameters in storage in the genesis block.
    pub fn init_storage<DB, H>(
        &self, 
        storage: &mut Storage<DB, H>,
    ) where
        DB: storage::DB + for<'iter> storage::DBIter<'iter>,
        H: storage::StorageHasher,
    {
        // write epoch parameters
        let epoch_key = parameters_storage::get_epoch_storage_key();
        let epoch_value = encode(&self.epoch_duration);
        storage
            .write(&epoch_key, epoch_value)
            .expect("Epoch parameters must be initialized in the genesis block");

        // write vp whitelist parameter
        let vp_whitelist_key = parameters_storage::get_vp_whitelist_storage_key();
        let vp_whitelist_value = encode(&self.vp_whitelist);
        storage.write(&vp_whitelist_key, vp_whitelist_value).expect(
            "Vp whitelist parameters must be initialized in the genesis block",
        );

        // write tx whitelist parameter
        let tx_whitelist_key = parameters_storage::get_tx_whitelist_storage_key();
        let tx_whitelist_value = encode(&self.tx_whitelist);
        storage.write(&tx_whitelist_key, tx_whitelist_value).expect(
            "Tx whitelist parameters must be initialized in the genesis block",
        );

        // write tx whitelist parameter
        let max_expected_time_per_block_key = parameters_storage::get_max_expected_time_per_block_key();
        let max_expected_time_per_block_value =
            encode(&self.max_expected_time_per_block);
        storage
            .write(
                &max_expected_time_per_block_key,
                max_expected_time_per_block_value,
            )
            .expect(
                "Max expected time per block parameters must be initialized in \
                the genesis block",
            );
    }
}

/// Update the max_expected_time_per_block parameter in storage. Returns the
/// parameters and gas cost.
pub fn update_max_expected_time_per_block_parameter<DB, H>(
    storage: &mut Storage<DB, H>,
    value: &DurationSecs,
) -> std::result::Result<u64, WriteError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    let key = parameters_storage::get_max_expected_time_per_block_key();
    update(storage, value, key)
}

/// Update the vp whitelist parameter in storage. Returns the parameters and gas
/// cost.
pub fn update_vp_whitelist_parameter<DB, H>(
    storage: &mut Storage<DB, H>,
    value: Vec<String>,
) -> std::result::Result<u64, WriteError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    let key = parameters_storage::get_vp_whitelist_storage_key();
    update(storage, &value, key)
}

/// Update the tx whitelist parameter in storage. Returns the parameters and gas
/// cost.
pub fn update_tx_whitelist_parameter<DB, H>(
    storage: &mut Storage<DB, H>,
    value: Vec<String>,
) -> std::result::Result<u64, WriteError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    let key = parameters_storage::get_tx_whitelist_storage_key();
    update(storage, &value, key)
}

/// Update the epoch parameter in storage. Returns the parameters and gas
/// cost.
pub fn update_epoch_parameter<DB, H>(
    storage: &mut Storage<DB, H>,
    value: &EpochDuration,
) -> std::result::Result<u64, WriteError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    let key = parameters_storage::get_epoch_storage_key();
    update(storage, value, key)
}

/// Update the  parameters in storage. Returns the parameters and gas
/// cost.
pub fn update<DB, H, T>(
    storage: &mut Storage<DB, H>,
    value: &T,
    key: Key,
) -> std::result::Result<u64, WriteError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
    T: BorshSerialize,
{
    let serialized_value = value
        .try_to_vec()
        .map_err(|e| WriteError::SerializeError(e.to_string()))?;
    let (gas, _size_diff) = storage
        .write(&key, serialized_value)
        .map_err(WriteError::StorageError)?;
    Ok(gas)
}

/// Read the the epoch duration parameter from store
pub fn read_epoch_parameter<DB, H>(
    storage: &Storage<DB, H>,
) -> std::result::Result<(EpochDuration, u64), ReadError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    // read epoch
    let epoch_key = parameters_storage::get_epoch_storage_key();
    let (value, gas) =
        storage.read(&epoch_key).map_err(ReadError::StorageError)?;
    let epoch_duration: EpochDuration =
        decode(value.ok_or(ReadError::ParametersMissing)?)
            .map_err(ReadError::StorageTypeError)?;

    Ok((epoch_duration, gas))
}

// Read the all the parameters from storage. Returns the parameters and gas
/// cost.
pub fn read<DB, H>(
    storage: &Storage<DB, H>,
) -> std::result::Result<(Parameters, u64), ReadError>
where
    DB: storage::DB + for<'iter> storage::DBIter<'iter>,
    H: storage::StorageHasher,
{
    // read epoch
    let (epoch_duration, gas_epoch) = read_epoch_parameter(storage)
        .expect("Couldn't read epoch duration parameters");

    // read vp whitelist
    let vp_whitelist_key = parameters_storage::get_vp_whitelist_storage_key();
    let (value, gas_vp) = storage
        .read(&vp_whitelist_key)
        .map_err(ReadError::StorageError)?;
    let vp_whitelist: Vec<String> =
        decode(value.ok_or(ReadError::ParametersMissing)?)
            .map_err(ReadError::StorageTypeError)?;

    // read tx whitelist
    let tx_whitelist_key = parameters_storage::get_tx_whitelist_storage_key();
    let (value, gas_tx) = storage
        .read(&tx_whitelist_key)
        .map_err(ReadError::StorageError)?;
    let tx_whitelist: Vec<String> =
        decode(value.ok_or(ReadError::ParametersMissing)?)
            .map_err(ReadError::StorageTypeError)?;

    let max_expected_time_per_block_key = parameters_storage::get_max_expected_time_per_block_key();
    let (value, gas_time) = storage
        .read(&max_expected_time_per_block_key)
        .map_err(ReadError::StorageError)?;
    let max_expected_time_per_block: DurationSecs =
        decode(value.ok_or(ReadError::ParametersMissing)?)
            .map_err(ReadError::StorageTypeError)?;

    Ok((
        Parameters {
            epoch_duration,
            max_expected_time_per_block,
            vp_whitelist,
            tx_whitelist,
        },
        gas_epoch + gas_tx + gas_vp + gas_time,
    ))
}