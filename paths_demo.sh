#!/bin/bash
RUST_LOG="trace" cargo run --manifest-path Real_case/Paths/Cargo.toml -- plow Real_case/Paths/demo/montreal/roads.json Real_case/Paths/demo/montreal/snow.lrive.json Real_case/Paths/demo/montreal/vehicles.json Real_case/Paths/demo/montreal/plow.meta.yaml ./plow-output.json
