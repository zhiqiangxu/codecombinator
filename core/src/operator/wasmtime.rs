use super::OperatorError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Instance, Module, Store};

pub struct WasmTime {
    config: Config,
}

impl WasmTime {
    pub fn new(config: Config) -> WasmTime {
        WasmTime { config }
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
}

#[async_trait]
impl super::Source for WasmTime {
    async fn start(&self) -> Result<(), OperatorError> {
        let engine = Engine::default();

        let store = Store::new(&engine);

        let module = match &self.config.wat {
            Wat::FilePath(path) => Module::from_file(&engine, path),
            Wat::Content(content) => Module::new(&engine, content),
        }?;

        let instance = Instance::new(&store, &module, &[])?;

        let main = instance
            .get_func("invoke")
            .expect("`invoke` was not an exported function");

        let main = main.get0::<()>()?;

        main().unwrap();

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
        let _wt = WasmTime::new(Config {
            wat: Wat::Content(wat_content.into_bytes().into()),
        });

        let wt = WasmTime::new(Config {
            wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/hi/target/wasm32-unknown-unknown/release/hi.wasm".into()),
        });

        wt.start().await.unwrap();

        println!("{}",serde_json::to_string(&Config {
            wat: Wat::FilePath("/Users/xuzhiqiang/Desktop/workspace/opensource/rust_exp/hi/target/wasm32-unknown-unknown/release/hi.wasm".into()),
        }).unwrap());
    }
}
