#!/bin/sh
mkdir logs
cargo run --release | tee logs/"$(date).log"