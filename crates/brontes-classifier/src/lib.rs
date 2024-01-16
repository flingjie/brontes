use std::fmt::Debug;

use alloy_primitives::Log;
use brontes_database_libmdbx::implementation::tx::LibmdbxTx;
use brontes_pricing::types::PoolUpdate;
use reth_db::mdbx::RO;
use reth_primitives::{Address, Bytes};

pub mod classifier;
pub use classifier::*;

pub mod bindings;
use bindings::*;

#[cfg(feature = "tests")]
pub mod test_utils;

mod action_classifiers;
pub use action_classifiers::*;
mod discovery_classifiers;
use alloy_sol_types::{sol, SolInterface};
use brontes_types::normalized_actions::Actions;

sol!(UniswapV2, "./abis/UniswapV2.json");
sol!(SushiSwapV2, "./abis/SushiSwapV2.json");
sol!(UniswapV3, "./abis/UniswapV3.json");
sol!(SushiSwapV3, "./abis/SushiSwapV3.json");
sol!(CurveCryptoSwap, "./abis/CurveCryptoSwap.json");
sol!(AaveV2, "./abis/AaveV2Pool.json");
sol!(AaveV3, "./abis/AaveV3Pool.json");
sol!(UniswapX, "./abis/UniswapXExclusiveDutchOrderReactor.json");

sol!(UniswapV2Factory, "./abis/UniswapV2Factory.json");
sol!(UniswapV3Factory, "./abis/UniswapV3Factory.json");
sol!(CurveV1MetapoolFactory, "./abis/CurveMetapoolFactoryV1.json");
sol!(CurveV2MetapoolFactory, "./abis/CurveMetapoolFactoryV2.json");
sol!(CurvecrvUSDFactory, "./abis/CurveCRVUSDFactory.json");
sol!(CurveCryptoSwapFactory, "./abis/CurveCryptoSwapFactory.json");
sol!(CurveTriCryptoFactory, "./abis/CurveTriCryptoFactory.json");

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    function name() public view returns (string);
    function symbol() public view returns (string);
    function decimals() public view returns (uint8);
    function totalSupply() public view returns (uint256);
}

pub trait ActionCollection: Sync + Send {
    fn dispatch(
        &self,
        sig: &[u8],
        trace_index: u64,
        data: StaticReturnBindings,
        return_data: Bytes,
        from_address: Address,
        target_address: Address,
        logs: &Vec<Log>,
        db_tx: &LibmdbxTx<RO>,
        block: u64,
        tx_idx: u64,
    ) -> Option<(PoolUpdate, Actions)>;
}

/// implements the above trait for decoding on the different binding enums
#[macro_export]
macro_rules! impl_decode_sol {
    ($enum_name:ident, $inner_type:path) => {
        impl TryDecodeSol for $enum_name {
            type DecodingType = $inner_type;

            fn try_decode(call_data: &[u8]) -> Result<Self::DecodingType, alloy_sol_types::Error> {
                Self::DecodingType::abi_decode(call_data, false)
            }
        }
    };
}

pub trait IntoAction: Debug + Send + Sync {
    fn get_signature(&self) -> [u8; 4];

    fn decode_trace_data(
        &self,
        index: u64,
        data: StaticReturnBindings,
        return_data: Bytes,
        from_address: Address,
        target_address: Address,
        logs: &Vec<Log>,
        db_tx: &LibmdbxTx<RO>,
    ) -> Option<Actions>;
}
