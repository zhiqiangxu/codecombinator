use super::OperatorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Extern, Instance, Linker, Module, Store};
use wasmtime_wasi::{Wasi, WasiCtx};

pub struct Wasm {
    config: Config,
}

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
            let wasi = Wasi::new(&store, WasiCtx::new(std::env::args()).unwrap());
            wasi.add_to_linker(&mut linker)?;
            linker.module("", &module)?;
            let ex = linker.get_one_by_name("", "invoke")?;
            match ex {
                Extern::Func(f) => f,
                _ => panic!("invalid extern type"),
            }
        } else {
            let instance = Instance::new(&store, &module, &[])?;
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
        let wat_content = r#"(module
            (func (export "invoke") 
               
            ))"#
        .to_string();
        let _wt = Wasm::new(Config {
            wat: Wat::Content(wat_content.into_bytes().into()),
            wsgi: false,
        });

        let wt = Wasm::new(Config {
            wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/hi/target/wasm32-wasi/debug/hi.wasi.wasm".into()),
            wsgi: true,
        });

        wt.start().await.unwrap();

        println!("{}",serde_json::to_string(&Config {
            wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/hi/target/wasm32-wasi/debug/hi.wasi.wasm".into()),
            wsgi: true,
        }).unwrap());
    }
}
