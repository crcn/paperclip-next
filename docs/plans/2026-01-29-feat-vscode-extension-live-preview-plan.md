---
title: feat: VSCode Extension with Live Preview and Direct Streaming
type: feat
date: 2026-01-29
deepened: 2026-01-29
hardened: 2026-01-29
---

# VSCode Extension with Live Preview and Direct Streaming

## Enhancement Summary

**Deepened on:** 2026-01-29
**Hardened on:** 2026-01-29
**Research phases:** 3 (Deep review, Threat model, Pressure test)
**Research agents used:** kieran-typescript-reviewer, performance-oracle, security-sentinel, architecture-strategist, code-simplicity-reviewer, julik-frontend-races-reviewer, pattern-recognition-specialist, framework-docs-researcher, best-practices-researcher

### Key Improvements (Phase 1: Deep Review)

1. **Security Hardening**: TLS credentials, WebView CSP policies, path traversal prevention, rate limiting
2. **Architecture Refinement**: Redesigned bidirectional → unidirectional streaming, consolidated state management
3. **Performance Optimization**: Realistic latency (150-300ms), connection pooling, backpressure handling
4. **Type Safety**: Eliminated `any` types, proper return types, discriminated unions
5. **Race Condition Prevention**: `inFlight` flags, subscription tracking, disposal order
6. **Simplification**: 490 LOC (31%) deferred to post-MVP
7. **Memory Leak Prevention**: Proper disposal patterns, cancellation tokens
8. **Best Practices**: VSCode extension patterns, gRPC streaming strategies

### Critical Additions (Phase 2: Threat Model)

9. **Explicit State Teardown**: ClosePreview RPC for guaranteed cleanup
10. **Resource Caps**: Total VDOM memory limit (500MB), max client states (100)
11. **Enhanced Path Security**: Symlink detection, Unicode normalization, Windows drive validation
12. **Parser/Evaluator Safeguards**: Timeouts (5s), depth limits (50), node limits (10,000)
13. **Process-Level Rate Limiting**: Per-PID request tracking (100/min)
14. **Capability-Based Auth**: Secret tokens for client authentication (post-MVP)

### Reliability Enhancements (Phase 3: Pressure Test)

15. **Preview Pool**: Max 10 active WebViews with LRU eviction
16. **Heartbeat + Cleanup**: 5-minute client timeout, automatic state pruning
17. **Transactional Patches**: Rollback on partial update failure
18. **Reconnection Strategy**: Exponential backoff with jitter, force Initialize on reconnect
19. **File System Priority**: Buffer updates always win over disk events
20. **Version-Based Sync**: Detect state mismatches, force full resync

### Future-Proofing (Phase 3: Semantic ID Protocol)

21. **Semantic Patch Paths**: `{ kind: "semantic", id: SemanticID }` for stable node identity
22. **Move Operations**: Explicit `MoveChild` patch for reordering (no full re-render)
23. **Migration Path**: Dual-mode support (positional + semantic) for gradual rollout

---

## Overview

Build a VSCode extension that provides live, keystroke-level preview of Paperclip (`.pc`) files using direct gRPC streaming. The extension connects to the workspace server, streams text changes directly (bypassing file I/O), and renders VDOM updates in a WebView panel.

### Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│                    VSCode Extension                             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Extension Host (Node.js)                                  │  │
│  │                                                            │  │
│  │  • PreviewManager (max 10 active, LRU eviction)          │  │
│  │  • Shared WorkspaceClient (connection pooling)           │  │
│  │  • BufferStreamer (150ms debounce, inFlight guard)       │  │
│  │  • Cancellation token support                            │  │
│  └────┬─────────────────────────────────────┬───────────────┘  │
│       │ gRPC (TLS in prod)                  │ postMessage()    │
└───────┼─────────────────────────────────────┼──────────────────┘
        │                                     │
┌───────▼─────────────────────────────────┐  │
│ Workspace Server (Rust)                 │  │
│                                         │  │
│ • StreamBuffer RPC (unidirectional)    │  │
│ • ClosePreview RPC (explicit cleanup)  │  │
│ • Per-client VDOM state (max 100)      │  │
│ • Total memory cap (500MB)             │  │
│ • Heartbeat tracking (5min timeout)    │  │
│ • Parse/eval timeouts (5s)             │  │
│ • Depth limits (50), node limits (10k) │  │
│ • Semantic ID generation               │  │
│ • Process-level rate limiting          │  │
└─────────────────────────────────────────┘  │
                                             │
        ┌────────────────────────────────────▼──────────┐
        │ WebView Panel (Strict CSP)                    │
        │                                                │
        │ • Nonce-based script/style loading            │
        │ • Transactional patch application             │
        │ • RAF batching                                │
        │ • Semantic ID navigation                      │
        │ • Version-based state sync                    │
        │ • Visibility-aware replay                     │
        └────────────────────────────────────────────────┘
```

### Performance Expectations

**Realistic latency breakdown:**
- Parse: 1-25µs ✅ (benchmarked)
- Evaluate: 1-10µs ✅ (benchmarked)
- Serialize: 10-50ms
- Network (localhost): 1-5ms
- Deserialize: 10-50ms
- Render: 16-50ms

**Total: 150-300ms** (feels instant for live preview)

### Resource Limits (Production Config)

```rust
pub const MAX_CLIENT_STATES: usize = 100;
pub const MAX_TOTAL_VDOM_BYTES: usize = 500 * 1024 * 1024;  // 500MB
pub const CLIENT_TIMEOUT_SECS: u64 = 300;  // 5 minutes
pub const MAX_COMPONENT_DEPTH: usize = 50;
pub const MAX_VDOM_NODES: usize = 10_000;
pub const PARSE_TIMEOUT_SECS: u64 = 5;
pub const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;  // 10MB
pub const RATE_LIMIT_PER_PROCESS: usize = 100;  // per minute
```

```typescript
export const MAX_ACTIVE_WEBVIEWS = 10;
export const RECONNECT_JITTER_MS = 200;  // ±200ms random
export const HEARTBEAT_INTERVAL_MS = 60_000;  // 1 minute
```

---

## Problem Statement / Motivation

**Current Development Flow:**
1. Edit `.pc` files in VSCode
2. Save to disk
3. Workspace server detects file change via `notify` watcher
4. Open browser preview to see changes
5. Context switch between editor and browser

**Pain Points:**
- File-based sync requires save operation (latency + friction)
- Context switching breaks flow state
- Can't easily share live preview with remote collaborators
- No tight editor integration for component preview

**Goal:**
Create a seamless editing experience where:
- ✨ **Keystroke-level updates** - Changes stream directly without saving
- 🔄 **Instant feedback** - Preview updates in <300ms (realistic)
- 🪟 **Integrated preview** - WebView panel next to code editor
- 🔌 **Reliable at scale** - Handles 100+ client connections gracefully
- 🔒 **Production-ready security** - Hardened against DoS, path traversal, state pollution

---

## Proposed Solution

### Phase 1: Foundation & Server Hardening (Week 1)

#### 1.1: Enhanced Proto Definition

**File: `proto/workspace.proto`**
```protobuf
service WorkspaceService {
  // Existing RPCs
  rpc StreamPreview(PreviewRequest) returns (stream PreviewUpdate);
  rpc WatchFiles(WatchRequest) returns (stream FileEvent);
  rpc ApplyMutation(MutationRequest) returns (MutationResponse);
  rpc GetDocumentOutline(OutlineRequest) returns (OutlineResponse);

  // NEW: Unidirectional buffer streaming
  rpc StreamBuffer(BufferRequest) returns (stream PreviewUpdate);

  // NEW: Explicit state cleanup
  rpc ClosePreview(ClosePreviewRequest) returns (ClosePreviewResponse);

  // NEW: Heartbeat for liveness tracking
  rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
}

message BufferRequest {
  string client_id = 1;
  string file_path = 2;
  string content = 3;

  // Version-based sync
  optional uint64 expected_state_version = 4;
}

message ClosePreviewRequest {
  string client_id = 1;
}

message ClosePreviewResponse {
  bool success = 1;
  optional string message = 2;
}

message HeartbeatRequest {
  string client_id = 1;
}

message HeartbeatResponse {
  bool acknowledged = 1;
  uint64 server_time = 2;
}

message PreviewUpdate {
  string file_path = 1;
  repeated Patch patches = 2;
  optional string error = 3;
  uint64 timestamp = 4;

  // NEW: State version for sync
  uint64 state_version = 5;
}

message Patch {
  oneof patch_type {
    InitializePatch initialize = 1;
    SetAttributePatch set_attribute = 2;
    SetTextPatch set_text = 3;
    InsertChildPatch insert_child = 4;
    RemoveChildPatch remove_child = 5;
    MoveChildPatch move_child = 6;  // NEW: For semantic reordering
  }
}

message PatchPath {
  oneof path_kind {
    string positional = 1;  // "root/0/1/2"
    string semantic = 2;    // "Card{Card-0}::div[a8b3]"
  }
}

message InitializePatch {
  VDocument vdom = 1;
}

message SetAttributePatch {
  PatchPath path = 1;
  string key = 2;
  string value = 3;
}

message SetTextPatch {
  PatchPath path = 1;
  string value = 2;
}

message InsertChildPatch {
  PatchPath parent = 1;
  uint32 index = 2;
  VNode child = 3;
  optional string child_semantic_id = 4;
}

message RemoveChildPatch {
  PatchPath parent = 1;
  string child_semantic_id = 2;  // Remove by semantic ID
}

message MoveChildPatch {
  PatchPath parent = 1;
  string child_semantic_id = 2;
  uint32 from_index = 3;
  uint32 to_index = 4;
}
```

**Tasks:**
- [ ] Add `BufferRequest` with version field
- [ ] Add `ClosePreviewRequest/Response`
- [ ] Add `HeartbeatRequest/Response`
- [ ] Add `state_version` to `PreviewUpdate`
- [ ] Add `MoveChildPatch` for semantic reordering
- [ ] Add `PatchPath` enum (positional + semantic)

---

#### 1.2: Server Implementation with Hardening

**File: `packages/workspace/src/server.rs`**

```rust
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};
use tonic::{Request, Response, Status, Streaming};

// Configuration constants
const MAX_CLIENT_STATES: usize = 100;
const MAX_TOTAL_VDOM_BYTES: usize = 500 * 1024 * 1024;  // 500MB
const CLIENT_TIMEOUT_SECS: u64 = 300;  // 5 minutes
const MAX_COMPONENT_DEPTH: usize = 50;
const MAX_VDOM_NODES: usize = 10_000;
const PARSE_TIMEOUT_SECS: u64 = 5;
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;  // 10MB
const RATE_LIMIT_PER_PROCESS: usize = 100;  // per minute

pub struct ClientState {
    vdom: VDocument,
    version: u64,
    last_update: Instant,
    file_path: String,
}

pub struct ProcessRateLimiter {
    requests_per_process: HashMap<u32, VecDeque<Instant>>,
    max_requests_per_minute: usize,
}

impl ProcessRateLimiter {
    pub fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests_per_process: HashMap::new(),
            max_requests_per_minute,
        }
    }

    pub fn check(&mut self, pid: u32) -> Result<(), Status> {
        let now = Instant::now();
        let requests = self.requests_per_process.entry(pid).or_default();

        // Remove requests older than 1 minute
        requests.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

        if requests.len() >= self.max_requests_per_minute {
            return Err(Status::resource_exhausted(
                format!("Process {} exceeded rate limit", pid)
            ));
        }

        requests.push_back(now);
        Ok(())
    }
}

pub struct WorkspaceServiceImpl {
    bundle: Arc<Bundle>,
    client_states: Arc<Mutex<HashMap<String, ClientState>>>,
    client_heartbeats: Arc<Mutex<HashMap<String, Instant>>>,
    total_vdom_bytes: Arc<AtomicUsize>,
    workspace_root_canonical: PathBuf,
    rate_limiter: Arc<Mutex<ProcessRateLimiter>>,
}

impl WorkspaceServiceImpl {
    pub fn new(workspace_root: &str, bundle: Bundle) -> Self {
        let workspace_root_canonical = PathBuf::from(workspace_root)
            .canonicalize()
            .expect("Failed to resolve workspace root");

        let service = Self {
            bundle: Arc::new(bundle),
            client_states: Arc::new(Mutex::new(HashMap::new())),
            client_heartbeats: Arc::new(Mutex::new(HashMap::new())),
            total_vdom_bytes: Arc::new(AtomicUsize::new(0)),
            workspace_root_canonical,
            rate_limiter: Arc::new(Mutex::new(
                ProcessRateLimiter::new(RATE_LIMIT_PER_PROCESS)
            )),
        };

        // Start background cleanup task
        service.start_cleanup_task();

        service
    }

    fn start_cleanup_task(&self) {
        let heartbeats = self.client_heartbeats.clone();
        let states = self.client_states.clone();
        let total_bytes = self.total_vdom_bytes.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let now = Instant::now();
                let mut heartbeats = heartbeats.lock().unwrap();
                let mut states = states.lock().unwrap();

                // Remove stale clients
                heartbeats.retain(|client_id, last_heartbeat| {
                    let is_stale = now.duration_since(*last_heartbeat)
                        > Duration::from_secs(CLIENT_TIMEOUT_SECS);

                    if is_stale {
                        if let Some(state) = states.remove(client_id) {
                            let size = estimate_vdom_size(&state.vdom);
                            total_bytes.fetch_sub(size, Ordering::Relaxed);
                            warn!("Removed stale client: {}", client_id);
                        }
                        false
                    } else {
                        true
                    }
                });
            }
        });
    }

    fn ensure_capacity(&self, new_vdom_size: usize) -> Result<(), Status> {
        // Check total memory limit
        let current = self.total_vdom_bytes.load(Ordering::Relaxed);
        if current + new_vdom_size > MAX_TOTAL_VDOM_BYTES {
            return Err(Status::resource_exhausted(
                "Total VDOM memory limit exceeded"
            ));
        }

        // Check client count limit
        let states = self.client_states.lock().unwrap();
        if states.len() >= MAX_CLIENT_STATES {
            // Evict LRU client
            if let Some((lru_id, lru_state)) = states.iter()
                .min_by_key(|(_, state)| state.last_update) {
                let lru_id = lru_id.clone();
                let lru_size = estimate_vdom_size(&lru_state.vdom);
                drop(states);

                let mut states = self.client_states.lock().unwrap();
                states.remove(&lru_id);
                self.total_vdom_bytes.fetch_sub(lru_size, Ordering::Relaxed);
                warn!("Evicted LRU client due to capacity: {}", lru_id);
            }
        }

        Ok(())
    }

    fn validate_path(&self, file_path: &str) -> Result<PathBuf, Status> {
        use unicode_normalization::UnicodeNormalization;

        // 1. Unicode normalization
        let normalized = file_path.nfc()
            .collect::<String>()
            .replace('\u{2215}', "/")  // Division slash
            .replace('\u{2044}', "/"); // Fraction slash

        let path = PathBuf::from(normalized);

        // 2. Canonicalize (follows symlinks)
        let canonical = path.canonicalize()
            .map_err(|e| Status::invalid_argument(format!("Path error: {}", e)))?;

        // 3. Check within workspace
        if !canonical.starts_with(&self.workspace_root_canonical) {
            return Err(Status::permission_denied(
                "Path escapes workspace"
            ));
        }

        // 4. Check for symlinks in path components
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            let metadata = fs::symlink_metadata(&current)
                .map_err(|e| Status::invalid_argument(format!("Path check: {}", e)))?;

            if metadata.is_symlink() {
                return Err(Status::permission_denied(
                    "Symlinks not allowed in file path"
                ));
            }

            current = parent.to_path_buf();
        }

        Ok(canonical)
    }
}

impl WorkspaceService for WorkspaceServiceImpl {
    type StreamBufferStream = ReceiverStream<Result<PreviewUpdate, Status>>;

    async fn stream_buffer(
        &self,
        request: Request<BufferRequest>,
    ) -> Result<Response<Self::StreamBufferStream>, Status> {
        let req = request.into_inner();

        // Rate limiting
        let pid = get_peer_pid(&request)?;
        self.rate_limiter.lock().unwrap().check(pid)?;

        // Validate content size
        if req.content.len() > MAX_CONTENT_SIZE {
            return Err(Status::invalid_argument("Content exceeds 10MB limit"));
        }

        // Validate and canonicalize path
        let canonical_path = self.validate_path(&req.file_path)?;

        // Update heartbeat
        self.client_heartbeats.lock().unwrap()
            .insert(req.client_id.clone(), Instant::now());

        // Get previous state
        let mut states = self.client_states.lock().unwrap();

        // Version-based sync check
        if let Some(expected_version) = req.expected_state_version {
            if let Some(state) = states.get(&req.client_id) {
                if state.version != expected_version {
                    warn!("Version mismatch for {}, forcing Initialize", req.client_id);
                    states.remove(&req.client_id);
                }
            }
        }

        let prev_state = states.get(&req.client_id).cloned();
        drop(states);

        // Parse with timeout
        let content = req.content.clone();
        let parse_result = timeout(
            Duration::from_secs(PARSE_TIMEOUT_SECS),
            tokio::task::spawn_blocking(move || parse_pc(&content))
        ).await;

        let ast = match parse_result {
            Ok(Ok(Ok(ast))) => ast,
            Ok(Ok(Err(e))) => {
                return Self::send_error_response(
                    req.file_path,
                    format!("Parse error: {}", e),
                    prev_state.map(|s| s.version).unwrap_or(0)
                );
            }
            Ok(Err(_)) => {
                return Self::send_error_response(
                    req.file_path,
                    "Parser panic".to_string(),
                    prev_state.map(|s| s.version).unwrap_or(0)
                );
            }
            Err(_) => {
                return Err(Status::deadline_exceeded("Parse timeout"));
            }
        };

        // Evaluate with timeout and limits
        let bundle = self.bundle.clone();
        let eval_result = timeout(
            Duration::from_secs(PARSE_TIMEOUT_SECS),
            tokio::task::spawn_blocking(move || {
                evaluate_with_limits(&ast, &bundle, &EvaluatorConfig {
                    max_component_depth: MAX_COMPONENT_DEPTH,
                    max_vdom_nodes: MAX_VDOM_NODES,
                })
            })
        ).await;

        let vdom = match eval_result {
            Ok(Ok(Ok(vdom))) => vdom,
            Ok(Ok(Err(e))) => {
                return Self::send_error_response(
                    req.file_path,
                    format!("Eval error: {}", e),
                    prev_state.map(|s| s.version).unwrap_or(0)
                );
            }
            Ok(Err(_)) => {
                return Self::send_error_response(
                    req.file_path,
                    "Evaluator panic".to_string(),
                    prev_state.map(|s| s.version).unwrap_or(0)
                );
            }
            Err(_) => {
                return Err(Status::deadline_exceeded("Eval timeout"));
            }
        };

        // Check memory capacity
        let vdom_size = estimate_vdom_size(&vdom);
        self.ensure_capacity(vdom_size)?;

        // Generate patches
        let patches = if let Some(ref prev_state) = prev_state {
            diff_vdocument(&prev_state.vdom, &vdom)
        } else {
            vec![Patch::Initialize(vdom.clone())]
        };

        // Update state
        let new_version = prev_state.as_ref()
            .map(|s| s.version + 1)
            .unwrap_or(1);

        let mut states = self.client_states.lock().unwrap();

        // Update memory tracking
        if let Some(prev_state) = prev_state {
            let old_size = estimate_vdom_size(&prev_state.vdom);
            self.total_vdom_bytes.fetch_sub(old_size, Ordering::Relaxed);
        }
        self.total_vdom_bytes.fetch_add(vdom_size, Ordering::Relaxed);

        states.insert(req.client_id.clone(), ClientState {
            vdom,
            version: new_version,
            last_update: Instant::now(),
            file_path: req.file_path.clone(),
        });
        drop(states);

        // Stream patches (batched if many)
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            for chunk in patches.chunks(10) {
                let _ = tx.send(Ok(PreviewUpdate {
                    file_path: req.file_path.clone(),
                    patches: serialize_patches(chunk),
                    error: None,
                    timestamp: current_timestamp(),
                    state_version: new_version,
                })).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn close_preview(
        &self,
        request: Request<ClosePreviewRequest>,
    ) -> Result<Response<ClosePreviewResponse>, Status> {
        let client_id = request.into_inner().client_id;

        let mut states = self.client_states.lock().unwrap();
        let mut heartbeats = self.client_heartbeats.lock().unwrap();

        let existed = if let Some(state) = states.remove(&client_id) {
            let size = estimate_vdom_size(&state.vdom);
            self.total_vdom_bytes.fetch_sub(size, Ordering::Relaxed);
            heartbeats.remove(&client_id);
            info!("Cleaned up state for client_id: {}", client_id);
            true
        } else {
            warn!("Attempted to close non-existent client_id: {}", client_id);
            false
        };

        Ok(Response::new(ClosePreviewResponse {
            success: existed,
            message: if existed {
                Some("State cleaned up successfully".to_string())
            } else {
                Some("Client not found".to_string())
            },
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<HeartbeatResponse>, Status> {
        let client_id = request.into_inner().client_id;

        self.client_heartbeats.lock().unwrap()
            .insert(client_id, Instant::now());

        Ok(Response::new(HeartbeatResponse {
            acknowledged: true,
            server_time: current_timestamp(),
        }))
    }
}

fn estimate_vdom_size(vdom: &VDocument) -> usize {
    // Rough estimate: count nodes and multiply by average size
    fn count_nodes(node: &VNode) -> usize {
        1 + node.children.iter().map(count_nodes).sum::<usize>()
    }

    let node_count = count_nodes(&vdom.root);
    node_count * 500  // ~500 bytes per node average
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(unix)]
fn get_peer_pid(request: &Request<BufferRequest>) -> Result<u32, Status> {
    // Extract PID from peer connection (Unix domain socket metadata)
    // This is platform-specific
    use std::os::unix::io::AsRawFd;

    // Placeholder: actual implementation depends on gRPC internals
    Ok(std::process::id())
}

#[cfg(not(unix))]
fn get_peer_pid(request: &Request<BufferRequest>) -> Result<u32, Status> {
    Ok(std::process::id())
}
```

**Tasks:**
- [ ] Implement `StreamBuffer` with unidirectional streaming
- [ ] Implement `ClosePreview` for explicit cleanup
- [ ] Implement `Heartbeat` for liveness tracking
- [ ] Add periodic cleanup task (5-minute timeout)
- [ ] Add LRU eviction when client count exceeds 100
- [ ] Add total memory cap (500MB) with enforcement
- [ ] Add parse/eval timeouts (5 seconds)
- [ ] Add depth limits (50) and node limits (10,000)
- [ ] Add enhanced path validation (symlinks, Unicode, canonicalization)
- [ ] Add per-process rate limiting (100/min)
- [ ] Add version-based state sync

---

### Phase 2: Client Implementation with Reliability (Week 1-2)

#### 2.1: Shared Workspace Client

**File: `packages/vscode-extension/src/workspace-client.ts`**

```typescript
import { createWorkspaceClient, WorkspaceClient } from '@paperclip/workspace-client';
import { GrpcTransport } from '@paperclip/workspace-client/grpc';
import * as grpc from '@grpc/grpc-js';
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

let sharedClient: WorkspaceClient | undefined;
let heartbeatInterval: NodeJS.Timeout | undefined;
let connectionId: string | undefined;

export async function getSharedClient(
  extensionPath: string
): Promise<WorkspaceClient> {
  if (sharedClient?.isConnected()) {
    return sharedClient;
  }

  const protoPath = path.join(extensionPath, '../../proto/workspace.proto');

  if (!fs.existsSync(protoPath)) {
    throw new Error(`Proto file not found: ${protoPath}`);
  }

  // Security: Use TLS in production
  const credentials = process.env.NODE_ENV === 'production'
    ? grpc.credentials.createSsl(fs.readFileSync('ca.crt'))
    : grpc.credentials.createInsecure();

  const transport = new GrpcTransport({
    protoPath,
    credentials,
  });

  connectionId = `connection-${Date.now()}`;

  sharedClient = createWorkspaceClient(transport, {
    clientId: `vscode-shared-${Date.now()}`,
    autoReconnect: true,
    maxReconnectAttempts: 10,
    reconnectDelayMs: 1000,
    maxReconnectDelayMs: 30000,
  });

  await sharedClient.connect('localhost:50051');

  // Event listeners
  sharedClient.on('connected', () => {
    console.log('[WorkspaceClient] Connected');
    startHeartbeat(sharedClient!);
  });

  sharedClient.on('disconnected', () => {
    console.log('[WorkspaceClient] Disconnected');
    stopHeartbeat();
    connectionId = undefined;
  });

  sharedClient.on('connection-failed', (event) => {
    console.error('[WorkspaceClient] Connection failed:', event.error);

    // Exponential backoff with jitter
    const attempt = event.attempt || 0;
    const baseDelay = 1000;
    const maxDelay = 30000;
    const delay = Math.min(baseDelay * Math.pow(2, attempt), maxDelay);
    const jitter = delay * 0.2 * (Math.random() - 0.5);
    const totalDelay = delay + jitter;

    console.log(`[WorkspaceClient] Reconnecting in ${totalDelay}ms (attempt ${attempt})`);
  });

  return sharedClient;
}

function startHeartbeat(client: WorkspaceClient): void {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
  }

  heartbeatInterval = setInterval(async () => {
    try {
      await client.heartbeat();
    } catch (error) {
      console.error('[WorkspaceClient] Heartbeat failed:', error);
    }
  }, 60_000);  // 1 minute
}

function stopHeartbeat(): void {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval);
    heartbeatInterval = undefined;
  }
}

export function getConnectionId(): string | undefined {
  return connectionId;
}

export function disposeSharedClient(): void {
  stopHeartbeat();

  if (sharedClient) {
    sharedClient.disconnect();
    sharedClient = undefined;
  }
}
```

---

#### 2.2: Buffer Streamer with Race Condition Fixes

**File: `packages/vscode-extension/src/buffer-streamer.ts`**

```typescript
import * as vscode from 'vscode';
import { WorkspaceClient } from '@paperclip/workspace-client';
import { PreviewUpdate } from '@paperclip/workspace-client/types';
import { getConnectionId } from './workspace-client';

export class BufferStreamer implements vscode.Disposable {
  private debounceTimer?: NodeJS.Timeout;
  private inFlight = false;
  private disposed = false;
  private lastConnectionId?: string;
  private expectedStateVersion?: number;
  private disposables: vscode.Disposable[] = [];
  private cancellationToken?: vscode.CancellationTokenSource;

  constructor(
    private document: vscode.TextDocument,
    private client: WorkspaceClient,
    private clientId: string,
    private onUpdate: (update: PreviewUpdate) => void,
    private onError: (error: Error) => void,
    private debounceMs = 150
  ) {}

  async start(): Promise<void> {
    if (this.disposed) {
      throw new Error('BufferStreamer already disposed');
    }

    this.cancellationToken = new vscode.CancellationTokenSource();

    // Send initial buffer
    await this.sendBuffer();

    // Listen to document changes
    const changeListener = vscode.workspace.onDidChangeTextDocument(event => {
      if (event.document === this.document) {
        this.scheduleUpdate();
      }
    });

    this.disposables.push(changeListener);
  }

  private scheduleUpdate(): void {
    if (this.disposed) return;

    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
    }

    this.debounceTimer = setTimeout(() => {
      this.sendBuffer().catch(error => {
        this.onError(error instanceof Error ? error : new Error(String(error)));
      });
    }, this.debounceMs);
  }

  private async sendBuffer(): Promise<void> {
    if (this.disposed || this.inFlight) return;

    // Check for reconnection
    const currentConnectionId = getConnectionId();
    const isReconnect = this.lastConnectionId &&
                        this.lastConnectionId !== currentConnectionId;

    if (isReconnect) {
      console.log('[BufferStreamer] Server reconnected, forcing full resync');
      this.expectedStateVersion = undefined;  // Force Initialize
    }

    this.lastConnectionId = currentConnectionId;
    this.inFlight = true;

    try {
      const content = this.document.getText();
      const filePath = this.document.uri.fsPath;

      // Validate path within workspace
      const workspaceFolder = vscode.workspace.getWorkspaceFolder(this.document.uri);
      if (!workspaceFolder) {
        throw new Error('Document not in workspace');
      }

      // Call unidirectional StreamBuffer RPC
      const response = await this.client.streamBuffer({
        client_id: this.clientId,
        file_path: filePath,
        content,
        expected_state_version: this.expectedStateVersion,
      }, this.cancellationToken!.token);

      // Process stream of patches
      for await (const update of response) {
        if (this.cancellationToken!.token.isCancellationRequested) {
          break;
        }

        // Track state version
        if (update.state_version) {
          this.expectedStateVersion = update.state_version;
        }

        this.onUpdate(update);
      }
    } catch (error) {
      if (!this.disposed) {
        this.onError(error instanceof Error ? error : new Error(String(error)));
      }
    } finally {
      this.inFlight = false;
    }
  }

  updateDebounce(debounceMs: number): void {
    this.debounceMs = debounceMs;
  }

  dispose(): void {
    this.disposed = true;

    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = undefined;
    }

    this.cancellationToken?.cancel();
    this.cancellationToken?.dispose();
    this.cancellationToken = undefined;

    for (const disposable of this.disposables) {
      disposable.dispose();
    }
    this.disposables = [];
  }
}
```

---

#### 2.3: Preview Manager with Pool

**File: `packages/vscode-extension/src/preview-manager.ts`**

```typescript
import * as vscode from 'vscode';
import { PreviewPanel } from './preview-panel';
import { ServerManager } from './server-manager';
import { getSharedClient } from './workspace-client';

const MAX_ACTIVE_PREVIEWS = 10;

export class PreviewManager implements vscode.Disposable {
  private panels = new Map<string, PreviewPanel | null>();
  private activePreviews: { key: string; panel: PreviewPanel }[] = [];

  constructor(
    private context: vscode.ExtensionContext,
    private serverManager: ServerManager
  ) {}

  async openPreview(document: vscode.TextDocument): Promise<void> {
    const key = document.uri.toString();

    // Check if already creating or exists
    if (this.panels.has(key)) {
      const panel = this.panels.get(key);
      if (panel) {
        panel.reveal();
        this.moveToFront(key);
      }
      return;
    }

    // Reserve slot to prevent double-creation race
    this.panels.set(key, null);

    try {
      // Ensure server running
      await this.serverManager.ensureRunning();

      // Get shared client
      const client = await getSharedClient(this.context.extensionPath);

      // Check if we need to evict LRU preview
      if (this.activePreviews.length >= MAX_ACTIVE_PREVIEWS) {
        const lru = this.activePreviews.shift()!;
        console.log(`[PreviewManager] Evicting LRU preview: ${lru.key}`);
        lru.panel.deactivate();  // Keep streamer, dispose WebView
      }

      // Create preview panel
      const panel = new PreviewPanel(
        this.context,
        document,
        client
      );

      this.panels.set(key, panel);
      this.activePreviews.push({ key, panel });

      panel.onDispose(() => {
        this.panels.delete(key);
        this.activePreviews = this.activePreviews.filter(p => p.key !== key);
      });
    } catch (error) {
      this.panels.delete(key);
      throw error;
    }
  }

  private moveToFront(key: string): void {
    const index = this.activePreviews.findIndex(p => p.key === key);
    if (index !== -1) {
      const [preview] = this.activePreviews.splice(index, 1);
      this.activePreviews.push(preview);
    }
  }

  ensurePreview(document: vscode.TextDocument): void {
    const config = vscode.workspace.getConfiguration('paperclip');
    if (config.get<boolean>('autoOpenPreview', false)) {
      this.openPreview(document).catch(error => {
        vscode.window.showErrorMessage(
          `Failed to open preview: ${error instanceof Error ? error.message : String(error)}`
        );
      });
    }
  }

  updateDebounce(debounceMs: number): void {
    for (const { panel } of this.activePreviews) {
      panel.updateDebounce(debounceMs);
    }
  }

  dispose(): void {
    for (const panel of this.panels.values()) {
      panel?.dispose();
    }
    this.panels.clear();
    this.activePreviews = [];
  }
}
```

---

#### 2.4: Preview Panel with Visibility Replay

**File: `packages/vscode-extension/src/preview-panel.ts`**

```typescript
import * as vscode from 'vscode';
import * as path from 'path';
import * as crypto from 'crypto';
import { WorkspaceClient } from '@paperclip/workspace-client';
import { BufferStreamer } from './buffer-streamer';
import { PreviewUpdate } from '@paperclip/workspace-client/types';

export class PreviewPanel implements vscode.Disposable {
  private panel?: vscode.WebviewPanel;
  private streamer?: BufferStreamer;
  private disposables: vscode.Disposable[] = [];
  private nonce: string;
  private lastUpdate?: PreviewUpdate;
  private clientId: string;
  public readonly documentUri: string;

  constructor(
    private context: vscode.ExtensionContext,
    private document: vscode.TextDocument,
    client: WorkspaceClient
  ) {
    this.documentUri = document.uri.toString();
    this.clientId = `vscode-${this.documentUri}-${Date.now()}`;
    this.nonce = crypto.randomBytes(16).toString('base64');

    this.createWebView();
    this.setupStreaming(client);
  }

  private createWebView(): void {
    this.panel = vscode.window.createWebviewPanel(
      'paperclipPreview',
      `Preview: ${path.basename(this.document.fileName)}`,
      vscode.ViewColumn.Beside,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [
          vscode.Uri.file(path.join(this.context.extensionPath, 'media')),
        ],
      }
    );

    this.panel.webview.html = this.getWebviewContent();

    // Replay last update on visibility change
    this.panel.onDidChangeViewState(e => {
      if (e.webviewPanel.visible && this.lastUpdate) {
        this.panel!.webview.postMessage({
          type: 'preview-update',
          payload: this.lastUpdate,
        });
      }
    }, null, this.disposables);

    this.panel.onDidDispose(() => this.dispose(), null, this.disposables);
  }

  private async setupStreaming(client: WorkspaceClient): Promise<void> {
    try {
      const config = vscode.workspace.getConfiguration('paperclip');
      const debounceMs = config.get<number>('previewDebounce', 150);

      this.streamer = new BufferStreamer(
        this.document,
        client,
        this.clientId,
        (update) => this.handlePreviewUpdate(update),
        (error) => this.handleError(error),
        debounceMs
      );

      await this.streamer.start();
    } catch (error) {
      vscode.window.showErrorMessage(
        `Failed to start preview: ${error instanceof Error ? error.message : String(error)}`
      );
    }
  }

  private handlePreviewUpdate(update: PreviewUpdate): void {
    this.lastUpdate = update;

    if (this.panel?.visible) {
      this.panel.webview.postMessage({
        type: 'preview-update',
        payload: update,
      });
    }
  }

  private handleError(error: Error): void {
    this.panel?.webview.postMessage({
      type: 'preview-update',
      payload: {
        filePath: this.document.uri.fsPath,
        patches: [],
        error: error.message,
        timestamp: Date.now(),
      },
    });
  }

  private getWebviewContent(): string {
    const scriptUri = this.panel!.webview.asWebviewUri(
      vscode.Uri.file(path.join(this.context.extensionPath, 'media', 'preview.js'))
    );

    const styleUri = this.panel!.webview.asWebviewUri(
      vscode.Uri.file(path.join(this.context.extensionPath, 'media', 'preview.css'))
    );

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${this.nonce}'; style-src 'nonce-${this.nonce}'; img-src data: ${this.panel!.webview.cspSource};">
  <link rel="stylesheet" href="${styleUri}" nonce="${this.nonce}">
  <title>Paperclip Preview</title>
</head>
<body>
  <div id="preview-root"></div>
  <div id="error-overlay" class="hidden"></div>
  <script nonce="${this.nonce}" src="${scriptUri}"></script>
</body>
</html>`;
  }

  reveal(): void {
    this.panel?.reveal();
  }

  deactivate(): void {
    // Keep streamer, dispose WebView
    this.panel?.dispose();
    this.panel = undefined;
  }

  reactivate(): void {
    // Recreate WebView
    if (!this.panel) {
      this.createWebView();

      // Replay last update
      if (this.lastUpdate) {
        this.handlePreviewUpdate(this.lastUpdate);
      }
    }
  }

  updateDebounce(debounceMs: number): void {
    this.streamer?.updateDebounce(debounceMs);
  }

  onDispose(callback: () => void): void {
    this.disposables.push({ dispose: callback });
  }

  dispose(): void {
    // Dispose in correct order
    this.streamer?.dispose();

    // Explicitly close server-side state
    if (this.streamer) {
      this.streamer.client.closePreview(this.clientId).catch(err => {
        console.error('Failed to close preview:', err);
      });
    }

    this.panel?.dispose();

    for (const disposable of this.disposables) {
      disposable.dispose();
    }
    this.disposables = [];
  }
}
```

---

### Phase 3: WebView Rendering with Transactions (Week 2)

**File: `packages/vscode-extension/media/preview.js`**

```typescript
// Transactional VDOM renderer with semantic ID support

let currentVDOM = null;
let stagedVDOM = null;
let pendingPatches = [];
let rafScheduled = false;
let currentStateVersion = 0;

window.addEventListener('message', (event) => {
  const message = event.data;

  if (message.type === 'preview-update') {
    handlePreviewUpdate(message.payload);
  }
});

function handlePreviewUpdate(update) {
  if (update.error) {
    showError(update.error);
    return;
  }

  hideError();

  // Check version sync
  if (update.state_version && currentStateVersion > 0) {
    if (update.state_version !== currentStateVersion + 1) {
      console.warn('State version mismatch, requesting full sync');
      // TODO: Request full resync
    }
  }

  currentStateVersion = update.state_version || 0;

  // Queue patches for transactional application
  pendingPatches.push(...update.patches);

  if (!rafScheduled) {
    rafScheduled = true;
    requestAnimationFrame(applyPendingPatches);
  }
}

function applyPendingPatches() {
  rafScheduled = false;

  if (pendingPatches.length === 0) return;

  // Start transaction: clone current VDOM
  stagedVDOM = currentVDOM ? deepClone(currentVDOM) : null;

  try {
    // Apply all patches to staged copy
    for (const patch of pendingPatches) {
      stagedVDOM = applyPatch(stagedVDOM, patch);
    }

    // Commit transaction
    currentVDOM = stagedVDOM;
    pendingPatches = [];

    // Render to DOM
    render(currentVDOM, document.getElementById('preview-root'));
  } catch (error) {
    // Rollback: keep currentVDOM unchanged
    console.error('Failed to apply patches:', error);
    showError('Update failed: ' + error.message);

    // Patches remain in queue, will retry
    // Or request full resync if persistent
  }
}

function applyPatch(vdom, patch) {
  switch (patch.type) {
    case 'Initialize':
      return patch.vdom;

    case 'SetAttribute':
      const node = navigateToNode(vdom, patch.path);
      if (node) {
        node.attributes = node.attributes || {};
        node.attributes[patch.key] = patch.value;
      }
      return vdom;

    case 'SetText':
      const textNode = navigateToNode(vdom, patch.path);
      if (textNode && textNode.type === 'Text') {
        textNode.content = patch.value;
      }
      return vdom;

    case 'InsertChild':
      const parent = navigateToNode(vdom, patch.parent);
      if (parent) {
        parent.children = parent.children || [];
        parent.children.splice(patch.index, 0, patch.child);
      }
      return vdom;

    case 'RemoveChild':
      const parentNode = navigateToNode(vdom, patch.parent);
      if (parentNode && parentNode.children) {
        const childIndex = parentNode.children.findIndex(
          c => c.semanticId === patch.child_semantic_id
        );
        if (childIndex !== -1) {
          parentNode.children.splice(childIndex, 1);
        }
      }
      return vdom;

    case 'MoveChild':
      const moveParent = navigateToNode(vdom, patch.parent);
      if (moveParent && moveParent.children) {
        const childIndex = moveParent.children.findIndex(
          c => c.semanticId === patch.child_semantic_id
        );
        if (childIndex !== -1) {
          const [child] = moveParent.children.splice(childIndex, 1);
          moveParent.children.splice(patch.to_index, 0, child);
        }
      }
      return vdom;

    default:
      console.warn('Unknown patch type:', patch.type);
      return vdom;
  }
}

function navigateToNode(vdom, patchPath) {
  if (!patchPath) return null;

  // Handle PatchPath enum
  if (patchPath.positional) {
    return navigateByPosition(vdom, patchPath.positional);
  } else if (patchPath.semantic) {
    return navigateBySemanticId(vdom, patchPath.semantic);
  }

  // Legacy: direct string path (positional)
  return navigateByPosition(vdom, patchPath);
}

function navigateByPosition(vdom, path) {
  if (!path || path === 'root') return vdom;

  const parts = path.split('/').slice(1);
  let current = vdom;

  for (const part of parts) {
    const index = parseInt(part, 10);
    if (!current.children || !current.children[index]) {
      return null;
    }
    current = current.children[index];
  }

  return current;
}

function navigateBySemanticId(vdom, semanticId) {
  if (!vdom) return null;

  // Build semantic ID map
  const semanticMap = new Map();

  function visit(node) {
    if (node.semanticId) {
      semanticMap.set(node.semanticId, node);
    }
    if (node.children) {
      node.children.forEach(visit);
    }
  }

  visit(vdom);

  return semanticMap.get(semanticId) || null;
}

function deepClone(obj) {
  if (obj === null || typeof obj !== 'object') return obj;

  if (Array.isArray(obj)) {
    return obj.map(deepClone);
  }

  const cloned = {};
  for (const key in obj) {
    cloned[key] = deepClone(obj[key]);
  }
  return cloned;
}

function render(vdom, container) {
  if (!vdom) return;

  // Simple full re-render for MVP
  container.innerHTML = '';
  const element = createDOMElement(vdom);
  container.appendChild(element);
}

function createDOMElement(vnode) {
  if (vnode.type === 'Text') {
    return document.createTextNode(vnode.content || '');
  }

  const element = document.createElement(vnode.tag || 'div');

  if (vnode.attributes) {
    for (const [key, value] of Object.entries(vnode.attributes)) {
      element.setAttribute(key, value);
    }
  }

  if (vnode.styles) {
    for (const [key, value] of Object.entries(vnode.styles)) {
      element.style[key] = value;
    }
  }

  if (vnode.children) {
    for (const child of vnode.children) {
      element.appendChild(createDOMElement(child));
    }
  }

  return element;
}

function showError(error) {
  const overlay = document.getElementById('error-overlay');
  overlay.textContent = escapeHtml(error);
  overlay.classList.remove('hidden');
}

function hideError() {
  const overlay = document.getElementById('error-overlay');
  overlay.classList.add('hidden');
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// Development mode: freeze VDOM to catch mutations
if (location.hostname === 'localhost') {
  const originalApplyPatch = applyPatch;
  applyPatch = function(vdom, patch) {
    if (vdom) {
      try {
        Object.freeze(vdom);
        if (vdom.children) {
          vdom.children.forEach(c => Object.freeze(c));
        }
      } catch (e) {
        // Already frozen or can't freeze
      }
    }
    return originalApplyPatch(vdom, patch);
  };
}
```

---

## Acceptance Criteria

### Functional Requirements (MVP)

- [ ] ✅ VSCode extension activates on `.pc` file open
- [ ] ✅ Syntax highlighting for Paperclip syntax
- [ ] ✅ WebView preview panel displays component output
- [ ] ✅ Keystroke-level updates with <300ms latency
- [ ] ✅ Parse/eval errors displayed in overlay
- [ ] ✅ Single shared gRPC client for all previews
- [ ] ✅ Automatic server startup if not running
- [ ] ✅ Graceful reconnection with exponential backoff + jitter
- [ ] ✅ Preview pool with max 10 active WebViews
- [ ] ✅ Explicit state cleanup via ClosePreview RPC

### Security & Reliability

- [ ] ✅ TLS credentials for production gRPC
- [ ] ✅ Strict WebView CSP with nonces
- [ ] ✅ Enhanced path validation (symlinks, Unicode, canonicalization)
- [ ] ✅ Parse/eval timeouts (5 seconds)
- [ ] ✅ Depth limits (50 levels) and node limits (10,000 nodes)
- [ ] ✅ Per-process rate limiting (100 requests/minute)
- [ ] ✅ Total VDOM memory cap (500MB)
- [ ] ✅ Client state cap (100) with LRU eviction
- [ ] ✅ Heartbeat + periodic cleanup (5-minute timeout)
- [ ] ✅ Transactional patch application with rollback
- [ ] ✅ Version-based state synchronization

### Performance

- [ ] ✅ Total latency < 300ms (realistic)
- [ ] ✅ No memory leaks after 1 hour
- [ ] ✅ Handles 100+ concurrent client connections
- [ ] ✅ Graceful degradation under load

---

## Implementation Timeline

| Phase | Duration | Deliverables |
|-------|----------|-------------|
| 1. Foundation & Hardening | Week 1 | Proto with ClosePreview/Heartbeat, server with all limits, enhanced path validation, rate limiting |
| 2. Client Reliability | Week 1-2 | Shared client, preview pool, heartbeat, transactional patches, reconnection logic |
| 3. WebView & Rendering | Week 2 | Strict CSP, transactional rendering, semantic ID support, visibility replay |
| 4. Testing & Validation | Week 2-3 | Unit tests, integration tests, security audit, pressure tests |

**MVP Total**: 2-3 weeks
**Post-MVP**: Semantic ID rollout (Phase 2), HTTP server + ngrok (Phase 4)

---

## Success Metrics

- **Latency**: Edit → Preview < 300ms (realistic)
- **Reliability**: <0.1% error rate under load
- **Security**: Zero path traversal or DoS vulnerabilities
- **Performance**: No perceptible lag with 10 active previews
- **Memory**: Stable memory usage over 8-hour sessions

---

## Appendix A: Threat Model Summary

| Threat | Severity | Mitigation |
|--------|----------|------------|
| State pollution DoS | 🔴 High | Total memory cap (500MB), per-process rate limit (100/min) |
| Symlink path traversal | 🔴 High | Symlink detection, canonical validation, workspace boundary check |
| Client ID squatting | 🟡 Medium | Capability-based auth with secrets (post-MVP) |
| Parser/evaluator DoS | 🟡 Medium | Timeouts (5s), depth limits (50), node limits (10k) |
| Unbounded memory growth | 🔴 High | Client count cap (100), LRU eviction, heartbeat cleanup (5min) |

---

## Appendix B: Pressure Test Summary

| Scenario | Breaking Point | Mitigation |
|----------|---------------|------------|
| 50+ previews | 20 WebViews = sluggish | Preview pool (max 10 active), lazy creation, LRU eviction |
| Rapid open/close | 1000 iterations = memory leak | Heartbeat (5min), periodic cleanup, explicit ClosePreview |
| Network flake | Partial patches | Transactional application, version-based sync, rollback |
| File system race | Buffer vs disk conflicts | Buffer always wins, sequence numbers |
| Server restart | Thundering herd, lost state | Exponential backoff + jitter, force Initialize on reconnect |

---

## Appendix C: Semantic ID Protocol (Post-MVP)

### Migration Path

**Phase 1 (MVP)**: Positional paths only
- Simple `"root/0/1/2"` string paths
- Works for MVP, defers complexity

**Phase 2 (Post-MVP)**: Dual-mode support
- Server emits both `patches_positional` and `patches_semantic`
- Client opts in via feature flag
- Measure performance improvement

**Phase 3 (Future)**: Semantic only
- Remove positional patches
- Full semantic ID system
- Enables undo/time-travel features

### Semantic ID Format

```
Card{Card-0}::div[d4f5]::Button{Button-1}::button[a8b3]
```

**Components:**
- `Card{Card-0}` - Component instance with key
- `div[d4f5]` - Element with CRC32 ID
- Path represents full hierarchy from root to node

### Benefits

- **Stable identity across refactoring**: Moving elements doesn't change their semantic ID
- **Efficient reordering**: `MoveChild` patch instead of full re-render
- **Better diffing**: Only changed nodes get patches, not their siblings
- **Foundation for undo/redo**: Identity-based history tracking

### Performance Gain

**Example: Reordering 3 items in list of 1000**

Positional:
- Patches: 1000 SetText (everything after reorder point)
- DOM ops: 1000 textContent updates

Semantic:
- Patches: 3 MoveChild
- DOM ops: 3 DOM node moves

**Result: 300x fewer DOM operations**

---

## References

### Internal
- **Workspace Client**: `packages/workspace-client/src/client.ts`
- **Hot Reload Spike**: `docs/spikes/SPIKE_0.2_HOT_RELOAD.md`
- **Identity System**: `docs/IDENTITY_SYSTEM.md`
- **Semantic Identity**: Already implemented in `packages/evaluator/src/identity.rs`

### External
- VSCode Extension API: https://code.visualstudio.com/api
- VSCode WebView Security: https://code.visualstudio.com/api/extension-guides/webview#security
- gRPC Node.js: https://grpc.io/docs/languages/node/
- gRPC Best Practices: https://grpc.io/docs/guides/performance/

---

**End of Comprehensive Plan**

This plan now includes:
- ✅ All threat mitigations from security analysis
- ✅ All reliability fixes from pressure testing
- ✅ Semantic ID protocol design for future-proofing
- ✅ Complete implementation details with code
- ✅ Realistic resource limits and configuration
- ✅ Clear migration path for gradual rollout
