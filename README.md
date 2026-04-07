# 🧠 MemIgnite 🚀

**Redis-inspired high-performance in-memory key-value database engine built in Rust**

MemIgnite is a Redis-inspired **high-performance in-memory key-value database engine** built in **Rust** for educational and systems engineering purposes.

The project was designed to deeply understand how real-world cache engines and in-memory databases work internally — covering:

- async networking
- concurrent state management
- durability
- crash recovery
- TTL scheduling
- background compaction
- LRU eviction
- observability
- benchmarking
- graceful shutdown

---

# 🎯 Project Vision

MemIgnite was built as a **database internals + backend systems learning project**.

The goal was to explore how production systems like Redis balance:

- speed ⚡
- durability 💾
- concurrency 🧵
- memory control 🧠
- graceful operations 🔄

while keeping the codebase **clean, readable, and beginner-friendly**.

---

# ✨ Core Features

## ⚡ Networking & Command Engine
- Async TCP server using **Tokio**
- Multi-client concurrent connections
- Custom command parser
- Interactive CLI via `nc`
- Beginner-friendly command syntax

## 🗄️ Data Structures
- String key-value storage
- List support using `LPUSH` and `RDROP`
- Concurrent storage with **DashMap**

## ⏳ TTL & Expiration
- `EX` and `EXAT` support
- Background expiration scheduler
- Min-heap based expiry tracking
- Lazy expiration validation on reads

## 💾 Durability & Recovery
- Append Only File (**AOF**) persistence
- Configurable fsync policies:
  - `Always`
  - `EverySec`
  - `No`
- Startup replay recovery
- Snapshot-based background AOF rewrite

## 🚀 Performance Optimizations
- Background AOF write queue using Tokio `mpsc`
- Producer-consumer write pipeline
- Reduced lock contention
- Better concurrent write throughput

## 🧠 Cache Features
- LRU eviction with max key limit
- Access-time metadata tracking
- Automatic least-recently-used key eviction

## 📊 Observability
- Runtime `INFO` stats
- Command counters
- Client connection counters
- Rewrite counters
- Uptime tracking

## 🛑 Reliability
- Graceful shutdown via `Ctrl+C`
- Final AOF flush before exit
- Clean persistence guarantees

---

# 🛠️ Tech Stack

- **Language:** Rust
- **Async Runtime:** Tokio
- **Concurrent Store:** DashMap
- **Persistence:** Append Only File (AOF)
- **Channels / Queue:** Tokio mpsc
- **Benchmarking:** Python (socket + threading)
- **Client Testing:** netcat (`nc`)
- **OS Support:** macOS / Linux / Windows (WSL)

### 🔍 Why DashMap?
DashMap was chosen for **sharded concurrent reads/writes** with much lower contention than a single global mutex.

Benefits:
- multiple clients can access different keys in parallel
- improves throughput under mixed workloads
- scales better in write-heavy benchmarks
- ideal for multi-client TCP workloads

---

# 📁 Project Structure

```text
src/
├── main.rs        # bootstrap, AOF queue, graceful shutdown
├── server.rs      # TCP listener + connection lifecycle
├── handler.rs     # command execution per client
├── parser.rs      # raw text → Command enum
├── command.rs     # supported command definitions
├── store.rs       # DashMap store, TTL, LRU, AOF enqueue
├── aof.rs         # append, recovery, rewrite, fsync policies
├── stats.rs       # runtime metrics and uptime
└── benchmark.py   # multi-client benchmark workloads
```

---

# 🧠 Supported Commands

```text
SET <key> <value>
SET <key> <value> EX <seconds>
SET <key> <value> EXAT <unix_timestamp>
GET <key>
DEL <key>

LPUSH <key> <value>
RDROP <key>

INFO
REWRITEAOF

PING
ECHO <message>
HELP
QUIT
```

---

# 🏗️ Architecture Diagram

```text
                        ┌────────────────────┐
                        │    TCP Clients      │
                        │  nc / benchmark.py  │
                        └─────────┬──────────┘
                                  │
                                  ▼
                        ┌────────────────────┐
                        │ Tokio TCP Listener  │
                        │   async accept()    │
                        └─────────┬──────────┘
                                  │
                                  ▼
                        ┌────────────────────┐
                        │   Command Parser    │
                        │ parse_command()     │
                        └─────────┬──────────┘
                                  │
                                  ▼
               ┌────────────────────────────────────────┐
               │                Store                    │
               │  DashMap + TTL Heap + LRU Metadata     │
               └──────┬───────────────┬────────────────┘
                      │               │
                      │               └──────────────┐
                      ▼                              ▼
         ┌────────────────────┐         ┌────────────────────────┐
         │ Expiration Worker  │         │   INFO / Metrics       │
         │ Background cleanup │         │ Stats + Uptime         │
         └────────────────────┘         └────────────────────────┘
                      │
                      ▼
         ┌──────────────────────────────┐
         │   AOF Queue (mpsc channel)   │
         │ producer-consumer pipeline    │
         └──────────────┬───────────────┘
                        │
                        ▼
         ┌──────────────────────────────┐
         │ Background AOF Writer Task   │
         │ async append + fsync policy  │
         └──────────────┬───────────────┘
                        │
                        ▼
         ┌──────────────────────────────┐
         │      appendonly.aof          │
         │ crash recovery source        │
         └──────────────────────────────┘
```

---

# 🗺️ DashMap Concurrency Architecture

```text
                    ┌──────────────────────────────┐
                    │         DashMap Store         │
                    │   sharded concurrent hashmap  │
                    └──────────────┬───────────────┘
                                   │
        ┌───────────────┬──────────┼──────────┬───────────────┐
        ▼               ▼          ▼          ▼               ▼
   ┌────────┐      ┌────────┐ ┌────────┐ ┌────────┐     ┌────────┐
   │Shard 0 │      │Shard 1 │ │Shard 2 │ │Shard 3 │ ... │Shard N │
   └────┬───┘      └────┬───┘ └────┬───┘ └────┬───┘     └────┬───┘
        │               │          │          │               │
        ▼               ▼          ▼          ▼               ▼
   different keys can be read/written in parallel by different clients
```

### 🔍 Why DashMap improves MemIgnite
Instead of protecting the full store with one global mutex, DashMap internally splits data into multiple shards.

This significantly reduces contention for:
- mixed workloads
- write-heavy benchmarks
- multi-client stress tests

---

# 📜 AOF Persistence Architecture

```text
Write Command
   ↓
Store updates in-memory DashMap
   ↓
Raw command pushed into mpsc AOF queue
   ↓
Background writer task consumes queue
   ↓
appendonly.aof
   ↓
Crash recovery replays commands on startup
   ↓
Background REWRITEAOF compacts file using store snapshot
```

### 🔍 Why AOF matters
The Append Only File is MemIgnite’s durability layer.

It ensures:
- writes survive process restarts
- crash recovery restores database state
- rewrite removes redundant historical commands
- fsync policy balances durability vs throughput

---

# 💾 Storage Flow

```text
Client Command
   ↓
Parser converts raw input into Command enum
   ↓
Handler validates and routes operation
   ↓
Store updates DashMap in-memory state
   ↓
TTL metadata updated (if EX / EXAT)
   ↓
LRU access metadata updated
   ↓
AOF command sent to mpsc queue
   ↓
Background writer persists to appendonly.aof
   ↓
Fsync policy decides flush behavior
```

---

# 🚀 Getting Started

## Run the server
```bash
cargo run
```

## Connect as client
```bash
nc 127.0.0.1 6379
```

---

# 📌 Example Session

```text
SET name raj
OK

GET name
raj

SET session token123 EX 60
OK

LPUSH nums 1
1

RDROP nums
1

INFO
REWRITEAOF
```

---

# 📈 Benchmark Results

Validated with **100 concurrent clients × 1000 operations each**.

| Workload | Throughput | Avg Latency | P95 Latency |
|---|---:|---:|---:|
| Write-heavy | 9.2k ops/sec | 10.6 ms | 21 ms |
| Mixed | 6.0k ops/sec | 16.3 ms | 31 ms |
| Read-heavy | 3.5k ops/sec | 28.2 ms | 48 ms |

### 🔍 Observation
Read-heavy appears slower because `INFO` performs full-store scans, making it more expensive than normal point reads.

---

# 📚 Engineering Concepts Explored

- async TCP server design
- concurrent shared state
- lock contention reduction
- producer-consumer architecture
- durability vs performance trade-offs
- WAL / AOF persistence
- crash recovery
- snapshot-based background compaction
- LRU eviction strategies
- graceful shutdown coordination
- benchmark-driven optimization

---

# 🎯 Future Improvements

- RESP protocol support
- Redis CLI compatibility
- RDB-style periodic snapshots
- follower replication
- shard-aware clustering
- O(1) linked LRU
- advanced list commands
- pipelining support

---

# 📚 Educational Purpose

MemIgnite is built primarily for:
- learning Rust async systems programming
- understanding database internals
- practicing backend performance engineering
- building recruiter-friendly systems projects

---

# 👨‍💻 Author Note

This project was designed and implemented by **Saiyogiraj** as a hands-on systems engineering and database internals learning project.

While inspired by Redis design principles, **MemIgnite is an original educational implementation built from scratch in Rust** with:

- clean architecture
- beginner-friendly systems concepts
- measurable performance validation

It serves as both a **deep learning exercise** and a **portfolio-grade backend systems project**.
