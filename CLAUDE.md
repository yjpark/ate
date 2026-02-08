# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ATE (Age-Encrypted Tag-ged Entries) is a Rust workspace for managing encrypted, tagged entries with hierarchical organization.

## Build Commands

```bash
cargo build          # Build all crates
cargo test           # Run tests
cargo check          # Fast compile check
cargo clippy         # Lint
cargo fmt            # Format
just build           # Alternative: build via just
```

## Development Environment

Uses Nix flakes with direnv. Run `direnv allow` to enter the environment which provides:
- Rust stable toolchain with wasm32-unknown-unknown target
- Node.js, npm, tailwindcss
- SurrealDB
- Shell alias: `c=cargo`

## Architecture

Two-crate workspace with protocol/model separation:

**ate_proto** - Serializable protocol definitions
- `Entry`: id (Uuid), tags (IndexMap), aged data, memo
- `Tag`: either `GroupTag` or `LeafTag`
- `GroupTag`: hierarchical tags (Folder, Year, Month, Day, Week, Custom) with optional parent
- `LeafTag`: entry-level tags (InFolder, Recipient, Added, Updated, Custom)
- `Recipient`: name + public key for age encryption

**ate_model** - Runtime models with concurrent data structures
- Replaces IndexMap with DashMap for concurrent access
- Uses Arc/Weak reference patterns for tree structures
- Entry implements `Identifiable` trait from edger_tree
- Tag wraps `DashTree` from edger_tree for hierarchical concurrent access

## Code Patterns

- **Optional serde**: Use `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
- **Prelude modules**: Export common types via `prelude` module
- **Extensibility**: Use Custom variants in enums for unknown types
- **Concurrency**: DashMap for shared mutable state, Arc/Weak for references

## External Dependencies

- `edger_tree` at `../crates/common/tree` - provides DashTree and Identifiable trait
