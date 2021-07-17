#!/bin/bash

export SOLANA_ADAPTER_DIRECTORY=/usr/local/var/www/solana-adapter/src/gravity-core-adapter;

_execute_in_directory() {
  exe_command=$1;
  wheretoexec=$2;
  current_dir=$(pwd);
  echo "$exe_command";
  echo "$wheretoexec";
  echo "$current_dir";

  cd $wheretoexec;
  $exe_command;
  cd $current_dir;
}

rebuild_solana_contract() {
  contract_name=$1;

  rebuild_contract() {
    current_dir=$(pwd);
    cd "./$contract_name";

    # ls -la
    # echo "i am here: $(pwd)"
    # echo "current_dir: $current_dir"
    printf "Rebuilding %s contract...\n" $contract_name

    cargo build-bpf;
    cd "$current_dir";
  }

  _execute_in_directory "rebuild_contract" $SOLANA_ADAPTER_DIRECTORY
}

# public
rebuild-all-solana-contracts() {
  contracts=(gravity nebula luport ibport);

  for contract in ${contracts[@]}
  do
    rebuild_solana_contract "$contract"
  done
}
