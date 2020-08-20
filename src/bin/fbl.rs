use async_std::task;
use codecombinator::operator::{add, graph, sink, source, Operator};

fn main() {
    let mut g = graph::Graph::new();

    let add = Box::new(add::Add::<'static, u32>::new());

    let source = Box::new(source::Source::<'static, u32>::new());

    let sink = Box::new(sink::Sink::<u32>::new());

    let source = g.add_operator(source);
    let add = g.add_operator(add);
    let sink = g.add_operator(sink);

    g.connect(source, 0, add, 0).unwrap();
    g.connect(source, 0, add, 1).unwrap();
    g.connect(add, 0, sink, 0).unwrap();

    let graph_task = task::spawn(async move {
        match g.process().await {
            Ok(_) => {}
            Err(err) => println!("err : {:?}", err),
        };
    });
    task::block_on(graph_task);
}
