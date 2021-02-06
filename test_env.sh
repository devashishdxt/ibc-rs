#!/usr/bin/env bash

CONFIG_FILE=config.toml

CHAIN_0_RPC_ADDR="tcp://localhost:20600"
CHAIN_1_RPC_ADDR="tcp://localhost:20601"
CHAIN_2_RPC_ADDR="tcp://localhost:20602"

log() {
  echo
  echo
  echo
  echo "---------------- $1 ----------------"
  echo
  echo
  echo
}

balance() {
  echo "0 balance:"
  gaiad --node $CHAIN_0_RPC_ADDR query bank balances $(gaiad --home data/ibc-0 keys --keyring-backend="test" show user -a)
  echo "1 balance:"
  gaiad --node $CHAIN_1_RPC_ADDR query bank balances $(gaiad --home data/ibc-1 keys --keyring-backend="test" show user -a)
  echo "2 balance:"
  gaiad --node $CHAIN_2_RPC_ADDR query bank balances $(gaiad --home data/ibc-2 keys --keyring-backend="test" show user -a)
  log "balance shown"
}

hermes() {
  cargo run --bin hermes -- -c "$CONFIG_FILE" $@
}

setup() {
  hermes channel handshake ibc-0 ibc-1 transfer transfer
  hermes channel handshake ibc-1 ibc-2 transfer transfer
}
