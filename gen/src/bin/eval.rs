use gen::graph::VisualGraph;
use std::env;
use std::fs::File;
use std::io::BufReader;
use tera::Context;
use tera::Tera;

fn main() {
    let args: Vec<String> = env::args().collect();

    let vg: VisualGraph;
    match args.len() {
        1 => {
            // compile time path
            vg = serde_json::from_str(include_str!("../resource/graph.json")).unwrap();
        }
        2 => {
            // runtime path
            let json_file_reader = BufReader::new(File::open(args.get(1).unwrap()).unwrap());
            vg = serde_json::from_reader(json_file_reader).unwrap();
        }
        _ => {
            panic!("invalid param, only need to specify json file!");
        }
    }

    let g = vg.to_graph();

    let result = Tera::one_off(
        include_str!("../resource/tera/graph.tpl"),
        &Context::from_serialize(&g).unwrap(),
        false,
    )
    .unwrap();

    println!("{}", result);
}
