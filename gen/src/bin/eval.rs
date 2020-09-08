use gen::graph::VisualGraph;
use tera::Context;
use tera::Tera;

fn main() {
    let vg: VisualGraph = serde_json::from_str(include_str!("../resource/graph.json")).unwrap();
    let g = vg.to_graph();

    let result = Tera::one_off(
        include_str!("../resource/tera/graph.tpl"),
        &Context::from_serialize(&g).unwrap(),
        false,
    )
    .unwrap();

    println!("{}", result);
}
