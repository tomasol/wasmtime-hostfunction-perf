# Sample wasm component and runtime

To build the project:
```sh
nix develop # or direnv allow
cargo component build -p component --target wasm32-unknown-unknown
cargo run -p runtime
```
