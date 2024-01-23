use anyhow::bail;
use async_trait::async_trait;
use std::time::Instant;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};

wasmtime::component::bindgen!({
    path: "../wit-runtime/",
    async: true,
    // interfaces: "import runtime:runtime/host-functions;",
});

struct HostImpl {}
#[async_trait]
impl runtime::runtime::host_functions::Host for HostImpl {
    async fn return_ok(&mut self) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn return_err(&mut self) -> wasmtime::Result<()> {
        bail!("host error")
    }

    async fn panic(&mut self) -> wasmtime::Result<()> {
        println!("panicking");
        panic!()
    }
}

#[tokio::main]
async fn main() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Engine::new(&config)?;
    let wasm_file = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "target/wasm32-unknown-unknown/debug/component.wasm".to_string());

    println!("Loading `{wasm_file}`");
    let component = Component::from_file(&engine, wasm_file)?;

    let mut linker = Linker::new(&engine);
    Example::add_to_linker(&mut linker, |state: &mut HostImpl| state)?;

    let mut store = Store::new(&engine, HostImpl {});
    let (bindings, _) = Example::instantiate_async(&mut store, &component, &linker).await?;

    let stopwatch = Instant::now();
    bindings.interface0.call_return_ok(&mut store).await?;
    println!(
        "return_ok finished in {} µs",
        stopwatch.elapsed().as_micros()
    );

    let stopwatch = Instant::now();
    let err = bindings
        .interface0
        .call_return_err(&mut store)
        .await
        .unwrap_err();
    assert_eq!(err.source().unwrap().to_string(), "host error");
    println!(
        "return_err finished in {} µs",
        stopwatch.elapsed().as_micros()
    );

    Ok(())
}
