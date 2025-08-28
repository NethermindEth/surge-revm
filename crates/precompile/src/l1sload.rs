use crate::{Error, Precompile, PrecompileResult, PrecompileWithAddress};
use revm_primitives::{Address, Bytes, PrecompileOutput, B256};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub const L1SLOAD: PrecompileWithAddress = PrecompileWithAddress(
    crate::u64_to_address(0x10001),
    Precompile::Standard(l1sload_run),
);

/// Gas constants for L1SLOAD precompile
pub const L1SLOAD_FIXED_GAS: u64 = 2000;
pub const L1SLOAD_PER_LOAD_GAS: u64 = 2000;

/// Expected input length: 20 bytes (address) + 32 bytes (storage key) + 32 bytes (block number) = 84 bytes
pub const EXPECTED_INPUT_LENGTH: usize = 84;

/// In-memory cache for L1 storage values
static L1_STORAGE_CACHE: LazyLock<Mutex<HashMap<(Address, B256, B256), B256>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Set a value in the L1 storage cache
pub fn set_l1_storage_value(
    contract_address: Address,
    storage_key: B256,
    block_number: B256,
    value: B256,
) {
    if let Ok(mut cache) = L1_STORAGE_CACHE.lock() {
        cache.insert((contract_address, storage_key, block_number), value);
    }
}

/// Get a value from the L1 storage cache
fn get_l1_storage_value(
    contract_address: Address,
    storage_key: B256,
    block_number: B256,
) -> Option<B256> {
    if let Ok(cache) = L1_STORAGE_CACHE.lock() {
        cache
            .get(&(contract_address, storage_key, block_number))
            .copied()
    } else {
        None
    }
}

/// L1SLOAD precompile - read storage values from L1 contracts (RIP-7728).
///
/// The input to the L1SLOAD precompile consists of:
/// - [0: 19] (20 bytes)  - address: The L1 contract address
/// - [20: 51] (32 bytes) - storageKey: The storage key to read
/// - [52: 83] (32 bytes) - blockNumber: The L1 block number to read from
///
/// Output: Storage value (32 bytes)
pub fn l1sload_run(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    // Check gas limit
    let gas_used = L1SLOAD_FIXED_GAS + L1SLOAD_PER_LOAD_GAS;
    if gas_used > gas_limit {
        return Err(Error::OutOfGas.into());
    }

    // Validate input length
    if input.len() != EXPECTED_INPUT_LENGTH {
        return Err(Error::Other("Invalid input length".into()).into());
    }

    // Parse input parameters
    let contract_address = Address::from_slice(&input[0..20]);
    let storage_key = B256::from_slice(&input[20..52]);
    let block_number = B256::from_slice(&input[52..84]);

    // Get cached L1 storage value
    let storage_value = get_l1_storage_value(contract_address, storage_key, block_number);

    match storage_value {
        Some(value) => {
            // Convert storage value to output bytes (32 bytes)
            let mut output = [0u8; 32];
            output.copy_from_slice(value.as_slice());
            Ok(PrecompileOutput::new(gas_used, Bytes::from(output)))
        }
        None => {
            // Return zero value if no cached data found
            let output = [0u8; 32];
            Ok(PrecompileOutput::new(gas_used, Bytes::from(output)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1sload_invalid_input_length() {
        let short_input = Bytes::from(vec![0u8; 80]); // Too short
        let result = l1sload_run(&short_input, 1000);
        assert!(result.is_err());

        let long_input = Bytes::from(vec![0u8; 100]); // Too long
        let result = l1sload_run(&long_input, 1000);
        assert!(result.is_err());
    }

    #[test]
    fn test_l1sload_valid_input() {
        let mut input = vec![0u8; EXPECTED_INPUT_LENGTH];

        // Set contract address (20 bytes)
        input[0..20].copy_from_slice(&[1u8; 20]);

        // Set storage key (32 bytes)
        input[20..52].copy_from_slice(&[2u8; 32]);

        // Set block number (32 bytes)
        input[52..84].copy_from_slice(&[3u8; 32]);

        let result = l1sload_run(&Bytes::from(input), 1000);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.gas_used, L1SLOAD_FIXED_GAS + L1SLOAD_PER_LOAD_GAS);
        assert_eq!(output.bytes.len(), 32);
    }

    #[test]
    fn test_l1sload_out_of_gas() {
        let input = Bytes::from(vec![0u8; EXPECTED_INPUT_LENGTH]);
        let result = l1sload_run(&input, 50); // Not enough gas
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_functionality() {
        let address = Address::from_slice(&[1u8; 20]);
        let key = B256::from_slice(&[2u8; 32]);
        let block = B256::from_slice(&[3u8; 32]);
        let value = B256::from_slice(&[5u8; 32]);

        // Initially no data
        assert!(get_l1_storage_value(address, key, block).is_none());

        // Set data
        set_l1_storage_value(address, key, block, value);

        // Now data exists
        assert_eq!(get_l1_storage_value(address, key, block), Some(value));
    }
}
