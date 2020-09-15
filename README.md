# code combinator without loss of performance

**`code combinator`** works by combinating reusable `operators`, and it functions by callback instead of value passing via channels(e.g. [**`actix`**](https://github.com/actix/actix)), so no performance loss.

current operators:
1. `mysql`
2. `sql_runner`
3. `http_api`
4. `http_server`
5. `wasm`
6. `simple_auth`
7. `saga_aggregator`

Each operator has its own configurations, defined in `struct Config` in specific file under `core/src/operator`.

For example, if the input json is:

```json
{
    "operators": [
        {
            "kind": "mysql",
            "config": {
                "dsn": "mysql://user:pass@172.168.3.1:3307/db?readTimeout=3s&charset=utf8mb4"
            }
        },
        {
            "kind": "sql_runner",
            "config": {
                "sql": "select * from user limit 10"
            }
        },
        {
            "kind": "http_api",
            "config": {
                "uri": "/",
                "method": "GET"
            }
        },
        {
            "kind": "http_server",
            "config": {
                "listen_addr": "127.0.0.1:8088"
            }
        }
    ],
    "applies": {
        "1": [
            0
        ],
        "2": [
            1
        ],
        "3": [
            2
        ]
    }
}
```

The above will generate `1` http server listening at `/` at port `8088`, and when accessed, the rows matching `select * from user limit 10` will be returned.


## run

```
# uses gen/src/resource/graph.json by default, you can specify another json file
# generates the file core/src/bin/demo.rs
php build.php

# run the generated file core/src/bin/demo.rs
cargo run --bin demo
```


