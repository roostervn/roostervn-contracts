#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck > /dev/null && shellcheck "$0"

# there are imported by others contracts
BASE_CONTRACTS="rvn-base"
SLEEP_TIME=30

# Deployment environment variable
CODE_ID=116
CONTRACT_ADDR="aura14nnayu3csk2qd7fq6kg8swck35cr7tl29nq2yfvk7lv5ngmkcndqazuzyn"
IPFS_URL="ipfs://bafybeihh5itjrg2bfpbktdntbx2pyqnj77wk2gbsmwblkaiywk3nmhzj7i/e9c2e6662425ef8d40bd8e494f4a9407.jpeg"
EXTENSION_DATA='{}'
OWNER="aura1zdkrfm38qa7s3ecmrw77xvvyqfa97wctksu003"
#========================== TestContract =========================================================================
#======= define args input
## mint accept OPTARGS "owner" or "guest" only
while getopts m:t:a:r:b:approve_all:revoke_all: flag
do
    case "${flag}" in 
        m)  # mint NFT
            # Check mint a new NFT
            MINT_TOKEN_ID="rvn-base-sample-nft"
            MINT_NFT_MSG=$(jo mint=$(jo token_id=$MINT_TOKEN_ID owner=$OWNER token_url=$IPFS_URL extension=$EXTENSION_DATA))
            #echo $MINT_NFT_MSG
            MINT_FEE=$(jo amount=$(jo -a $(jo denom="uaura" amount="16")) gas="152375")
            # owner mint OK
            if [[ ${OPTARG} == "owner" ]]; then
                MINT_RES=$(aurad tx wasm execute $CONTRACT_ADDR "$MINT_NFT_MSG" --fees 200uaura --from wallet --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y )
            elif [[ ${OPTARG} == "guest" ]]; then
                MINT_RES=$(aurad tx wasm execute $CONTRACT_ADDR "$MINT_NFT_MSG" --fees 200uaura --from guest --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y )
            else
                echo "mint only accept owner or guest only"
            fi
            #echo $MINT_RES
            echo $MINT_RES
            ;;
        t)  # transfer_nft
            TOKEN_ID="rvn-base-sample-nft"
            OWN_MSG=$(jo owner_of=$(jo token_id=$TOKEN_ID ))
            ## owner of token_id
            OWNER_TOKEN_ID=$(aurad query wasm contract-state smart $CONTRACT_ADDR "$OWN_MSG" --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json | jq -r ".data.owner")
            if [[ -n ${OPTARG} && -n "$OWNER_TOKEN_ID" ]]; then
                SEND_NFT_MSG=$(jo transfer_nft=$(jo recipient=${OPTARG} token_id=$TOKEN_ID ))
                SEND_RES=$(aurad tx wasm execute $CONTRACT_ADDR "$SEND_NFT_MSG" --fees 200uaura --from wallet --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json -y | jq -r ".txhash")
                echo "txhash of transfer_of $SEND_RES"
                sleep 5
                SEND_RES_TX_RESULT=$(aurad query tx --type=hash $SEND_RES --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json | jq -r ".raw_log")
                echo "result of transaction request $SEND_RES_TX_RESULT"
                sleep 5
                OWNER_AFTER_TRANSFER=$(aurad query wasm contract-state smart $CONTRACT_ADDR "$OWN_MSG" --node https://rpc.serenity.aura.network:443 --chain-id serenity-testnet-001 --output json | jq -r ".data.owner")
                echo "owner of NFT $TOKEN_ID is $OWNER_AFTER_TRANSFER"
            else
                echo "Please input address will receive nft from minter"
            fi
            ;;
        a)  # approve
            echo "approve"
            ;;
        r)  # revoke 
            echo  "revoke"
            ;;
        b)  # burn all
            echo "burn"
            ;;  
    esac
done
echo "Test finnished"