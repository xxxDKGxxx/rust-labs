# Rust Programming - MINI WUT

This repository contains lab exercises and projects completed during the **Rust Programming** course at the **Faculty of Mathematics and Information Science (MiNI), Warsaw University of Technology (WUT)**.

## 🚀 Projects

The repository includes three major projects demonstrating various aspects of the Rust ecosystem, from low-level data structures to high-level game engines.

### 1. Rustabase (Project 1)
A SQL-like database engine implemented in Rust.
- **Key Features**: SQL parsing with `pest`, CRUD operations, command persistence, and filtering using complex WHERE clauses (AND, OR, nested conditions).
- **Custom Feature**: `ColumnOperatorFilter` allowing direct column-to-column comparisons within queries.
- **Tech Stack**: `pest` (parsing), `clap` (CLI), `serde` (serialization).

### 2. Bevy Strategy Game (Project 2)
A 2D strategy game prototype built using the Bevy game engine.
- **Key Features**: Procedural map generation, country management, army movement, and turn-based AI logic.
- **Tech Stack**: `bevy` (ECS, rendering, audio), `bevy_egui` (UI), `noise-rs` (map generation), `serde_json` (save/load system).

### 3. Red-Black Tree Dictionary (Project 3)
A memory-safe Red-Black Tree implementation with an emphasis on interoperability.
- **Key Features**: Efficient dictionary operations, rigorous trait implementations, and C-compatible bindings (`cdylib`).
- **Tech Stack**: `libc` (FFI), Rust FFI (Foreign Function Interface).

## 🧪 Lab Exercises

A series of weekly labs covering the core concepts of the Rust language:

- **Lab 1-2**: Ownership, borrowing, slices, and basic structs.
- **Lab 3-5**: Enums, pattern matching, error handling, and generic types.
- **Lab 7-8**: Closures, iterators, and advanced collection manipulation.
- **Lab 10-12**: Multi-threading, shared state, trait objects, and system-level programming.

## 🛠️ Requirements

- **Rust**: Edition 2021/2024
- **Cargo**: Standard build tool for dependency management.
- **Dependencies**: Each project and lab contains its own `Cargo.toml` with specific requirements (e.g., Bevy requires appropriate system libraries for graphics and audio).

---
*Completed as part of the Rust Programming course curriculum.*
