use async_std::task;
use codecombinator::operator::{add, graph, sink, source, sql, Operator};

#[async_std::main]
async fn main() {
    let mut g = graph::Graph::new();
    let add = Box::new(add::Add::<u32>::new());
    let source = Box::new(source::Source::<u32>::new());
    let sink = Box::new(sink::Sink::<u32>::new());
    let sql = Box::new(
        sql::SQL::new(sql::Config {
            dsn: "openkg:some_pass@tcp(172.168.3.46:3307)/openkg".into(),
            ..Default::default()
        })
        .await,
    );

    let source = g.add_operator(source);
    let add = g.add_operator(add);
    let sink = g.add_operator(sink);
    let sql = g.add_operator(sql);

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
