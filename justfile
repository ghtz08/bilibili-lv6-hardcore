alias c := check

default:
    @just --list --justfile {{justfile()}}

check:
    cd '{{justfile_directory()}}'
    git stage .
    cargo fmt
    git stage .
    cargo fix --allow-staged
    cargo clippy -- -D warnings
    cargo build
