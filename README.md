first run anvil at local:
```
anvil --fork-block-number <block_num> --fork-url <some_rpc_endpoint>
```

then run `cargo run --bin simulate` to simulate a tx that swaps 0.1 eth to dai on uniswap v2.
trace of the tx is printed out at console.
