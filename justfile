default: run_release

run_release:
    cargo run --release -- images/wall2.jpg

release:
    cargo build --release
