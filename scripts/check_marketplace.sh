#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

# there are imported by others contracts
BASE_CONTRACTS="marketplace"
SLEEP_TIME=3

# Deployment environment variable
CODE_ID=120
MARKET_CONTRACT_ADDR="aura1p39334eay3249x6y6ngpy3uaugjhdnwhzyrt7769x44xqus903asavny3z"
TOKEN_CONTRACT_ADDR="aura1k0pz94lpj9guug8yd0r3ja8x98c6mtpzygrtt492vhqm0zwvzmysf3cuya"
EXTENSION_DATA='{}'
OWNER_ADDR="aura1zdkrfm38qa7s3ecmrw77xvvyqfa97wctksu003"
GUEST_ADDR="aura14w8xhr3xz8hn9cut3un6f785zmlpwe6xk0l5ah"
NFT_CONTRACT_ADDR="aura14nnayu3csk2qd7fq6kg8swck35cr7tl29nq2yfvk7lv5ngmkcndqazuzyn"
#========================== TestContract =========================================================================
#======= define args input
while getopts o:b:w:q:t: flag
do
    case "${flag}" in
    o)  ## Create an Offerings for Sell NFT
        if [[ -n ${OPTARG} ]]; then
            TOKEN_ID="rvn-base-sample-nft"
            LIST_PRICE_AMOUNT="10"
            LIST_PRICE=$(jo list_price=$(jo address=$TOKEN_CONTRACT_ADDR -s amount="$LIST_PRICE_AMOUNT"))
            LIST_PRICE_ENCODE=$(echo $LIST_PRICE | base64)
            LIST_MSG=$(jo send_nft=$(jo contract=$MARKET_CONTRACT_ADDR token_id=$TOKEN_ID msg=$LIST_PRICE_ENCODE))
            EXEC_RES=$(aurad tx wasm execute $NFT_CONTRACT_ADDR "$LIST_MSG" --gas auto --gas-prices 0.025uaura --gas-adjustment 1.3 --from guest --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y | jq -r ".txhash")
            RESULT=$(aurad query tx --type=hash $EXEC_RES --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json | jq -r ".raw_log")
            echo $RESULT
        fi
        ;;
    b)  ## Take an Action for Buy NFT which is selling with id = arg input
        if [[ -n ${OPTARG} ]]; then
            SENDER=$OWNER_ADDR
            ## ensure SENDER have enough money
            AMOUNT_MSG=$(jo balance=$(jo address=$OWNER_ADDR))
            OWNER_AMOUNT=$(aurad query wasm contract-state smart $TOKEN_CONTRACT_ADDR "$AMOUNT_MSG" --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json)
            echo $OWNER_AMOUNT
            ## pack Buy msg to send to marketplace to buy NFT
            MSG=$(jo -- -s offering_id=${OPTARG})
            echo $MSG
            MSG_ENCODE=$(echo $MSG | base64)
            echo $MSG_ENCODE
            BUY_MSG=$(jo receive=$(jo sender=$SENDER msg=$MSG_ENCODE -s amount=10))
            echo $BUY_MSG
            RES=$(aurad tx wasm execute $MARKET_CONTRACT_ADDR "$BUY_MSG" --from wallet --gas auto --gas-prices 0.025uaura --gas-adjustment 1.3 --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y | jq -r ".txhash")
            echo $RES
        fi
        ;;
    w)  ## Withraw NFT to Wallet
        echo $GUEST_ADDR
        ;;
    q)  ## Query Offerings on marketplace
        if [[ -n ${OPTARG} ]]; then
            MSG=$(jo get_offerings={})
            RES=$(aurad query wasm contract-state smart $MARKET_CONTRACT_ADDR "$MSG" --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json)
            echo $RES
        fi
        ;;
    t)  ## Query Token with token_id ~ input 
        if [[ -n ${OPTARG} ]]; then
            MSG=$(jo owner_of=$(jo token_id=${OPTARG}))
            RES=$(aurad query wasm contract-state smart $NFT_CONTRACT_ADDR "$MSG" --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json)
            echo $RES
        fi
        ;;
    esac
done
echo "Test finnished"





