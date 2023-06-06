#!/usr/bin/env bash

# shellcheck disable=SC2155

set -e

public_key=$(octez-client show address prod | grep "Public Key" | cut -d ' ' -f 3)

rollup_address=$(octez-client show known smart rollup tezos-place | xargs)

mkdir -p ./rollup/wasm_2_0_0

build() {
    cargo build --release --target wasm32-unknown-unknown --package kernel
    wasm-strip ./target/wasm32-unknown-unknown/release/kernel.wasm
}

case $1 in

originate-rollup)
    build

    cargo run --bin smart-rollup-installer -- get-reveal-installer \
        --upgrade-to ./target/wasm32-unknown-unknown/release/kernel.wasm \
        --output ./installer.hex \
        --preimages-dir ./kernel_preimages

    octez-client \
        -f ./secret/password \
        originate smart rollup tezos-place from prod \
        of kind wasm_2_0_0 of type 'ticket bytes' with kernel ./installer.hex \
        --burn-cap 2 \
        --force
    ;;

originate-contract)
    ligo compile contract ./contract/upgrade.mligo >./contract/upgrade.tz

    octez-client \
        -f ./secret/password \
        originate contract upgrade transferring 0 from prod \
        running ./contract/upgrade.tz \
        --init "\"$public_key\"" \
        --burn-cap 0.1415 \
        --force
    ;;

upgrade)
    if ! git diff --quiet --exit-code; then
      echo "Error: Worktree is dirty. Please commit or stash your changes before upgrading."
      exit 1
    fi
 
    build

    hash=$(cargo run --bin upgrade-client -- get-reveal-installer \
        --kernel ./target/wasm32-unknown-unknown/release/kernel.wasm \
        -P ./rollup/wasm_2_0_0/ | cut -d ' ' -f 3)

    echo "Upgrading Rollup to $hash"

    signature=$(octez-client -f ./secret/password sign bytes "0x$hash" for prod | cut -d ' ' -f 2)
    echo "Signature: $signature"

    contract="$(octez-client show known contract upgrade)"

    payload="Pair  (Pair    \"$signature\"    0x$hash)  \"$SR_ADDRESS\""

    octez-client -f ./secret/password \
        transfer 0 from prod to "$contract" --entrypoint "default" \
        --arg "$payload" \
        --burn-cap 0.02
    ;;

start)
    echo Starting Rollup "$rollup_address"
    sudo mkdir -p /var/lib/rollup/.tezos-smart-rollup-node/wasm_2_0_0
    sudo cp ./kernel_preimages/* /var/lib/rollup/.tezos-smart-rollup-node/wasm_2_0_0
    # FIXME: fix the config setup  to read from ./secret/config.json
    sudo HOME=/var/lib/rollup TEZOS_LOG='* -> info' octez-smart-rollup-node-PtMumbai \
        -E https://mainnet.api.tez.ie \
        run \
        --rollup sr1VHLzFsBdyL8jiEGHRdkBj3o9k7NujQhsx \
        --mode observer \
        --log-kernel-debug 
    ;;
*)
    echo "Unknown command: $1"
    ;;
esac
