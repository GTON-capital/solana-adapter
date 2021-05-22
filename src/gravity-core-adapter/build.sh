#!/bin/bash


current=$(pwd)

gravity_dir='gravity'
nebula_dir='nebula'

cd $gravity_dir
cargo build-bpf 
cd ..

cd $nebula_dir
cargo build-bpf 
cd ..

# should output:
# Binary files gravity/target/deploy/solana_gravity_contract.so and nebula/target/deploy/solana_nebula_contract.so differ
diff gravity/target/deploy/solana_gravity_contract.so nebula/target/deploy/solana_nebula_contract.so