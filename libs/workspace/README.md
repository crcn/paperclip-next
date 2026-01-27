# paperclip-workspace

gRPC workspace server with real-time file watching and preview streaming.

## Features

- 🌐 **gRPC streaming** - Real-time preview updates using Tonic
- 👁️ **File watching** - Automatic recompilation on .pc file changes
- ⚡ **Fast pipeline** - Parse → Evaluate → Stream in microseconds
- 📡 **Streaming RPC** - Efficient binary protocol
- ✅ **Tested** - 1 passing test

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
paperclip-workspace = { path = "../workspace" }
```

## Usage

### Running the Server

#### As Binary

```bash
# Start server in current directory
cargo run --bin paperclip-server

# Start server with specific directory
cargo run --bin paperclip-server /path/to/project

# Server listens on 127.0.0.1:50051
```

#### As Library

```rust
use paperclip_workspace::WorkspaceServer;
use std::path::PathBuf;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_dir = PathBuf::from("./examples");
    let workspace = WorkspaceServer::new(root_dir);

    let addr = "127.0.0.1:50051".parse()?;

    Server::builder()
        .add_service(workspace.into_service())
        .serve(addr)
        .await?;

    Ok(())
}
```

### File Watching

```rust
use paperclip_workspace::FileWatcher;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from("./examples");
    let watcher = FileWatcher::new(path).expect("Failed to create watcher");

    println!("Watching for file changes...");

    loop {
        if let Some(event) = watcher.next_event() {
            println!("File changed: {:?}", event.paths);

            // Process the file
            for path in event.paths {
                println!("Changed: {}", path.display());
            }
        }
    }
}
```

## gRPC API

### Service: `WorkspaceService`

#### `StreamPreview`

Stream preview updates for a .pc file.

**Request:**
```protobuf
message PreviewRequest {
  string file_path = 1;
}
```

**Response Stream:**
```protobuf
message PreviewUpdate {
  string file_path = 1;
  string vdom_json = 2;
  optional string error = 3;
  int64 timestamp = 4;
}
```

**Usage:**
```rust
use paperclip_workspace::proto::{
    workspace_service_client::WorkspaceServiceClient,
    PreviewRequest,
};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = WorkspaceServiceClient::connect("http://127.0.0.1:50051").await?;

    let request = Request::new(PreviewRequest {
        file_path: "button.pc".to_string(),
    });

    let mut stream = client.stream_preview(request).await?.into_inner();

    while let Some(update) = stream.message().await? {
        if let Some(error) = update.error {
            eprintln!("Error: {}", error);
        } else {
            println!("Update at {}: {}", update.timestamp, update.vdom_json);
        }
    }

    Ok(())
}
```

#### `WatchFiles`

Watch multiple files for changes.

**Request:**
```protobuf
message WatchRequest {
  string directory = 1;
  repeated string patterns = 2;
}
```

**Response Stream:**
```protobuf
message FileEvent {
  enum EventType {
    CREATED = 0;
    MODIFIED = 1;
    DELETED = 2;
  }

  EventType event_type = 1;
  string file_path = 2;
  int64 timestamp = 3;
}
```

## Protocol Buffer Schema

Located at `../../proto/workspace.proto`:

```protobuf
syntax = "proto3";

package paperclip.workspace;

service WorkspaceService {
  rpc StreamPreview(PreviewRequest) returns (stream PreviewUpdate);
  rpc WatchFiles(WatchRequest) returns (stream FileEvent);
}

message PreviewRequest {
  string file_path = 1;
}

message PreviewUpdate {
  string file_path = 1;
  string vdom_json = 2;
  optional string error = 3;
  int64 timestamp = 4;
}

message WatchRequest {
  string directory = 1;
  repeated string patterns = 2;
}

message FileEvent {
  enum EventType {
    CREATED = 0;
    MODIFIED = 1;
    DELETED = 2;
  }

  EventType event_type = 1;
  string file_path = 2;
  int64 timestamp = 3;
}
```

## Example Workflow

### 1. Start Server

```bash
cargo run --bin paperclip-server examples
```

Server starts and watches the `examples/` directory.

### 2. Create/Edit .pc File

```javascript
// examples/button.pc
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366FF
        }
        text "Click me"
    }
}
```

### 3. Server Automatically Detects Change

Server receives file system notification, parses the file, evaluates to Virtual DOM, and streams the update to all connected clients.

### 4. Client Receives Update

```json
{
  "file_path": "button.pc",
  "vdom_json": "{\"nodes\":[{\"type\":\"Element\",...}],\"styles\":[]}",
  "error": null,
  "timestamp": 1706371200000
}
```

## File Watcher API

### `FileWatcher`

File system watcher using the `notify` crate.

#### Methods

**`new(path: PathBuf) -> WatcherResult<Self>`**

Create a new file watcher for the given path (recursive).

**`next_event(&self) -> Option<Event>`**

Wait for the next file system event (blocking).

**`try_next_event(&self) -> Option<Event>`**

Try to get the next event without blocking.

### Example

```rust
use paperclip_workspace::FileWatcher;
use std::path::PathBuf;

fn main() {
    let watcher = FileWatcher::new(PathBuf::from("."))
        .expect("Failed to create watcher");

    loop {
        if let Some(event) = watcher.next_event() {
            match event.kind {
                notify::EventKind::Create(_) => {
                    println!("File created: {:?}", event.paths);
                }
                notify::EventKind::Modify(_) => {
                    println!("File modified: {:?}", event.paths);
                }
                notify::EventKind::Remove(_) => {
                    println!("File removed: {:?}", event.paths);
                }
                _ => {}
            }
        }
    }
}
```

## Testing

Run tests:

```bash
cargo test -p paperclip-workspace
```

Test with real files:

```bash
# Terminal 1: Start server
cargo run --bin paperclip-server examples

# Terminal 2: Edit a file
echo 'public component Test { render div { text "Hello" } }' > examples/test.pc

# Server should log the change and stream update
```

## Error Handling

The server handles errors gracefully:

```rust
// If parse fails
PreviewUpdate {
    file_path: "button.pc",
    vdom_json: "",
    error: Some("Parse error: UnexpectedToken at position 42"),
    timestamp: 1706371200000,
}

// If evaluate fails
PreviewUpdate {
    file_path: "button.pc",
    vdom_json: "",
    error: Some("Evaluation error: Component 'Foo' not found"),
    timestamp: 1706371200000,
}
```

## Configuration

Server configuration:

```rust
// Default address
let addr = "127.0.0.1:50051".parse()?;

// Custom address
let addr = "0.0.0.0:8080".parse()?;

// With TLS (requires tonic-tls feature)
use tonic::transport::ServerTlsConfig;

let tls_config = ServerTlsConfig::new()
    .cert_path("cert.pem")
    .key_path("key.pem");

Server::builder()
    .tls_config(tls_config)?
    .add_service(workspace.into_service())
    .serve(addr)
    .await?;
```

## Performance

- **File watch latency:** ~1-10ms (OS dependent)
- **Parse + Evaluate:** ~5 microseconds
- **Stream latency:** ~1-5ms (network dependent)
- **Total latency:** ~10-20ms (edit → preview update)

## Dependencies

- `tonic` - gRPC framework
- `tokio` - Async runtime
- `notify` - File system watching
- `paperclip-parser` - .pc parsing
- `paperclip-evaluator` - AST evaluation
- `serde_json` - JSON serialization

## Development

Build proto files:

```bash
cargo build -p paperclip-workspace
# build.rs compiles proto files automatically
```

Run with logging:

```bash
RUST_LOG=debug cargo run --bin paperclip-server examples
```

## License

MIT
