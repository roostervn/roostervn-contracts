#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, Binary, CosmosMsg, Deps, DepsMut, Env, 
    MessageInfo, Response, StdResult, Order, Querier, Storage, WasmMsg, SubMsg,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use cw2::set_contract_version;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use std::str::from_utf8;

use crate::package::{ContractInfoResponse, OfferingResponse, QueryOfferingResult};
use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InitMsg, InstantiateMsg, QueryMsg, HandleMsg, SellNft, BuyNft};
use crate::state::{State, STATE, CONTRACT_INFO, OFFERINGS, Offering, increment_offerings};



// version info for migration info
const CONTRACT_NAME: &str = "crates.io:marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
// Instantiate 
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let info = ContractInfoResponse { name: msg.name};
    CONTRACT_INFO.save(deps.storage, &info)?;
    Ok(Response::default())
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
// Declare a custom Error variant for the ones where you will want to use of of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<Response, ContractError> {
    match msg {
        HandleMsg::WithdrawNft { offering_id } => try_withdraw(deps, info, offering_id),
        HandleMsg::Receive(msg) => try_receive(deps, info, msg),
        HandleMsg::ReceiveNft(msg) => try_receive_nft(deps, info, msg),
    }
}

// =================================== Message Handlers ========================================

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
pub fn try_receive(
    deps: DepsMut,
    info: MessageInfo,
    rcv_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: BuyNft = from_binary(&rcv_msg.msg)?;

    // check if offering exists
    let off = OFFERINGS.load(deps.storage, &msg.offering_id)?;

    // chek for enough coins
    if rcv_msg.amount < off.list_price.amount {
        return Err(ContractError::InsufficientFunds {});
    }

    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::Transfer {
        recipient: off.seller.clone().into_string(),
        amount: rcv_msg.amount,
    };
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: info.sender.clone().into_string(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    };
    
    // create transfer cw721 msg
    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: rcv_msg.sender.clone(),
        token_id: off.token_id.clone(),
    };
    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: off.contract_addr.clone().into_string(),
        msg: to_binary(&transfer_cw721_msg)?,
        funds: vec![],
    };

    // if everything is fine transfer cw20 to seller
    let cw20_transfer_cosmos_msg: CosmosMsg = exec_cw20_transfer.into();
    // transfer nft to owner
    let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();

    let cw20_submsg = SubMsg::new(cw20_transfer_cosmos_msg);
    let cw721_submsg = SubMsg::new(cw721_transfer_cosmos_msg);

    let cosmos_msgs = vec![cw20_submsg, cw721_submsg];

    // delete offering
    OFFERINGS.remove(deps.storage, &msg.offering_id);

    let price_string = format!("{} {}", rcv_msg.amount, info.sender);

    Ok(Response::new()
        .add_attribute("action", "buy_nft")
        .add_attribute("buyer", rcv_msg.sender)
        .add_attribute("seller", off.seller)
        .add_attribute("paid_price", price_string)
        .add_attribute("token_id", off.token_id)
        .add_attribute("contract_addr", off.contract_addr)
        .add_submessages(cosmos_msgs)
    )
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
pub fn try_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: SellNft = from_binary(&rcv_msg.msg)?;

    // check same token_id from same original contract is already on sale
    // get OFFERING_COUNT
    let id = increment_offerings(deps.storage)?.to_string();

    // save Offering
    let token_id = rcv_msg.token_id.to_string();
    let off = Offering::<String> {
        owner: deps.api.addr_validate(&rcv_msg.sender)?,
        contract_addr: info.sender.clone(),
        token_id: rcv_msg.token_id,
        seller: deps.api.addr_validate(&rcv_msg.sender)?,
        list_price: msg.list_price.clone(),
        extension: format!("Offer {} from {}", token_id, deps.api.addr_validate(&rcv_msg.sender)?),
    };


    OFFERINGS.save(deps.storage, &id, &off)?;

    let price_string = format!("{} {}", msg.list_price.amount, msg.list_price.address);

    Ok(Response::new()
        .add_attribute("action", "sell_nft")
        .add_attribute("original_contract", info.sender)
        .add_attribute("owner", off.owner)
        .add_attribute("seller", off.seller)
        .add_attribute("list_price", price_string)
        .add_attribute("token_id", off.token_id)
    )
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
pub fn try_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    offering_id: String,
) -> Result<Response, ContractError> {
    // check if token_id is currency sold by the requesting address
    let off = OFFERINGS.load(deps.storage, &offering_id)?;
    if off.seller == info.sender.clone() {
        // transfer token back to original owner
        let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
            recipient: off.seller.clone().to_string(),
            token_id: off.token_id.clone(),
        };

        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: off.contract_addr.clone().into_string(),
            msg: to_binary(&transfer_cw721_msg)?,
            funds: vec![],
        };

        let cw721_transfer_cosmos_msg: CosmosMsg = exec_cw721_transfer.into();
        let cw721_submsg = SubMsg::new(cw721_transfer_cosmos_msg);

        OFFERINGS.remove(deps.storage, &offering_id);

        return Ok(Response::new()
            .add_attribute("action", "withdraw_nft")
            .add_attribute("seller", info.sender)
            .add_attribute("offering_id", offering_id)
            .add_submessage(cw721_submsg)
        );
    }
    Err(ContractError::Unauthorized {})
}
/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 * {Deprecated}
 */
pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}
/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 * {Deprecated}
 */
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetOfferings {} => to_binary(&query_offerings(deps)?),
    }
}

// ================================ Query Handlers ==================================================
/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
fn query_offerings(deps: Deps) -> StdResult<OfferingResponse> {
    let res: StdResult<Vec<QueryOfferingResult>> = OFFERINGS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|kv_item| parse_offering(kv_item))
        .collect();
    Ok(OfferingResponse {
        offerings: res?, // Placeholder
    })
}

/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 */
fn parse_offering<T>(item: StdResult<(String, Offering<T>)>) -> StdResult<QueryOfferingResult> 
where T: Serialize + DeserializeOwned + Clone,
{
    item.and_then(|(k, offering)| {
        let extension = serde_json::to_string(&offering.extension).unwrap();
        Ok(QueryOfferingResult{
            id: k.to_string(),
            token_id: offering.token_id,
            list_price: offering.list_price,
            contract_addr: offering.contract_addr.clone(),
            seller: offering.seller.clone(),
            owner: offering.owner.clone(),
            extension: extension,
        })
    })
}

//================================= Query Handle ====================================================
/**
 * @author kevinnguyen <kevin.nguyen.ai@gmail.com>
 * Test contracts
 */
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies,mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{Deps, DepsMut, Addr, coins, from_binary, Uint128};
    use cw20::Cw20CoinVerified;
    use cw721::Cw721ReceiveMsg;

    #[test]
    fn sell_offering_path() {
        let mut deps = mock_dependencies();

        let msg = InitMsg { count: 17, name: "test marketplace".to_string() };
        let info = mock_info("creator", &coins(1000, "token"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, _res.messages.len());

        // seller can release nft
        let info = mock_info("anyone", &coins(2, "token"));

        let sell_msg = SellNft {
            list_price: Cw20CoinVerified {
                address: Addr::unchecked("cw20ContractAddr"),
                amount: Uint128::new(5),
            },
        };

        let rcv_msg = HandleMsg::ReceiveNft(
            Cw721ReceiveMsg {
                sender: String::from("seller"),
                token_id: String::from("SellableNFT"),
                msg: to_binary(&sell_msg).unwrap(),
            },
        );
        
        let _res = execute(deps.as_mut(), mock_env(), info, rcv_msg).unwrap();

        // Offering should be listed
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let value: OfferingResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.offerings.len());

        let buy_msg = BuyNft {
            offering_id: value.offerings[0].id.clone(),
        };

        let rcv_msg = HandleMsg::Receive(
            Cw20ReceiveMsg {
                sender: String::from("buyer"),
                amount: Uint128::new(5),
                msg: to_binary(&buy_msg).unwrap()
            }
        );

        let info_buy = mock_info("cw20ContractAddr", &coins(2, "token"));

        let _res = execute(deps.as_mut(), mock_env(), info_buy, rcv_msg).unwrap();

        // check Offerings again. Should be 0
        let buy_res = query(deps.as_ref(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let buy_value: OfferingResponse = from_binary(&buy_res).unwrap();
        assert_eq!(0, buy_value.offerings.len());
    }

    #[test]
    fn withdraw_offering_path() {
        let mut deps = mock_dependencies();

        let msg = InitMsg {
            name: String::from("test market"),
            count: 1000,
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Release Offering NFT to Sell
        let info = mock_info("anyone", &coins(2, "token"));

        let sell_msg = SellNft {
            list_price: Cw20CoinVerified {
                address: Addr::unchecked("cw20ContractAddr"),
                amount: Uint128::new(5),
            },
        };

        let rcv_msg = HandleMsg::ReceiveNft(
            Cw721ReceiveMsg {
                sender: String::from("seller"),
                token_id: String::from("SellableNFT"),
                msg: to_binary(&sell_msg).unwrap(),
            },
        );

        let _res = execute(deps.as_mut(), mock_env(), info, rcv_msg).unwrap();

        // Offerings should be listed
        let list_res = query(deps.as_ref(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let list_value: OfferingResponse = from_binary(&list_res).unwrap();
        assert_eq!(1, list_value.offerings.len());
        //assert_eq!("1", list_value.offerings[0].id.clone());
        // Withraw offering
        let withraw_info = mock_info("seller", &coins(2, "token"));
        let withraw_msg = HandleMsg::WithdrawNft {
            offering_id: list_value.offerings[0].id.clone(),
        };
        let _res = execute(deps.as_mut(), mock_env(), withraw_info, withraw_msg).unwrap();
        assert_eq!("1", _res.attributes[2].value);
        // Offering should be removed
        let rm_res = query(deps.as_ref(), mock_env(), QueryMsg::GetOfferings {}).unwrap();
        let rm_value: OfferingResponse = from_binary(&rm_res).unwrap();
        assert_eq!(0, rm_value.offerings.len());
    }
}