#!/bin/bash



cargo build-bpf --bpf-out-dir nebula --features nebula-contract
cargo build-bpf --bpf-out-dir gravity --features gravity-contract

diff nebula/solana_gravity_adapter.so gravity/solana_gravity_adapter.so