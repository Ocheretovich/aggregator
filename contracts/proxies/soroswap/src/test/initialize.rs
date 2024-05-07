use soroban_sdk::{Address, testutils::Address as _};

use crate::error::CombinedProxyError;
use crate::test::{SoroswapAggregatorProxyTest};


#[test]
fn test_initialize_and_get_admin() {
    let test = SoroswapAggregatorProxyTest::setup();

    //Initialize aggregator proxy
    // let initialize_aggregator_addresses = create_protocols_addresses(&test);

    test.proxy_contract.initialize(&test.admin, &test.router_contract.address);

    let admin = test.proxy_contract.get_admin();
    assert_eq!(admin, test.admin);
}

// #[test]
// fn test_get_admin_not_yet_initialized() {
//     let test = SoroswapAggregatorTest::setup();
//     let result = test.aggregator_contract.try_get_admin();

//     assert_eq!(result, Err(Ok(CombinedAggregatorError::AggregatorNotInitialized)));
// }

// #[test]
// fn test_initialize_twice() {
//     let test = SoroswapAggregatorTest::setup();
    
//     //Initialize aggregator
//     let initialize_aggregator_addresses = create_protocols_addresses(&test);
//     test.aggregator_contract.initialize(&test.admin, &initialize_aggregator_addresses);

//     let result_second_init = test.aggregator_contract.try_initialize(&test.admin, &initialize_aggregator_addresses);
//     assert_eq!(
//         result_second_init,
//         (Err(Ok(CombinedAggregatorError::AggregatorInitializeAlreadyInitialized)))
//     );
// }
