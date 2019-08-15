#!/bin/bash

PORT="7071"
HOST="crates.mirror.com"
EXTERN_URL="$HOST:$PORT"
CRATE_OPS="-u http://localhost/api/v1/crates/"

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

RUST_BACKTRACE=1
$DIR/target/release/cargo-cacher $CRATE_OPS  -p $PORT -e $EXTERN_URL &> /var/log/cargo-cacher.log &

