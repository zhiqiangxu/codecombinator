use super::OperatorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Extern, Func, Instance, Linker, Module, Store};
use wasmtime_wasi::{Wasi, WasiCtx};

pub struct Wasm {
    config: Config,
}

mod runtime;
mod std;

impl Wasm {
    pub fn new(config: Config) -> Wasm {
        Wasm { config }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Wat {
    FilePath(String),
    Content(Box<[u8]>),
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub wat: Wat,
    #[serde(default)]
    pub wsgi: bool,
}

#[async_trait]
impl super::Source for Wasm {
    async fn start(&self) -> Result<(), OperatorError> {
        let engine = Engine::default();

        let store = Store::new(&engine);

        let module = match &self.config.wat {
            Wat::FilePath(path) => Module::from_file(&engine, path),
            Wat::Content(content) => Module::new(&engine, content),
        }?;

        let f = if self.config.wsgi {
            let mut linker = Linker::new(&store);
            let wasi = Wasi::new(&store, WasiCtx::new(::std::env::args()).unwrap());
            wasi.add_to_linker(&mut linker)?;
            linker.func("env", "add", runtime::add).unwrap();
            linker.module("", &module)?;
            let ex = linker.get_one_by_name("", "invoke")?;
            match ex {
                Extern::Func(f) => f,
                _ => panic!("invalid extern type"),
            }
        } else {
            let mut linker = Linker::new(&store);
            linker.func("env", "add", runtime::add).unwrap();
            let instance = linker.instantiate(&module).unwrap();
            let main = instance
                .get_func("invoke")
                .expect("`invoke` was not an exported function");
            main
        };

        f.call(&[]).unwrap();
        // let main = f.get0::<()>()?;

        // main().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operator::Source;

    #[async_std::test]
    async fn it_works() {
        // let wat_content = r#"(module
        //     (import "env" "add" (func $add (param i32 i32) (result i32)))
        //     (func (export "invoke")
        //         i32.const 1
        //         i32.const 1
        //         call $add
        //         drop
        //     ))"#
        // .to_string();

        let wat_content = r#"
            (import "env" "add" (func $add (param i32 i32) (result i32)))
            (func (export "invoke") 
                i32.const 1
                i32.const 1
                call $add
                drop
            )"#
        .to_string();
        let wt = Wasm::new(Config {
            wat: Wat::Content(wat_content.into_bytes().into()),
            wsgi: false,
        });

        // let wt = Wasm::new(Config {
        //     wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/jvm/target//wasm32-unknown-unknown/debug/wasm_invoke.wasm".into()),
        //     wsgi: false,
        // });

        // let wt = Wasm::new(Config {
        //     wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/jvm/target/wasm32-wasi/debug/wasm_invoke.wasi.wasm".into()),
        //     wsgi: true,
        // });

        wt.start().await.unwrap();
    }
}
