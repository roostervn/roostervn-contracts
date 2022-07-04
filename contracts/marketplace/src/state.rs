#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::package::ContractInfoResponse;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::{from_utf8, FromStr};

use cosmwasm_std::{Addr, CanonicalAddr, StdResult, Storage, MemoryStorage};
use cosmwasm_std::testing::MockStorage;
use cw20::Cw20CoinVerified;
use cw_storage_plus::{index_string, Index, IndexList, IndexedMap, Item, Map, MultiIndex, KeyDeserialize};

pub static CONFIG_KEYS: &[u8] = b"config";
// @{Deprecated} State sample code
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}
/**
 * Offering<T> offer the struct of Offer will list on marketplace
 * where must have specific owner, seller, contract of Offering 
 * especially it can expand structure of storage to generics storage
 * type T where recommend for more functional of Offering like 
 * discount, special award or linking to other Offering is possible also
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Offering<T> {
    pub owner: Addr,
    pub token_id: String,
    pub contract_addr: Addr,
    pub seller: Addr,
    pub list_price: Cw20CoinVerified,
    pub extension: T 
}

/**
 * Trait define private using in scope of crate state only
 */
pub trait GenericConvert<T> 
where T: Serialize + DeserializeOwned + Clone,
{
    fn set(&mut self, field: &str, value: &T);
}

/**
 * implementation for Offering<T>
 */
impl<T> Offering<T> 
where T: Serialize + DeserializeOwned + Clone,
{ 
    // parse string type to T where T is comformtable like string
    pub fn parse_str_to_t(&mut self, data: &str) -> Result<T, T::Err> where T: FromStr {
        data.parse::<T>()
    }
    // parse T to string type using trait implementation of GenerictConvert<T>
    pub fn parse_t_to_str(&mut self, field: &str, value: &T) {
        let _ = &self.set(field, value);
        
    }
}

/**
 * Trait impl GenericConvert for Offering<T>
 */
impl<T> GenericConvert<T> for Offering<T> 
where T: Serialize + DeserializeOwned + Clone,
{
    // set `extension` in Offering<T> to value which reference to T from input
    fn set(&mut self, field: &str, value: &T)
    {
        self.extension = (value as &T).clone();
    }
}

// @{Deprecated} STATE  
pub const STATE: Item<State> = Item::new("state");
// OFFERINGS is a map which maps the offering_id to an offering. Offering_id is derived from OFFERINGS_COUNT
pub const OFFERINGS: Map<&str, Offering<String>> = Map::new("offerings" as &str);
pub const OFFERINGS_COUNT: Item<u64> = Item::new("num_offerings" as &str);
pub const CONTRACT_INFO: Item<ContractInfoResponse> = Item::new("marketplace_info" as &str);

// new offering of Offering<T>
pub fn new_offering<'a, T> () -> Map<'a, &'a str, Offering<T>> {
    Map::new(from_utf8(b"offerings").unwrap())
}

// amount of Offering in storage where storage ref to contracts deps
pub fn num_offerings<S: Storage>(storage: &S) -> StdResult<u64> {
    Ok(OFFERINGS_COUNT.may_load(storage)?.unwrap_or_default())
}

// 
pub fn increment_offerings<S: Storage>(storage: &mut S) -> StdResult<u64> {
    let val = num_offerings(storage)? + 1;
    let _ = OFFERINGS_COUNT.save(storage,&val);
    Ok(val)
}

// Indexes Struct of Offering to ref Addr to Offering<T> by Addr to Storage
pub struct OfferingIndexes<'a, T> 
where T: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, Addr, Offering<T>, Addr>,
    pub seller: MultiIndex<'a, Addr, Offering<T>, Addr>,
    pub contract: MultiIndex<'a, Addr, Offering<T>, Addr>,
}

// Interface for building indexes
impl<'a, T> IndexList<Offering<T>> for OfferingIndexes<'a, T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Offering<T>>> + '_> {
        let v: Vec<&dyn Index<Offering<T>>> = vec![&self.owner, &self.seller, &self.contract];
        Box::new(v.into_iter())

    }
}

// public function used to get Address of specs Offering<T>
pub fn offering_owner_idx<T>(d: &Offering<T>) -> Addr {
    d.owner.clone()
}

// Storage function used to build IndexedMap
pub fn offerings<'a, T, S: Storage> () -> IndexedMap<'a, &'a str, Offering<T>, OfferingIndexes<'a, T>> 
where T: Serialize + DeserializeOwned + Clone,
{
    let indexes: OfferingIndexes<'a, T> = OfferingIndexes {
        owner: MultiIndex::new(
            |o| o.owner.clone(),
            "offerings",
            "offerings_owner"
        ),
        seller: MultiIndex::new(
            |o| o.seller.clone(),
            "offerings",
            "offerings_seller"
        ),
        contract: MultiIndex::new(
            |o| o.contract_addr.clone(),
            "offerings",
            "offerings_contract",
        ),
    };
    IndexedMap::new("offerings", indexes)
}

#[cfg(test)]
mod test_state {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies,
        mock_dependencies_with_balance,
        mock_env, mock_info,
        MockStorage,
        MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{coins, from_binary, Uint128, Order};
    use std::borrow::BorrowMut;
    #[test]
    fn test_offerings() {
        // define test case env
        let mut store = MockStorage::new();
        let owner1 = Addr::unchecked("addr1");
        let contract1 = Addr::unchecked("contract1");
        let extension1 = String::from("testing_addr1");
        // define Offering by test case
        let offering_addr1 = Offering::<String> {
            owner: owner1.clone(),
            token_id: String::from("NFT1"),
            seller: owner1.clone(),
            contract_addr: contract1.clone(),
            list_price: Cw20CoinVerified {
                address: owner1.clone(),
                amount: Uint128::from(1000000u128),
            },
            extension: extension1,
        };
        // test for storage init and save
        let token_id = increment_offerings(store.borrow_mut()).unwrap();
        offerings::<String, MockStorage >().save(store.borrow_mut(), &u64::from(token_id).to_string(), &offering_addr1).unwrap();

        // want to load Offering<T> using owner1 and contract1
        let list: Vec<_> = offerings::<String, MemoryStorage>()
            .idx.owner
            .prefix(owner1)
            .range(&store, None, None, Order::Ascending)
            .collect::<StdResult<_>>().unwrap();
        let (_, t) = &list[0];
        assert_eq!(t, &offering_addr1);
        


    }
}