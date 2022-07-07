use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use marketplace::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, HandleMsg, InitMsg, BuyNft, SellNft};
use marketplace::state::State;
use marketplace::package::{ContractInfoResponse, QueryOfferingResult, OfferingResponse };

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(CountResponse), &out_dir);
    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(SellNft), &out_dir);
    export_schema(&schema_for!(BuyNft), &out_dir);
    export_schema(&schema_for!(OfferingResponse), &out_dir);
    export_schema(&schema_for!(ContractInfoResponse), &out_dir);
    export_schema(&schema_for!(QueryOfferingResult), &out_dir);
}
