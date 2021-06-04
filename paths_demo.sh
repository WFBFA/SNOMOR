#!/bin/bash
RUST_LOG="trace" cargo run --manifest-path Paths/Cargo.toml -- plow Paths/demo/montreal/roads.json Paths/demo/montreal/snow.lrive.json Paths/demo/montreal/vehicles.json Paths/demo/montreal/plow.meta.yaml ./plow-output.json
