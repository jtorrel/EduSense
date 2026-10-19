#!/usr/bin/env bash
# Installe les outils utilisés par la CI pour pouvoir tout exécuter localement.
set -euo pipefail

rustup show active-toolchain
rustup component add llvm-tools-preview
cargo install --locked cargo-llvm-cov cargo-mutants cargo-deny cargo-audit
(cd frontend && npm ci)
