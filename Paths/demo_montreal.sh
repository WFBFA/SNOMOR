#!/bin/bash
RUST_LOG="trace" cargo run -- plow demo/montreal/roads.json demo/montreal/snow.lrive.json demo/montreal/vehicles.json demo/montreal/plow.meta.yaml ./plow-output.json
