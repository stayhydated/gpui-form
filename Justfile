default:
    @just --list

fmt:
    cargo sort-derives
    cargo fmt
    taplo fmt
