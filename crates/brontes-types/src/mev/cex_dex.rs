use std::fmt::Debug;

use ::serde::ser::{Serialize, SerializeStruct, Serializer};
use itertools::Itertools;
use malachite::Rational;
use reth_primitives::B256;
use serde::Deserialize;
use serde_with::serde_as;
use sorella_db_databases::clickhouse::{fixed_string::FixedString, DbRow};

use super::{Mev, MevType};
use crate::{db::cex::CexExchange, normalized_actions::ClickhouseVecNormalizedSwap, Protocol};
#[allow(unused_imports)]
use crate::{
    display::utils::{display_sandwich, print_mev_type_header},
    normalized_actions::{NormalizedBurn, NormalizedLiquidation, NormalizedMint, NormalizedSwap},
    serde_primitives::vec_fixed_string,
    GasDetails,
};

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub struct CexDex {
    pub tx_hash:     B256,
    pub swaps:       Vec<NormalizedSwap>,
    pub prices:      Vec<StatArbDetails>,
    pub gas_details: GasDetails,
}

impl Mev for CexDex {
    fn mev_type(&self) -> MevType {
        MevType::CexDex
    }

    fn total_gas_paid(&self) -> u128 {
        self.gas_details.gas_paid()
    }

    fn total_priority_fee_paid(&self, base_fee: u128) -> u128 {
        self.gas_details.priority_fee_paid(base_fee)
    }

    fn mev_transaction_hashes(&self) -> Vec<B256> {
        vec![self.tx_hash]
    }

    fn bribe(&self) -> u128 {
        self.gas_details.coinbase_transfer.unwrap_or(0)
    }
}

impl Serialize for CexDex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser_struct = serializer.serialize_struct("CexDex", 34)?;

        ser_struct.serialize_field("tx_hash", &FixedString::from(format!("{:?}", self.tx_hash)))?;

        let swaps: ClickhouseVecNormalizedSwap = self.swaps.clone().into();

        ser_struct.serialize_field("swaps.trace_idx", &swaps.trace_index)?;
        ser_struct.serialize_field("swaps.from", &swaps.from)?;
        ser_struct.serialize_field("swaps.recipient", &swaps.recipient)?;
        ser_struct.serialize_field("swaps.pool", &swaps.pool)?;
        ser_struct.serialize_field("swaps.token_in", &swaps.token_in)?;
        ser_struct.serialize_field("swaps.token_out", &swaps.token_out)?;
        ser_struct.serialize_field("swaps.amount_in", &swaps.amount_in)?;
        ser_struct.serialize_field("swaps.amount_out", &swaps.amount_out)?;

        let gas_details = (
            self.gas_details.coinbase_transfer,
            self.gas_details.priority_fee,
            self.gas_details.gas_used,
            self.gas_details.effective_gas_price,
        );

        ser_struct.serialize_field("gas_details", &(gas_details))?;

        ser_struct.serialize_field(
            "prices.address",
            &(self
                .prices_address
                .iter()
                .map(|addr| FixedString::from(format!("{:?}", addr)))
                .collect_vec()),
        )?;
        ser_struct.serialize_field("prices.price", &(self.prices_price))?;

        ser_struct.end()
    }
}

impl DbRow for CexDex {
    const COLUMN_NAMES: &'static [&'static str] = &[
        "tx_hash",
        "swaps.trace_idx",
        "swaps.from",
        "swaps.recipient",
        "swaps.pool",
        "swaps.token_in",
        "swaps.token_out",
        "swaps.amount_in",
        "swaps.amount_out",
        "gas_details",
    ];
}

#[serde_as]
#[derive(Debug, Deserialize, Clone, Default)]
pub struct StatArbDetails {
    pub cex_exchange:   CexExchange,
    pub cex_price:      Rational,
    pub dex_exchange:   Protocol,
    pub dex_price:      Rational,
    // Arbitrage profit considering both CEX and DEX swap fees, before applying gas fees
    pub profit_pre_gas: Rational,
}
