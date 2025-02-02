use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgSwapExactAmountIn, QueryTotalPoolLiquidityRequest, QueryTotalPoolLiquidityResponse,
    SwapAmountInRoute,
};

use cosmwasm_std::{Addr, Coin, Deps};

use osmo_bindings::{OsmosisMsg, OsmosisQuerier, OsmosisQuery};

use crate::{
    state::{ROUTING_TABLE, STATE},
    ContractError,
};

pub fn check_is_contract_owner(deps: Deps, sender: Addr) -> Result<(), ContractError> {
    let config = STATE.load(deps.storage).unwrap();
    if config.owner != sender {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_pool_route(
    deps: Deps,
    input_denom: String,
    output_denom: String,
    pool_route: Vec<SwapAmountInRoute>,
) -> Result<(), ContractError> {
    let mut current_denom = input_denom;

    // make sure that this route actually works
    for route_part in &pool_route {
        let request = QueryTotalPoolLiquidityRequest {
            pool_id: route_part.pool_id,
        }
        .into();

        let res: QueryTotalPoolLiquidityResponse = deps.querier.query(&request)?;

        if !res.liquidity.iter().any(|coin| coin.denom == current_denom) {
            return Result::Err(ContractError::InvalidPoolRoute {});
        }

        current_denom = route_part.token_out_denom.clone();
    }

    // make sure the final route output asset is the same as the expected output_denom
    if current_denom != output_denom {
        return Result::Err(ContractError::InvalidPoolRoute {});
    }

    Ok(())
}

pub fn generate_swap_msg(
    deps: Deps,
    sender: Addr,
    input_token: Coin,
    min_output_token: Coin,
) -> Result<MsgSwapExactAmountIn, ContractError> {
    // get trade route
    let route = ROUTING_TABLE.load(deps.storage, (&input_token.denom, &min_output_token.denom))?;

    // convert input coin to sdk coin stuct
    let sdk_input_coin = osmosis_std::types::cosmos::base::v1beta1::Coin {
        denom: input_token.denom,
        amount: input_token.amount.to_string(),
    };

    Ok(MsgSwapExactAmountIn {
        sender: sender.into_string(),
        routes: route,
        token_in: Some(sdk_input_coin),
        token_out_min_amount: min_output_token.amount.to_string(),
    })
}
