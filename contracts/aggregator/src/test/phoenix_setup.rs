
// For Phoenix
mod phoenix_adapter {
    soroban_sdk::contractimport!(
        file =
            "../target/wasm32-unknown-unknown/release/phoenix_adapter.optimized.wasm"
    );
    pub type SoroswapAggregatorAdapterForPhoenixClient<'a> = Client<'a>;
}
use phoenix_adapter::SoroswapAggregatorAdapterForPhoenixClient;
use crate::test::install_token_wasm;
// Adapter for phoenix
pub fn create_phoenix_adapter<'a>(e: &Env) -> SoroswapAggregatorAdapterForPhoenixClient<'a> {
    let adapter_address = &e.register_contract_wasm(None, phoenix_adapter::WASM);
    let adapter = SoroswapAggregatorAdapterForPhoenixClient::new(e, adapter_address);
    adapter
}


// #![cfg(test)]
// extern crate std;
use soroban_sdk::{
    vec,
    // IntoVal,
    String,
    Env, 
    Bytes,
    BytesN, 
    Address, 
    testutils::{
        arbitrary::std,
        Address as _,
    },
};

/* *************  PHOENIX FACTORY  *************  */

#[allow(clippy::too_many_arguments)]
pub mod factory {
    soroban_sdk::contractimport!(
        file = "../adapters/phoenix/phoenix_contracts/phoenix_factory.wasm"
    );
}
use factory::{LiquidityPoolInitInfo, StakeInitInfo, TokenInitInfo};

pub fn deploy_factory_contract(e: &Env, admin: & Address) -> Address {
    let factory_wasm = e.deployer().upload_contract_wasm(factory::WASM);
    let salt = Bytes::new(&e.clone());
    let salt = e.crypto().sha256(&salt);

    e.deployer().with_address(admin.clone(), salt).deploy(factory_wasm)
}

pub use factory::Client as PhoenixFactory;

/* *************  MULTIHOP  *************  */
#[allow(clippy::too_many_arguments)]
pub mod multihop {
    soroban_sdk::contractimport!(file = "../adapters/phoenix/phoenix_contracts/phoenix_multihop.wasm");
    pub type MultihopClient<'a> = Client<'a>;
}
pub use multihop::MultihopClient; 

pub fn install_multihop_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../adapters/phoenix/phoenix_contracts/phoenix_multihop.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}
pub fn deploy_multihop_contract<'a>(
    env: &Env,
    admin: impl Into<Option<Address>>,
    factory: &Address,
) -> MultihopClient<'a> {
    let admin = admin.into().unwrap_or(Address::generate(env));

    let multihop_address = &env.register_contract_wasm(None, multihop::WASM);
    let multihop = MultihopClient::new(env, multihop_address); 

    multihop.initialize(&admin, factory);
    multihop
}

/* *************  LP CONTRACT  *************  */

#[allow(clippy::too_many_arguments)]
pub mod lp_contract {
    soroban_sdk::contractimport!(
        file = "../adapters/phoenix/phoenix_contracts/phoenix_pool.wasm"
    );
}

pub fn install_lp_contract(env: &Env) -> BytesN<32> {
    env.deployer().upload_contract_wasm(lp_contract::WASM)
}


/* *************  STAKE  *************  */

#[allow(clippy::too_many_arguments)]
pub fn install_stake_wasm(env: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "../adapters/phoenix/phoenix_contracts/phoenix_stake.wasm"
    );
    env.deployer().upload_contract_wasm(WASM)
}



pub fn deploy_and_initialize_factory<'a>(env: &Env, admin: Address) -> PhoenixFactory<'a> {
    let factory_addr = deploy_factory_contract(&env, &admin.clone());
    let factory_client = PhoenixFactory::new(env, &factory_addr);
    let multihop_wasm_hash = install_multihop_wasm(env);
    let whitelisted_accounts = vec![env, admin.clone()];

    let lp_wasm_hash = install_lp_contract(env);
    let stake_wasm_hash = install_stake_wasm(env);
    let token_wasm_hash = install_token_wasm(env);

    factory_client.initialize(
        &admin.clone(),
        &multihop_wasm_hash,
        &lp_wasm_hash,
        &stake_wasm_hash,
        &token_wasm_hash,
        &whitelisted_accounts,
        &10u32,
    );
    factory_client
}

#[allow(clippy::too_many_arguments)]
pub fn deploy_and_initialize_lp(
    env: &Env,
    factory: &PhoenixFactory,
    admin: Address,
    mut token_a: Address,
    mut token_a_amount: i128,
    mut token_b: Address,
    mut token_b_amount: i128,
    fees: Option<i64>,
) {
    // 2. create liquidity pool from factory

    if token_b < token_a {
        std::mem::swap(&mut token_a, &mut token_b);
        std::mem::swap(&mut token_a_amount, &mut token_b_amount);
    }

    let token_init_info = TokenInitInfo {
        token_a: token_a.clone(),
        token_b: token_b.clone(),
    };
    let stake_init_info = StakeInitInfo {
        min_bond: 10i128,
        min_reward: 5i128,
        manager: Address::generate(env),
        max_complexity: 10u32,
    };

    let lp_init_info = LiquidityPoolInitInfo {
        admin: admin.clone(),
        fee_recipient: admin.clone(),
        max_allowed_slippage_bps: 5000,
        max_allowed_spread_bps: 500,
        swap_fee_bps: fees.unwrap_or(0i64),
        max_referral_bps: 5_000,
        token_init_info,
        stake_init_info,
    };

    let lp = factory.create_liquidity_pool(
        &admin.clone(),
        &lp_init_info,
        &String::from_str(env, "Pool"),
        &String::from_str(env, "PHO/XLM"),
    );

    let lp_client = lp_contract::Client::new(env, &lp);
    lp_client.provide_liquidity(
        &admin.clone(),
        &Some(token_a_amount),
        &None,
        &Some(token_b_amount),
        &None,
        &None::<i64>,
    );
}
