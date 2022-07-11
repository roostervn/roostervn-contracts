#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

# there are imported by others contracts
BASE_CONTRACTS="marketplace"
SLEEP_TIME=3

# Deployment environment variable
CODE_ID=120
CONTRACT_ADDR="aura1p39334eay3249x6y6ngpy3uaugjhdnwhzyrt7769x44xqus903asavny3z"
EXTENSION_DATA='{}'
OWNER_ADDR="aura1zdkrfm38qa7s3ecmrw77xvvyqfa97wctksu003"
GUEST_ADDR="aura14w8xhr3xz8hn9cut3un6f785zmlpwe6xk0l5ah"

#========================== TestContract =========================================================================
#======= define args input
while getopts o:b:w: flag
do
    case "${flag}" in
    o)  ## Create an Offerings for Sell NFT
        if [[ -n ${OPTARG} ]]; then
            TOKEN_ID="rvn-base-sample-nft"
            NFT_CONTRACT_ADDR="aura14nnayu3csk2qd7fq6kg8swck35cr7tl29nq2yfvk7lv5ngmkcndqazuzyn"
            LIST_PRICE_AMOUNT="0.05"
            LIST_PRICE=$(jo list_price=$(jo address=$NFT_CONTRACT_ADDR amount=$LIST_PRICE_AMOUNT))
            LIST_MSG=$(jo receive_nft=$(jo sender=$GUEST_ADDR token_id=$TOKEN_ID msg=$LIST_PRICE))
            EXEC_RES=$(aurad tx wasm execute $CONTRACT_ADDR "$LIST_MSG" --fees 200uaura --from wallet --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y)
            echo $EXEC_RES
        fi
        ;;
    b)  ## Take an Action for Buy NFT which is selling
        echo $GUEST_ADDR
        ;;
    w)  ## Withraw NFT to Wallet
        echo $GUEST_ADDR
        ;;
    esac
done
echo "Test finnished"





