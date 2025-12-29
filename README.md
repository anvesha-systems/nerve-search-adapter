# nerve-search-adapter

Search capability adapter for the NERVE protocol

⸻

## Overview

nerve-search-adapter is a byte-native capability adapter that connects to nerve-core and executes search queries on behalf of higher-level systems such as agents, AI pipelines, or user interfaces.

It translates SEARCH_QUERY protocol messages into real search execution and streams SEARCH_RESULT messages back to the core with low latency and immediate cancellation support.

This repository intentionally contains no agent logic, no AI logic, and no UI code.

⸻

## Design Philosophy

- **Bytes-first, meaning-last**
- **Single responsibility**
- **Low latency over abstraction**
- **Fail fast, fail loud**
- **No protocol changes**

The adapter treats all payloads as opaque bytes.
Text decoding and interpretation are strictly deferred to higher layers.

⸻

## Responsibilities (v0.1.0)

The adapter:
- Connects to nerve-core via Unix Domain Socket
- Listens for SEARCH_QUERY messages
- Executes search (stubbed or delegated)
- Sends SEARCH_RESULT replies
- Honors CANCEL messages immediately
- Exits cleanly if nerve-core is unavailable

⸻

## Non-Goals (Explicit)

The adapter does not:
- Act as a server
- Accept multiple clients
- Perform agent planning or reasoning
- Run AI models
- Render UI
- Modify protocol framing or semantics
- Route messages between multiple parties

⸻

## Architecture

```
[ Agent / CLI / UI ]
        │
        ▼
   nerve-core
        │
        ▼
nerve-search-adapter
        │
        ▼
   Search Engine
```

- **nerve-core** owns the socket and lifecycle
- **The adapter** is a single client
- **Search logic** is delegated, not embedded

⸻

## Repository Structure

```
nerve-search-adapter/
├── src/
│   ├── main.rs       # bootstrap only
│   ├── client.rs     # core IPC loop
│   ├── handler.rs    # SEARCH_QUERY handling
│   └── state.rs      # request + cancel tracking
│
├── tests/
│   └── integration.rs
│
└── Cargo.toml
```

⸻

## Supported Message Types

| Message Type  | Behavior                  |
|---------------|---------------------------|
| SEARCH_QUERY  | Executes search and replies |
| CANCEL        | Cancels in-flight request |
| Others        | Ignored safely            |

Payload semantics are opaque at this layer.

⸻

## Cancellation Semantics

- Cancellation is best-effort and immediate
- Cancelled requests do not emit results
- Cancellation does not affect other requests

This behavior is critical for agentic automation.

⸻

## Running the Adapter

### Prerequisite

nerve-core must be running.

```bash
cargo run
```

The adapter attempts to connect to:

```
/tmp/nerve.sock
```

If the core is not available, the adapter exits with an error.

⸻

## Testing Strategy

Tests validate:
- Adapter lifecycle (connect / disconnect)
- Single-client invariant
- Correct handling of SEARCH_QUERY
- Proper cancellation behavior
- Protocol framing integrity

Tests do not assume multi-client routing.

Run integration tests:

```bash
cargo test --test integration -- --test-threads=1
```

⸻

## Versioning

This repository follows semantic versioning.

### Current version

**v0.1.0**

This version guarantees:
- compatibility with nerve-core v0.1.0
- stable adapter behavior
- no protocol changes

⸻

## Relationship to Other Repositories

### nerve-protocol
Defines wire format, framing, and message types.

### nerve-core
Core daemon and lifecycle manager.

### Agent runtimes / AI layers (future)
Built on top of this adapter.

⸻

## Philosophy

This adapter is intentionally simple.

**All intelligence belongs above this layer.**
**All complexity belongs outside this layer.**

⸻

## Status

Stable for integration. Ready for real search backends.

⸻

