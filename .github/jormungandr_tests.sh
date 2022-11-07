set -ex

RUSTFLAGS="-D warnings" 
RUST_BACKTRACE=1
CARGO_FLAGS="--verbose --locked"
CARGO_INCREMENTAL=0

cargo install cargo-nextest
cargo nextest run --no-fail-fast --partition hash:$1/$2 

