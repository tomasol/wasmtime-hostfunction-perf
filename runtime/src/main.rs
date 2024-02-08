use anyhow::bail;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use wasmtime::{component::*, Trap};
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

    async fn panic(&mut self, host: bool) -> wasmtime::Result<()> {
        println!("host panicking: {host}");
        panic!("host panicking")
    }
}

#[tokio::main]
async fn main() -> wasmtime::Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(true);
    let engine = Arc::new(Engine::new(&config)?);
    let wasm_file = std::env::args()
        .collect::<Vec<_>>()
        .get(1)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "target/wasm32-unknown-unknown/debug/component.wasm".to_string());

    println!("Loading `{wasm_file}`");
    let component = Arc::new(Component::from_file(&engine, wasm_file)?);

    let mut linker = Linker::new(&engine);
    Example::add_to_linker(&mut linker, |state: &mut HostImpl| state)?;
    let linker = Arc::new(linker);

    let host_panic_task = {
        let engine = engine.clone();
        let component = component.clone();
        let linker = linker.clone();
        tokio::spawn(async move {
            let stopwatch = Instant::now();
            let mut store = Store::new(&engine, HostImpl {});
            let (bindings, _) = Example::instantiate_async(&mut store, &component, &linker)
                .await
                .unwrap();
            println!(
                "instantiation finished in {} µs",
                stopwatch.elapsed().as_micros()
            );
            let _ = bindings.interface0.call_panic(&mut store, true).await;
            unreachable!();
            // host panic is not caught
        })
    };
    let join_err = host_panic_task.await.unwrap_err();
    assert!(join_err.is_panic());

    let regular_task = {
        let engine = engine.clone();
        let component = component.clone();
        let linker = linker.clone();
        tokio::spawn(async move {
            let stopwatch = Instant::now();
            let mut store = Store::new(&engine, HostImpl {});
            let (bindings, _) = Example::instantiate_async(&mut store, &component, &linker)
                .await
                .unwrap();
            println!(
                "instantiation2 finished in {} µs",
                stopwatch.elapsed().as_micros()
            );

            {
                let stopwatch = Instant::now();
                bindings
                    .interface0
                    .call_return_ok(&mut store)
                    .await
                    .unwrap();
                println!(
                    "return_ok finished in {} µs",
                    stopwatch.elapsed().as_micros()
                );
            }
            {
                let stopwatch = Instant::now();
                let err = bindings
                    .interface0
                    .call_panic(&mut store, false)
                    .await
                    .unwrap_err();
                let trap = err.downcast::<Trap>().unwrap();
                assert_eq!(Trap::UnreachableCodeReached, trap);
                println!(
                    "guest panic finished in {} µs",
                    stopwatch.elapsed().as_micros()
                );
            }
            {
                // Any further calls after a guest panic lead to cannot enter component
                let stopwatch = Instant::now();
                let err = bindings
                    .interface0
                    .call_return_ok(&mut store)
                    .await
                    .unwrap_err();
                let trap = err.downcast::<Trap>().unwrap();
                assert_eq!(Trap::CannotEnterComponent, trap);
                println!(
                    "CannotEnterComponent finished in {} µs",
                    stopwatch.elapsed().as_micros()
                );
            }
        })
    };

    regular_task.await.unwrap();

    let stopwatch = Instant::now();
    let mut store = Store::new(&engine, HostImpl {});
    let (bindings, _) = Example::instantiate_async(&mut store, &component, &linker)
        .await
        .unwrap();
    println!(
        "instantiation3 finished in {} µs",
        stopwatch.elapsed().as_micros()
    );
    let stopwatch = Instant::now();
    bindings
        .interface0
        .call_return_ok(&mut store)
        .await
        .unwrap();
    println!(
        "return_ok finished in {} µs",
        stopwatch.elapsed().as_micros()
    );

    Ok(())
}
