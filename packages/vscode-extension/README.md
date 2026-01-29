# Paperclip VSCode Extension

Live preview for Paperclip (.pc) files with production-grade streaming.

## Features

- **Live Preview**: Real-time updates as you type in .pc files
- **Production-Hardened**: Rate limiting, memory caps, and security hardening
- **Efficient Streaming**: gRPC-based unidirectional streaming with debouncing
- **Multiple Previews**: Support for concurrent preview panels with LRU eviction

## Installation

### Prerequisites

1. **Paperclip Server**: The extension requires the `paperclip-server` binary to be running
2. **Rust Toolchain**: Required to build the server (if not already built)

### Building the Server

```bash
cd packages/workspace
cargo build --release
```

The server binary will be at `target/release/paperclip-server`.

### Installing the Extension

1. Open this directory in VSCode
2. Press F5 to launch the Extension Development Host
3. In the new window, open a `.pc` file
4. Click the preview icon in the editor toolbar

## Configuration

- `paperclip.serverPath`: Path to paperclip-server binary (auto-detected if empty)
- `paperclip.serverPort`: gRPC server port (default: 50051)
- `paperclip.maxPreviewPanels`: Maximum concurrent preview panels (default: 10)
- `paperclip.previewDebounceMs`: Debounce delay for updates in ms (default: 100)

## Usage

1. **Start the Server**:
   ```bash
   cargo run --bin paperclip-server
   ```

2. **Open a .pc file** in VSCode

3. **Open Preview**: Click the preview icon in the editor toolbar, or run command:
   ```
   Paperclip: Open Preview
   ```

4. **Edit the file**: Changes appear in the preview in real-time

## Architecture

### Components

- **WorkspaceClient**: Shared gRPC client with automatic reconnection
- **BufferStreamer**: Per-document streaming with race condition handling
- **PreviewManager**: Preview pool with LRU eviction
- **PreviewPanel**: WebView with strict CSP and visibility replay

### Security Features

- **Content Security Policy**: Strict CSP with nonces
- **Path Validation**: Unicode normalization and symlink detection
- **Rate Limiting**: Process-level rate limiting (100 req/min)
- **Memory Caps**: Total VDOM memory limited to 500MB
- **Timeouts**: Parse and eval operations timeout after 5 seconds

### Reliability Features

- **Automatic Reconnection**: Exponential backoff with jitter
- **Heartbeat Tracking**: 5-minute client timeout with automatic cleanup
- **Debouncing**: Configurable debounce for rapid typing
- **Transactional Patches**: Rollback on patch application errors
- **Visibility Replay**: Queued updates replayed when panel becomes visible

## Development

### Build

```bash
npm install
npm run compile
```

### Watch Mode

```bash
npm run watch
```

### Testing

Press F5 in VSCode to launch the Extension Development Host.

## License

See repository LICENSE file.
