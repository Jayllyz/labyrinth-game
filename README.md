# 🧩 Labyrinth Game

[![CI](https://github.com/Jayllyz/labyrinth-game/actions/workflows/ci.yaml/badge.svg)](https://github.com/Jayllyz/labyrinth-game/actions/workflows/ci.yaml)
[![E2E Tests](https://github.com/Jayllyz/labyrinth-game/actions/workflows/test-e2e.yaml/badge.svg)](https://github.com/Jayllyz/labyrinth-game/actions/workflows/test-e2e.yaml)
![License](https://img.shields.io/github/license/Jayllyz/labyrinth-game?style=flat-square)

A multiplayer maze solving game built in Rust. Players navigate through procedurally generated mazes and compete to find the exit using various algorithms.

> **Note**: The server is not yet implemented. Until then, you can use the server binary in the `e2e/` directory.

## 📖 Table of Contents

- [🧩 Labyrinth Game](#-labyrinth-game)
  - [📖 Table of Contents](#-table-of-contents)
  - [✨ Features](#-features)
  - [🎮 How to Play](#-how-to-play)
    - [Prerequisites](#prerequisites)
    - [Starting the Server](#starting-the-server)
    - [Starting the Client](#starting-the-client)
  - [🏗️ Architecture](#️-architecture)
  - [🧮 Algorithms](#-algorithms)
    - [Maze Generation](#maze-generation)
    - [Maze Solving](#maze-solving)
  - [📄 License](#-license)

## ✨ Features

- **Procedurally generated mazes**: Using the Sidewinder algorithm with customizable seed values
- **Multiple solving algorithms**: 
  - Tremeaux algorithm
  - Right-hand wall following
  - Breadth-First Search (BFS)
  - A* (A-Star) pathfinding
- **Multiplayer support**: Run multiple agents simultaneously to solve the maze
- **Terminal User Interface (TUI)**: Real-time visualization of maze solving progress

## 🎮 How to Play

### Prerequisites

- Rust 1.85.0 or higher

### Starting the Server

The server hosts the game and manages connections from clients.

```bash
# In e2e/ directory
./server run --maze 100,100
```

Or you can run the minimal server (for testing purposes) with the following command:

```bash
cargo run -p server --release
```

### Starting the Client

The client connects to the server and runs the maze-solving agents.

```bash
# Basic usage with Terminal UI
cargo run -p client --release -- --tui

# Select solving algorithm
cargo run -p client --release -- --algorithm Tremeaux  # Options: Tremeaux, WallFollower, Alian

# Or using prebuilt binaries
./client --tui
./client --algorithm Tremeaux
```

## 🏗️ Architecture

The project is divided into the following components:

- **client**: Implements the game client and solving algorithms
- **server**: Implements the minimal server for testing purposes
- **e2e**: End-to-end tests for the client with the complete server
- **shared**: Contains common code shared between client and server
- **benchmarks**: Performance benchmarks for various internal components

## 🧮 Algorithms

### Maze Generation

- **Sidewinder**: Creates mazes with a bias toward horizontal passages

### Maze Solving

- **Tremeaux**: A depth-first algorithm that marks each passage when it is used
- **Alian**: An optimized version of the Tremeaux algorithm *(default)*
- **Right-hand Wall Following**: Always keeps the right hand on the wall
- **BFS (Breadth-First Search)**: Finds the shortest path in unweighted mazes
- **A\* (A-Star)**: Finds the shortest path using heuristics

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Based on the rules from [haveneer-training/sauve_qui_peut](https://github.com/haveneer-training/sauve_qui_peut).
