alias c := check
alias b := build
alias br := build-release
alias bm := build-musl
alias u := update
alias tu := tree-unchanged

default:
    @just --list --justfile {{justfile()}}

check:
    @cd '{{justfile_directory()}}'
    cargo check

fix:
    @cd '{{justfile_directory()}}'
    git stage .
    cargo fmt
    git stage .
    cargo fix --allow-staged
    cargo clippy --fix --allow-dirty
    cargo clippy -- -D warnings
    cargo build

build:
    @cd '{{justfile_directory()}}'
    cargo build --release

build-release:
    @cd '{{justfile_directory()}}'
    cargo build --release

build-musl:
    @cd '{{justfile_directory()}}'
    cargo build --target x86_64-unknown-linux-musl --release

update:
    @cd '{{justfile_directory()}}'
    cargo update --verbose

tree-unchanged:
    @cd '{{justfile_directory()}}'
    cargo update --verbose 2>&1 | grep -P ' Un\w+ ' | grep -oP '\w+\s+v\d+(\.\d+){2}' | sed 's/ v/@/' | xargs -r -t -n1 cargo tree --invert
