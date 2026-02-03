# Mini Redis (Rust)

A simplified Redis-like in-memory key-value store built in Rust to understand
backend systems, async networking, concurrency, caching, and persistence.

---

## 🚀 Features
- Async TCP server using Tokio
- Handles multiple clients concurrently
- In-memory key-value storage
- Thread-safe shared state
- TTL (Time-To-Live) support with lazy expiration
- Simple text-based command protocol

Supported commands:
- PING
- ECHO <message>
- SET <key> <value>
- SET <key> <value> EX <seconds>
- GET <key>
- DEL <key>

---

## 🧠 Design Decisions
- TTL implemented using **expiry timestamps**, not timers
- Uses **monotonic clock (`Instant`)** to avoid time-skew issues
- Lazy expiration ensures correctness with minimal overhead
- Shared state managed via `Arc` + async `RwLock`
- Each client handled in an independent async task

---

## 🏗️ Architecture Overview
- `server.rs` – TCP listener and connection handling
- `handler.rs` – Client request processing
- `parser.rs` – Command parsing
- `store.rs` – In-memory storage and TTL logic
- `command.rs` – Command definitions

---

## ▶️ Running the Server
```bash
cargo run

nc 127.0.0.1 6379

SET name Saiyogiraj EX 5
GET name

