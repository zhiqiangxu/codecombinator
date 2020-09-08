

{% for op in operators %}
let config{{loop.index0}} = serde_json::from_str({{op.config|json_encode}}).unwrap();
{% if op.meta.new_async %}
let mut op{{loop.index0}} = Arc::new({{op.meta.file}}::{{op.meta.ty}}::new(config{{loop.index0}}).await);
{% else %}
let mut op{{loop.index0}} = Arc::new({{op.meta.file}}::{{op.meta.ty}}::new(config{{loop.index0}}));
{% endif %}
{% endfor %}

{% for apply in sorted_applies %}
Arc::get_mut(&mut op{{apply.to}}).unwrap().apply(Arc::downgrade(&op{{apply.from}}));
{% endfor %}

let mut handles = vec![];

{% for op in operators %}
    {% if op.meta.source %}
        handles.push(::async_std::task::spawn(async move {
            match op{{loop.index0}}.start().await {
                Ok(_) => {}
                Err(err) => println!("err : {:?}", err),
            };
        }));
    {% endif %}
{% endfor %}
    
::futures::future::join_all(handles).await;