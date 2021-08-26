# webgpu_feature_test
Test examples for features of WebGPU

## Run examples
To run examples locally, please refer to each example's README.

To run examples compiled on `wasm` and hosted on web, you will need:
1. Rust environment set up.
2. `wasm-bindgen-cli` by enter `cargo install wasm-bindgen-cli`. 
3. A simple web server
   * Like `light-server`, which can be installed by `npm install -g light-server` with `sudo`
4. Rust wasm toolchain setup by ` rustup target add wasm32-unknown-unknown`
5. run `cargo run example_name`, which will run `src/main.rs` that is equivalent to a shell script
   * For details, please refer to `src/main.rs` which is very simple
6. run a web server to host the web artifact
   * For example, `light-server --serve generated/example_name` e.g., `light-server --serve generated/hello_triangle`
7. Open a Nightly/Canary web browser, if you use `light-server`, the default address for hosted webpage is `0.0.0.0:4000`