<?php

$output = shell_exec("cargo run --bin eval");

$date = date('Y-m-d H:i:s');

$content = <<<EOT
// generated by build.php at $date

use async_std::sync::Arc;
use core::operator::{http_api, http_server, sql, sql_runner, Monad, Source};

#[async_std::main]
async fn main() {
    $output
}
EOT;

file_put_contents("core/src/bin/demo.rs", $content);


shell_exec("cargo fmt");

shell_exec("cargo fix --allow-dirty --allow-staged");