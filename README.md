<p align="center">
  <img src="memignite.png" width="180" />
</p>

<h1 align="center">MemIgnite</h1>
<p align="center">
  🧠 In-Memory Key-Value Engine Built in Rust
</p>
---

## Overview

MemIgnite is a high-performance in-memory key-value database built from scratch in Rust using Tokio for async networking.

It supports TTL (time-to-live), append-only persistence (AOF), restart-safe expiration recovery, and concurrent multi-client handling over raw TCP.

The project focuses on correctness, concurrency safety, deterministic restart behavior, and architectural clarity.

---

## Core Features

- Async TCP server (Tokio runtime)
- In-memory key-value store
- TTL support (`EX` and `EXAT`)
- Absolute timestamp expiration
- Lazy expiration model
- Append-only file (AOF) persistence
- Deterministic state rebuild on restart
- Concurrent client handling
- Fully tested core logic

---

## Architecture Overview

Client  
↓  
TCP Server (`tokio::net::TcpListener`)  
↓  
Connection Handler  
↓  
Command Parser  
↓  
Store (HashMap + TTL metadata)  
↓  
Append-Only Log (AOF)

---

## TTL Design

MemIgnite implements TTL using absolute expiry timestamps.

Supported options:
- `EX <seconds>` → converted internally to absolute timestamp
- `EXAT <unix_timestamp>`

Internal model:
- `EX` is converted to `EXAT`
- Absolute timestamps are stored
- On restart, remaining TTL is recalculated
- Expired keys are skipped during AOF replay

This prevents expired keys from being resurrected after restart.

---

## Expiration Model

MemIgnite uses lazy expiration:

- No background cleanup thread
- Expiration is checked during key access
- Expired keys are deleted on `GET`

This keeps the system simple and avoids unnecessary timers or scheduling overhead.

---

## Persistence Model

MemIgnite uses an append-only file (AOF):

- Every write operation is appended to disk
- On restart, commands are replayed sequentially
- TTL state is restored deterministically

### Crash Consistency Trade-Off

Memory is updated before AOF append.

If a crash occurs between:
1. Memory update
2. Log append

Recent writes may be lost.

This is a deliberate simplification. Production systems typically use write-ahead logging with fsync guarantees for stronger durability.

---

## Concurrency Model

MemIgnite ensures safe concurrent access using:

- `Arc` for shared ownership
- `RwLock` for concurrent reads
- `Mutex` for serialized AOF writes
- Async-safe locking discipline

Multiple clients can connect simultaneously without data races.

---

## Libraries Used

- `tokio` – Async runtime and TCP networking
- `std::collections::HashMap` – Core data storage
- `Arc`, `RwLock`, `Mutex` – Concurrency primitives
- `SystemTime`, `Duration` – TTL computation
- Standard Rust error propagation (`Result` + `?`)

No external database libraries were used.

---

## Running the Server

Start the server:

    cargo run

Connect using:

    nc 127.0.0.1 6379

Example commands:

    SET key value
    SET key value EX 10
    SET key value EXAT 1700000000
    GET key
    DEL key
    PING

---

## Tests

Core functionality is verified using:

    cargo test

Test coverage includes:
- Basic set/get
- Delete operations
- TTL expiration
- Lazy deletion behavior
- Restart-safe TTL replay
- Expired key skipping during recovery

---

## Design Scope

MemIgnite focuses on core in-memory database mechanics and correctness.

It intentionally does not implement:

- Replication
- Clustering
- Write-ahead fsync guarantees
- Background expiration sweeper
- Binary protocol
- Benchmark tooling

The goal of this project is architectural clarity and correctness of fundamental database behavior.

---

## Potential Extensions

MemIgnite can be extended with:

- Write-ahead logging with fsync for stronger durability
- Active expiration sweeper
- Replication (leader–follower)
- Binary protocol support
- Performance benchmarking suite
- Memory usage optimization

---

## Author

MemIgnite was built as a systems-focused backend engineering project in Rust to explore async networking, concurrency, persistence models, and crash-consistent design.
