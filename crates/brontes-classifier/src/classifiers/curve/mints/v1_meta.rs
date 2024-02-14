use brontes_macros::action_impl;
use brontes_pricing::Protocol;
use brontes_types::{
    normalized_actions::NormalizedMint, structured_trace::CallInfo, ToScaledRational,
};

// couldn't find a V1 metapool calling this
action_impl!(
    Protocol::CurveV1MetapoolImpl,
    crate::CurveV1MetapoolImpl::add_liquidity_0Call,
    Mint,
    [..AddLiquidity],
    logs: true,
    |
    info: CallInfo,
    log: CurveV1MetapoolImpladd_liquidity_0CallLogs,
    db_tx: &DB|{
        let log = log.AddLiquidity_field;

        let details = db_tx.get_protocol_details(info.from_address)?;
        let token_addrs = vec![details.token0, details.curve_lp_token.expect("Expected curve_lp_token, found None")];
        let protocol = details.protocol;

        let amounts = log.token_amounts;
        let (tokens, token_amts): (Vec<_>, Vec<_>) = token_addrs.into_iter().enumerate().map(|(i, t)|
        {
            let token = db_tx.try_fetch_token_info(t)?;
            let decimals = token.decimals;
            Ok((token, amounts[i].to_scaled_rational(decimals)))
        }
        ).collect::<eyre::Result<Vec<_>>>()?.into_iter().unzip();


        Ok(NormalizedMint {
            protocol,
            trace_index: info.trace_idx,
            pool: info.from_address,
            from: info.msg_sender,
            recipient: info.msg_sender,
            token: tokens,
            amount: token_amts,
        })
    }
);

action_impl!(
    Protocol::CurveV1MetapoolImpl,
    crate::CurveV1MetapoolImpl::add_liquidity_1Call,
    Mint,
    [..AddLiquidity],
    logs: true,
    |
    info: CallInfo,
    log: CurveV1MetapoolImpladd_liquidity_1CallLogs,
    db_tx: &DB|{
        let log = log.AddLiquidity_field;

        let details = db_tx.get_protocol_details(info.from_address)?;
        let token_addrs = vec![details.token0, details.curve_lp_token.expect("Expected curve_lp_token, found None")];
        let protocol = details.protocol;

        let amounts = log.token_amounts;
        let (tokens, token_amts): (Vec<_>, Vec<_>) = token_addrs.into_iter().enumerate().map(|(i, t)|
        {
            let token = db_tx.try_fetch_token_info(t)?;
            let decimals = token.decimals;
            Ok((token, amounts[i].to_scaled_rational(decimals)))
        }
        ).collect::<eyre::Result<Vec<_>>>()?.into_iter().unzip();


        Ok(NormalizedMint {
            protocol,
            trace_index: info.trace_idx,
            pool: info.from_address,
            from: info.msg_sender,
            recipient: info.msg_sender,
            token: tokens,
            amount: token_amts,
        })
    }
);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{hex, Address, B256, U256};
    use brontes_classifier::test_utils::ClassifierTestUtils;
    use brontes_types::{
        db::token_info::{TokenInfo, TokenInfoWithAddress},
        normalized_actions::Actions,
        Node, NodeData, ToScaledRational, TreeSearchArgs,
    };

    use super::*;

    #[brontes_macros::test]
    async fn test_curve_v1_metapool_add_liquidity0() {
        let classifier_utils = ClassifierTestUtils::new().await;
        classifier_utils.ensure_protocol(
            Protocol::CurveV1MetaPool,
            Address::new(hex!("A77d09743F77052950C4eb4e6547E9665299BecD")),
            Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            Address::new(hex!("6B175474E89094C44Da98b954EedeAC495271d0F")),
            Some(Address::new(hex!(
                "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            ))),
            Some(Address::new(hex!(
                "dAC17F958D2ee523a2206206994597C13D831ec7"
            ))),
            None,
            Some(Address::new(hex!(
                "6c3F90f043a72FA612cbac8115EE7e52BDe6E490"
            ))),
        );

        let mint = B256::from(hex!(
            "00723506614af2c7a56057b0bd70c263198019e58aac8c10b337d2391996ea0f"
        ));

        let token0 = TokenInfoWithAddress {
            address: Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            inner: TokenInfo {
                decimals: 18,
                symbol: "T".to_string(),
            },
        };

        let token1 = TokenInfoWithAddress {
            address: Address::new(hex!("6c3f90f043a72fa612cbac8115ee7e52bde6e490")),
            inner: TokenInfo {
                decimals: 18,
                symbol: "3Crv".to_string(),
            },
        };

        classifier_utils.ensure_token(token0.clone());
        classifier_utils.ensure_token(token1.clone());

        let eq_action = Actions::Mint(NormalizedMint {
            protocol: Protocol::CurveBasePool,
            trace_index: 0,
            from: Address::new(hex!("1a734e9bDa6893915928eE8edBA75cA17536d385")),
            recipient: Address::new(hex!("1a734e9bDa6893915928eE8edBA75cA17536d385")),
            pool: Address::new(hex!("A77d09743F77052950C4eb4e6547E9665299BecD")),
            token: vec![token0, token1],
            amount: vec![
                U256::from(1000000000000000000000 as u128).to_scaled_rational(18),
                U256::from(1000000000000000000000 as u128).to_scaled_rational(8),
            ],
        });

        let search_fn = |node: &Node, data: &NodeData<Actions>| TreeSearchArgs {
            collect_current_node: data
                .get_ref(node.data)
                .map(|s| s.is_mint())
                .unwrap_or_default(),
            child_node_to_collect: node
                .get_all_sub_actions()
                .iter()
                .filter_map(|d| data.get_ref(*d))
                .any(|action| action.is_mint()),
        };

        classifier_utils
            .contains_action(mint, 0, eq_action, search_fn)
            .await
            .unwrap();
    }

    #[brontes_macros::test]
    async fn test_curve_v1_metapool_add_liquidity1() {
        let classifier_utils = ClassifierTestUtils::new().await;
        classifier_utils.ensure_protocol(
            Protocol::CurveV1MetaPool,
            Address::new(hex!("A77d09743F77052950C4eb4e6547E9665299BecD")),
            Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            Address::new(hex!("6B175474E89094C44Da98b954EedeAC495271d0F")),
            Some(Address::new(hex!(
                "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            ))),
            Some(Address::new(hex!(
                "dAC17F958D2ee523a2206206994597C13D831ec7"
            ))),
            None,
            Some(Address::new(hex!(
                "6c3F90f043a72FA612cbac8115EE7e52BDe6E490"
            ))),
        );

        let mint = B256::from(hex!(
            "00723506614af2c7a56057b0bd70c263198019e58aac8c10b337d2391996ea0f"
        ));

        let token0 = TokenInfoWithAddress {
            address: Address::new(hex!("6967299e9F3d5312740Aa61dEe6E9ea658958e31")),
            inner: TokenInfo {
                decimals: 18,
                symbol: "T".to_string(),
            },
        };

        let token1 = TokenInfoWithAddress {
            address: Address::new(hex!("6c3f90f043a72fa612cbac8115ee7e52bde6e490")),
            inner: TokenInfo {
                decimals: 18,
                symbol: "3Crv".to_string(),
            },
        };

        classifier_utils.ensure_token(token0.clone());
        classifier_utils.ensure_token(token1.clone());

        let eq_action = Actions::Mint(NormalizedMint {
            protocol: Protocol::CurveBasePool,
            trace_index: 0,
            from: Address::new(hex!("DaD7ef2EfA3732892d33aAaF9B3B1844395D9cbE")),
            recipient: Address::new(hex!("DaD7ef2EfA3732892d33aAaF9B3B1844395D9cbE")),
            pool: Address::new(hex!("7fC77b5c7614E1533320Ea6DDc2Eb61fa00A9714")),
            token: vec![token0, token1],
            amount: vec![
                U256::from(1000000000000000000000 as u128).to_scaled_rational(18),
                U256::from(1000000000000000000000 as u128).to_scaled_rational(8),
            ],
        });

        let search_fn = |node: &Node, data: &NodeData<Actions>| TreeSearchArgs {
            collect_current_node: data
                .get_ref(node.data)
                .map(|s| s.is_mint())
                .unwrap_or_default(),
            child_node_to_collect: node
                .get_all_sub_actions()
                .iter()
                .filter_map(|d| data.get_ref(*d))
                .any(|action| action.is_mint()),
        };

        classifier_utils
            .contains_action(mint, 0, eq_action, search_fn)
            .await
            .unwrap();
    }
}
