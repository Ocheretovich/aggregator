#![no_std]
use soroban_sdk::{
    contract, contractimpl, token::Client as TokenClient, Address, BytesN, Env, String, Vec,
};

mod error;
mod event;
mod models;
mod storage;
mod test;

use error::AggregatorError;
use models::{Adapter, DexDistribution, MAX_DISTRIBUTION_LENGTH};
use soroswap_aggregator_adapter_interface::SoroswapAggregatorAdapterClient;
use storage::{
    extend_instance_ttl, get_adapter, get_admin, get_protocol_ids, has_adapter, is_initialized,
    put_adapter, remove_adapter, set_admin, set_initialized, set_pause_protocol,
};

pub fn check_nonnegative_amount(amount: i128) -> Result<(), AggregatorError> {
    if amount < 0 {
        Err(AggregatorError::NegativeNotAllowed)
    } else {
        Ok(())
    }
}

fn ensure_deadline(e: &Env, timestamp: u64) -> Result<(), AggregatorError> {
    let ledger_timestamp = e.ledger().timestamp();
    if ledger_timestamp >= timestamp {
        Err(AggregatorError::DeadlineExpired)
    } else {
        Ok(())
    }
}

fn check_initialized(e: &Env) -> Result<(), AggregatorError> {
    if is_initialized(e) {
        Ok(())
    } else {
        Err(AggregatorError::NotInitialized)
    }
}

fn check_admin(e: &Env) -> Result<(), AggregatorError> {
    let admin: Address = get_admin(&e)?;
    admin.require_auth();
    Ok(())
}

fn check_parameters(
    e: &Env,
    token_in: Address,
    token_out: Address,
    total_amount: i128,
    amount_0: i128,
    amount_1: i128,
    to: Address,
    deadline: u64,
    distribution: Vec<DexDistribution>,
) -> Result<(), AggregatorError> {
    check_initialized(e)?;
    check_nonnegative_amount(amount_0)?;
    check_nonnegative_amount(amount_1)?;
    to.require_auth();
    ensure_deadline(e, deadline)?;

    if distribution.len() > MAX_DISTRIBUTION_LENGTH {
        return Err(AggregatorError::DistributionLengthExceeded);
    }
    let total_amounts: i128 = distribution.iter().map(|dist| dist.amount).sum();
    // total amount should be ewqual to total amounts
    if total_amounts != total_amount {
        return Err(AggregatorError::ArithmeticError); // TODO change error
    }

    for dist in distribution {
        // each amount should be positive
        if dist.amount <= 0 {
            return Err(AggregatorError::NegativeNotAllowed); //change error
        }
        if dist.path.get(0) != Some(token_in.clone()) {
            return Err(AggregatorError::InvalidPath);
        }
        // check that all paths end with token_out
        if dist.path.last() != Some(token_out.clone()) {
            return Err(AggregatorError::InvalidPath);
        }
    }
    
    Ok(())
}

pub fn get_adapter_client(
    e: &Env,
    protocol_id: String,
) -> Result<SoroswapAggregatorAdapterClient, AggregatorError> {
    let adapter = get_adapter(&e, protocol_id.clone())?;
    if adapter.paused {
        return Err(AggregatorError::ProtocolPaused);
    }
    Ok(SoroswapAggregatorAdapterClient::new(&e, &adapter.address))
}

/*
    SOROSWAP AGGREGATOR SMART CONTRACT INTERFACE:
*/

pub trait SoroswapAggregatorTrait {
    /* ADMIN FUNCTIONS */

    /// Initializes the contract and sets the soroswap_router address.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `admin` - The address of the administrator.
    /// * `adapter_vec` - A vector containing the adapters to be initialized.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError::AlreadyInitialized` error if the contract is already initialized.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the initialization is successful.
    fn initialize(e: Env, admin: Address, adapter_vec: Vec<Adapter>)
        -> Result<(), AggregatorError>;

    /// Updates the adapters in the contract.
    ///
    /// This function overwrites any existing protocol address pairs if they exist.
    /// If an adapter does not exist, it will add it.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `adapter_vec` - A vector containing the adapters to be updated.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the adapters are successfully updated.
    fn update_adapters(e: Env, adapter_vec: Vec<Adapter>) -> Result<(), AggregatorError>;

    /// Removes an adapter from the contract.
    ///
    /// This function removes the adapter associated with the specified protocol ID.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `protocol_id` - The ID of the protocol whose adapter is to be removed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the adapter is successfully removed.
    fn remove_adapter(e: Env, protocol_id: String) -> Result<(), AggregatorError>;

    /// Sets the paused state of the protocol in the aggregator.
    ///
    /// # Argumentsnts
    /// * `e` - The runtime environment.t.
    /// * `protocol_id` - The ID of the protocol to set the paused state for.
    /// * `paused` - The boolean value indicating whether the protocol should be paused or not.
    ///
    /// # Returns
    /// Returns `Ok(())` if the operation is successful, otherwise returns an `AggregatorError`.
    fn set_pause(e: Env, protocol_id: String, paused: bool) -> Result<(), AggregatorError>;

    /// Upgrades the contract with new WebAssembly (WASM) code.
    ///
    /// This function updates the contract with new WASM code provided by the `new_wasm_hash`.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `new_wasm_hash` - The hash of the new WASM code to upgrade the contract to.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the upgrade is successful.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), AggregatorError>;

    /// Sets the `admin` address.
    ///
    /// # Arguments
    ///
    /// * `e` - An instance of the `Env` struct.
    /// * `new_admin` - The address to set as the new `admin`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Aggregator is not yet initialized or if the caller is not the existing `admin`.
    fn set_admin(e: Env, new_admin: Address) -> Result<(), AggregatorError>;

    /* SWAP FUNCTION */

    /// Swaps an exact amount of input tokens for output tokens across multiple DEXes.
    ///
    /// This function performs a swap operation where an exact amount of input tokens is exchanged for output tokens,
    /// distributed across multiple DEXes as specified by the `distribution` parameter.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `token_in` - The address of the input token.
    /// * `token_out` - The address of the output token.
    /// * `amount_in` - The exact amount of input tokens to be swapped.
    /// * `amount_out_min` - The minimum amount of output tokens expected to receive.
    /// * `distribution` - A vector specifying how the swap should be distributed across different DEXes.
    /// * `to` - The address to receive the output tokens.
    /// * `deadline` - The time by which the swap must be completed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if any of the following conditions are met:
    /// - The parameters are invalid.
    /// - The swap amounts calculation fails.
    /// - There is an arithmetic error.
    /// - The final output amount is less than the minimum expected amount.
    ///
    /// # Returns
    ///
    /// Returns a vector of vectors, where each inner vector contains the swap amounts for each DEX if the operation is successful.
    fn swap_exact_tokens_for_tokens(
        env: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<Vec<i128>>, AggregatorError>;

    /// Swaps tokens for an exact amount of output tokens across multiple DEXes.
    ///
    /// This function performs a swap operation where tokens are exchanged for an exact amount of output tokens,
    /// distributed across multiple DEXes as specified by the `distribution` parameter.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `token_in` - The address of the input token.
    /// * `token_out` - The address of the output token.
    /// * `amount_out` - The exact amount of output tokens to be received.
    /// * `amount_in_max` - The maximum amount of input tokens to be spent.
    /// * `distribution` - A vector specifying how the swap should be distributed across different DEXes.
    /// * `to` - The address to receive the output tokens.
    /// * `deadline` - The time by which the swap must be completed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if any of the following conditions are met:
    /// - The parameters are invalid.
    /// - The swap amounts calculation fails.
    /// - There is an arithmetic error.
    /// - The final input amount exceeds the maximum allowed.
    ///
    /// # Returns
    ///
    /// Returns a vector of vectors, where each inner vector contains the swap amounts for each DEX if the operation is successful.
    fn swap_tokens_for_exact_tokens(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_out: i128,
        amount_in_max: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<Vec<i128>>, AggregatorError>;

    /*  *** Read only functions: *** */

    /// Retrieves the administrator address of the contract.
    ///
    /// This function returns the current administrator address of the contract.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized.
    ///
    /// # Returns
    ///
    /// Returns the address of the current administrator if the operation is successful.
    fn get_admin(e: &Env) -> Result<Address, AggregatorError>;

    /// Retrieves the list of adapters registered in the contract.
    ///
    /// This function returns a vector containing all the adapters registered in the contract.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if there are issues retrieving adapters.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Adapter` objects if the operation is successful.
    fn get_adapters(e: &Env) -> Result<Vec<Adapter>, AggregatorError>;


    /// Retrieves the paused state of a specific protocol adapter.
    ///
    /// This function returns whether the adapter associated with the specified `protocol_id` is currently paused.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    /// * `protocol_id` - The ID of the protocol whose paused state is to be retrieved.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if there are issues retrieving the adapter or if the protocol ID is not found.
    ///
    /// # Returns
    ///
    /// Returns `true` if the adapter is paused, otherwise `false`.
    fn get_paused(e: &Env, protocol_id: String) -> Result<bool, AggregatorError>;

    /// Retrieves the version number of the contract.
    ///
    /// This function returns the version number of the contract. If the WebAssembly (WASM) code is updated,
    /// this number should be increased accordingly to reflect the new version.
    ///
    /// # Returns
    ///
    /// Returns the current version number of the contract as a `u32`.
    fn get_version() -> u32;
}

#[contract]
struct SoroswapAggregator;

#[contractimpl]
impl SoroswapAggregatorTrait for SoroswapAggregator {
    /* ADMIN FUNCTIONS */

    /// Initializes the contract and sets the soroswap_router address.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `admin` - The address of the administrator.
    /// * `adapter_vec` - A vector containing the adapters to be initialized.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError::AlreadyInitialized` error if the contract is already initialized.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the initialization is successful.
    fn initialize(
        e: Env,
        admin: Address,
        adapter_vec: Vec<Adapter>,
    ) -> Result<(), AggregatorError> {
        if is_initialized(&e) {
            return Err(AggregatorError::AlreadyInitialized);
        }
        admin.require_auth();

        for adapter in adapter_vec.iter() {
            put_adapter(&e, adapter);
        }

        set_admin(&e, admin.clone());

        // Mark the contract as initialized
        set_initialized(&e);
        event::initialized(&e, admin, adapter_vec);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Updates the adapters in the contract.
    ///
    /// This function overwrites any existing protocol address pairs if they exist.
    /// If an adapter does not exist, it will add it.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `adapter_vec` - A vector containing the adapters to be updated.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the adapters are successfully updated.
    fn update_adapters(e: Env, adapter_vec: Vec<Adapter>) -> Result<(), AggregatorError> {
        check_admin(&e)?;

        for adapter in adapter_vec.iter() {
            put_adapter(&e, adapter);
        }

        event::protocols_updated(&e, adapter_vec);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Removes an adapter from the contract.
    ///
    /// This function removes the adapter associated with the specified protocol ID.
    ///
    /// # Arguments
    ///
    /// * `e` - The environment in which the contract is running.
    /// * `protocol_id` - The ID of the protocol whose adapter is to be removed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the adapter is successfully removed.
    fn remove_adapter(e: Env, protocol_id: String) -> Result<(), AggregatorError> {
        check_admin(&e)?;

        remove_adapter(&e, protocol_id.clone());

        event::protocol_removed(&e, protocol_id);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Sets the paused state of the protocol in the aggregator.
    ///
    /// # Argumentsnts
    /// * `e` - The runtime environment.t.
    /// * `protocol_id` - The ID of the protocol to set the paused state for.
    /// * `paused` - The boolean value indicating whether the protocol should be paused or not.
    ///
    /// # Returns
    /// Returns `Ok(())` if the operation is successful, otherwise returns an `AggregatorError`.
    fn set_pause(e: Env, protocol_id: String, paused: bool) -> Result<(), AggregatorError> {
        check_admin(&e)?;

        set_pause_protocol(&e, protocol_id.clone(), paused)?;

        event::protocol_paused(&e, protocol_id, paused);
        extend_instance_ttl(&e);
        Ok(())
    }

    /// Sets a new administrator for the contract.
    ///
    /// This function updates the administrator of the contract to the specified `new_admin` address.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `new_admin` - The address of the new administrator.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the current admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation is successful.
    fn set_admin(e: Env, new_admin: Address) -> Result<(), AggregatorError> {
        check_admin(&e)?;

        let admin: Address = get_admin(&e)?;
        set_admin(&e, new_admin.clone());

        event::new_admin(&e, admin, new_admin);
        Ok(())
    }

    /// Upgrades the contract with new WebAssembly (WASM) code.
    ///
    /// This function updates the contract with new WASM code provided by the `new_wasm_hash`.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `new_wasm_hash` - The hash of the new WASM code to upgrade the contract to.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if the caller is not the admin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the upgrade is successful.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), AggregatorError> {
        check_admin(&e)?;

        e.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /// Swaps an exact amount of input tokens for output tokens across multiple DEXes.
    ///
    /// This function performs a swap operation where an exact amount of input tokens is exchanged for output tokens,
    /// distributed across multiple DEXes as specified by the `distribution` parameter.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `token_in` - The address of the input token.
    /// * `token_out` - The address of the output token.
    /// * `amount_in` - The exact amount of input tokens to be swapped.
    /// * `amount_out_min` - The minimum amount of output tokens expected to receive.
    /// * `distribution` - A vector specifying how the swap should be distributed across different DEXes.
    /// * `to` - The address to receive the output tokens.
    /// * `deadline` - The time by which the swap must be completed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if any of the following conditions are met:
    /// - The parameters are invalid.
    /// - The swap amounts calculation fails.
    /// - There is an arithmetic error.
    /// - The final output amount is less than the minimum expected amount.
    ///
    /// # Returns
    ///
    /// Returns a vector of vectors, where each inner vector contains the swap amounts for each DEX if the operation is successful.
    fn swap_exact_tokens_for_tokens(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        amount_out_min: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<Vec<i128>>, AggregatorError> {
        extend_instance_ttl(&e);
        check_parameters(
            &e,
            token_in.clone(),
            token_out.clone(),
            amount_in, //total amount
            amount_in,
            amount_out_min,
            to.clone(),
            deadline,
            distribution.clone(),
        )?;

        
        let mut swap_responses: Vec<Vec<i128>> = Vec::new(&e);

        // Check initial out balance
        let initial_token_out_balance = TokenClient::new(&e, &token_out).balance(&to);

        for (index, dist) in distribution.iter().enumerate() {
            let protocol_id = dist.protocol_id;
            let adapter_client = get_adapter_client(&e, protocol_id.clone())?;
            let response = adapter_client.swap_exact_tokens_for_tokens(
                &dist.amount, // amount_in
                &0, // amount_out_min: amount out min per protocol will allways be 0, we will then compare the toal amoiunt out
                &dist.path,
                &to,
                &deadline,
            );
            swap_responses.push_back(response);
        }

        // Check final token out balance
        let final_token_out_balance = TokenClient::new(&e, &token_out).balance(&to);
        let final_amount_out = final_token_out_balance
            .checked_sub(initial_token_out_balance)
            .ok_or(AggregatorError::ArithmeticError)?;

        if final_amount_out < amount_out_min {
            return Err(AggregatorError::InsufficientOutputAmount);
        }

        event::swap(
            &e,
            token_in,
            token_out,
            amount_in,
            final_amount_out,
            distribution,
            to,
        );

        Ok(swap_responses)
    }

    /// Swaps tokens for an exact amount of output tokens across multiple DEXes.
    ///
    /// This function performs a swap operation where tokens are exchanged for an exact amount of output tokens,
    /// distributed across multiple DEXes as specified by the `distribution` parameter.
    ///
    /// # Arguments
    ///
    /// * `e` - The runtime environment.
    /// * `token_in` - The address of the input token.
    /// * `token_out` - The address of the output token.
    /// * `amount_out` - The exact amount of output tokens to be received.
    /// * `amount_in_max` - The maximum amount of input tokens to be spent.
    /// * `distribution` - A vector specifying how the swap should be distributed across different DEXes.
    /// * `to` - The address to receive the output tokens.
    /// * `deadline` - The time by which the swap must be completed.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if any of the following conditions are met:
    /// - The parameters are invalid.
    /// - The swap amounts calculation fails.
    /// - There is an arithmetic error.
    /// - The final input amount exceeds the maximum allowed.
    ///
    /// # Returns
    ///
    /// Returns a vector of vectors, where each inner vector contains the swap amounts for each DEX if the operation is successful.
    fn swap_tokens_for_exact_tokens(
        e: Env,
        token_in: Address,
        token_out: Address,
        amount_out: i128,
        amount_in_max: i128,
        distribution: Vec<DexDistribution>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<Vec<i128>>, AggregatorError> {
        extend_instance_ttl(&e);
        check_parameters(
            &e,
            token_in.clone(),
            token_out.clone(),
            amount_out, //total amount
            amount_out,
            amount_in_max,
            to.clone(),
            deadline,
            distribution.clone(),
        )?;

        
        let mut swap_responses: Vec<Vec<i128>> = Vec::new(&e);

        // Check initial in balance
        let initial_token_in_balance = TokenClient::new(&e, &token_in).balance(&to);

        for (index, dist) in distribution.iter().enumerate() {
            let protocol_id = dist.protocol_id;
            let adapter_client = get_adapter_client(&e, protocol_id.clone())?;
            let response = adapter_client.swap_tokens_for_exact_tokens(
                &dist.amount, // amount_out
                &i128::MAX,   // amount_in_max
                &dist.path,   //path
                &to,          //to
                &deadline,    //deadline
            );
            swap_responses.push_back(response);
        }
        // Check final token in balance, so we did not spend more than amount_in_max
        let final_token_in_balance = TokenClient::new(&e, &token_in).balance(&to);
        let final_amount_in = initial_token_in_balance
            .checked_sub(final_token_in_balance)
            .ok_or(AggregatorError::ArithmeticError)?;

        if final_amount_in > amount_in_max {
            return Err(AggregatorError::ExcessiveInputAmount);
        }
        event::swap(
            &e,
            token_in,
            token_out,
            final_amount_in,
            amount_out,
            distribution,
            to,
        );
        Ok(swap_responses)
    }

    /*  *** Read only functions: *** */

    /// Retrieves the administrator address of the contract.
    ///
    /// This function returns the current administrator address of the contract.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized.
    ///
    /// # Returns
    ///
    /// Returns the address of the current administrator if the operation is successful.
    fn get_admin(e: &Env) -> Result<Address, AggregatorError> {
        check_initialized(&e)?;
        Ok(get_admin(&e)?)
    }

    /// Retrieves the list of adapters registered in the contract.
    ///
    /// This function returns a vector containing all the adapters registered in the contract.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if the contract is not initialized or if there are issues retrieving adapters.
    ///
    /// # Returns
    ///
    /// Returns a vector of `Adapter` objects if the operation is successful.
    fn get_adapters(e: &Env) -> Result<Vec<Adapter>, AggregatorError> {
        check_initialized(&e)?;

        let protocol_ids = get_protocol_ids(e);
        let mut adapter_vec = Vec::new(e);

        // Iterate over each protocol ID and collect their adapter object
        for protocol_id in protocol_ids.iter() {
            if has_adapter(e, protocol_id.clone()) {
                let adapter = get_adapter(e, protocol_id.clone())?;
                adapter_vec.push_back(adapter);
            }
        }

        Ok(adapter_vec)
    }

    /// Retrieves the paused state of a specific protocol adapter.
    ///
    /// This function returns whether the adapter associated with the specified `protocol_id` is currently paused.
    ///
    /// # Arguments
    ///
    /// * `e` - A reference to the runtime environment.
    /// * `protocol_id` - The ID of the protocol whose paused state is to be retrieved.
    ///
    /// # Errors
    ///
    /// Returns an `AggregatorError` if there are issues retrieving the adapter or if the protocol ID is not found.
    ///
    /// # Returns
    ///
    /// Returns `true` if the adapter is paused, otherwise `false`.
    fn get_paused(e: &Env, protocol_id: String) -> Result<bool, AggregatorError> {
        let adapter = get_adapter(e, protocol_id)?;
        Ok(adapter.paused)
    }

    /// Retrieves the version number of the contract.
    ///
    /// This function returns the version number of the contract. If the WebAssembly (WASM) code is updated,
    /// this number should be increased accordingly to reflect the new version.
    ///
    /// # Returns
    ///
    /// Returns the current version number of the contract as a `u32`.
    fn get_version() -> u32 {
        1
    }
}
