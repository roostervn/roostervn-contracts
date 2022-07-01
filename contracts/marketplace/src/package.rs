use cosmwasm_std::{Addr, Response};
use cw20::Cw20CoinVerified;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::error::{ContractError};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    pub name: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOfferingResult {
    pub id: String,
    pub token_id: String,
    pub list_price: Cw20CoinVerified,
    pub contract_addr: Addr,
    pub seller: Addr,
    pub owner: Addr,
    pub extension: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OfferingResponse {
    pub offerings: Vec<QueryOfferingResult>,
}