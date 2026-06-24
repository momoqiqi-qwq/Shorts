$ErrorActionPreference = "Stop"
rustup update stable
cargo build --release
Write-Host "Built target/release/shotrs.exe"
