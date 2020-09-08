use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct VisualNode {
    pub kind: String,
    pub config: String,
}

#[derive(Serialize, Deserialize)]
pub struct VisualGraph {
    pub operators: Vec<VisualNode>,
    pub applies: HashMap<usize, Vec<usize>>,
}

#[derive(Serialize)]
pub struct Operator {
    pub config: String,
    pub meta: OperatorMeta,
}

#[derive(Clone, Serialize)]
pub struct OperatorMeta {
    pub file: &'static str,
    pub ty: &'static str,
    pub source: bool,
    pub new_async: bool,
}

impl VisualGraph {
    pub fn to_graph(&self) -> Graph {
        let mut handled = vec![false; self.operators.len()];
        let mut edgeto = vec![-1; self.operators.len()];
        let mut onstack = vec![false; self.operators.len()];

        let mut sorted_applies = vec![];
        let mut operators = vec![];

        let meta: HashMap<_, _> = vec![
            (
                "mysql",
                OperatorMeta {
                    file: "sql",
                    ty: "Sql::<::sqlx::mysql::MySql>",
                    source: false,
                    new_async: true,
                },
            ),
            (
                "sql_runner",
                OperatorMeta {
                    file: "sql_runner",
                    ty: "SqlRunner",
                    source: false,
                    new_async: false,
                },
            ),
            (
                "http_api",
                OperatorMeta {
                    file: "http_api",
                    ty: "HTTPAPI",
                    source: false,
                    new_async: false,
                },
            ),
            (
                "http_server",
                OperatorMeta {
                    file: "http_server",
                    ty: "HTTPServer",
                    source: true,
                    new_async: false,
                },
            ),
        ]
        .into_iter()
        .collect();

        fn handle_one(
            i: usize,
            vg: &VisualGraph,
            handled: &mut Vec<bool>,
            edgeto: &mut Vec<isize>,
            onstack: &mut Vec<bool>,
            sorted_applies: &mut Vec<Apply>,
        ) {
            unsafe {
                *onstack.get_unchecked_mut(i) = true;

                match vg.applies.get(&i) {
                    Some(froms) => {
                        for from in froms {
                            let from = *from;
                            if *handled.get_unchecked(from) {
                                sorted_applies.push(Apply { to: i, from: from });
                                continue;
                            }
                            if *onstack.get_unchecked_mut(from) {
                                let mut cycle = vec![];
                                let mut iparent = *edgeto.get_unchecked(from);
                                loop {
                                    if iparent == -1 {
                                        panic!("bug happened");
                                    }

                                    let parent = iparent as usize;
                                    if parent != from {
                                        cycle.push(parent);
                                        iparent = *edgeto.get_unchecked(parent);
                                    } else {
                                        break;
                                    }
                                }
                                cycle.push(from);
                                cycle.push(i);
                                panic!(println!("cycle found:{:?}", cycle));
                            } else {
                                *edgeto.get_unchecked_mut(from) = i as isize;
                                handle_one(from, vg, handled, edgeto, onstack, sorted_applies);

                                sorted_applies.push(Apply { to: i, from: from });
                            }
                        }
                    }
                    None => {}
                }

                *onstack.get_unchecked_mut(i) = false;
                *handled.get_unchecked_mut(i) = true;
            }
        }

        for (i, op) in self.operators.iter().enumerate() {
            if !handled[i] {
                handle_one(
                    i,
                    &self,
                    &mut handled,
                    &mut edgeto,
                    &mut onstack,
                    &mut sorted_applies,
                );
            }
            operators.push(Operator {
                config: op.config.clone(),
                meta: meta.get(op.kind.as_str()).unwrap().clone(),
            });
        }

        Graph {
            operators,
            sorted_applies,
        }
    }
}

#[derive(Serialize)]
pub struct Apply {
    pub to: usize,
    pub from: usize,
}
#[derive(Serialize)]
pub struct Graph {
    pub operators: Vec<Operator>,
    pub sorted_applies: Vec<Apply>,
}
