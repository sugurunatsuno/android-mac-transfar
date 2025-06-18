# Design Overview

This document outlines a high-level design for an Android application that acts as a local file receiver. The app enables Mac or other devices on the same network to upload large files over HTTP.

## Architecture
1. **Frontend (Tauri UI)**
   - Built with Tauri using Rust and the `wry` runtime.
   - Provides a minimal interface for starting the server, showing the URL/QR code, viewing transfer history, and changing settings.
2. **Backend (Rust HTTP Server)**
   - Uses `hyper` to handle HTTP requests.
   - Exposes `/upload`, `/healthz`, and `/events` endpoints.
   - `/events` streams progress updates using Server-Sent Events (SSE).
   - Manages file writes to the configured storage path.
3. **Communication**
   - Upload progress is streamed to the UI via SSE or WebSocket.
   - When the server starts, a QR code and mDNS advertisement share the connection URL.
4. **Storage**
   - Files are saved under `/storage/emulated/0/Download/<AppName>/` by default or a user-chosen directory.
   - Name collisions are resolved by appending a numeric suffix.
5. **Security**
   - By default, the server is only reachable on the local network.
   - Optional basic authentication can be enabled in settings.
6. **Concurrency**
   - The server accepts at least three concurrent upload connections.
   - Each upload is processed independently, and the UI displays individual progress bars.
7. **Checksum Verification**
   - After a transfer completes, the server calculates a SHA‑256 hash and compares it with a value supplied by the client (if provided).

## Data Flow
1. User starts the server from the Android device.
2. The app binds to a local port and advertises its URL via mDNS and QR code.
3. A client accesses the URL in a browser and uploads files using a standard form.
4. The server streams progress updates back to the client UI.
5. When the upload finishes, the server saves the file to storage and performs checksum verification if enabled.
6. The result appears in the Android UI's transfer history, where the user can open or share the file.

## Future Extensions
- Implement HTTPS support with self-signed certificates.
- Provide a CLI client for scripted transfers.
- Optionally integrate with Android's sharing intents for easier outbound transfers.

