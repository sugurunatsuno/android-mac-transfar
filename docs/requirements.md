# Requirements Specification

## 1. Purpose & Background
- **Problem**: File distribution from Mac to multiple devices should be simple without relying on USB or cloud services.
- **Solution**: Run a local file hosting server on macOS using Tauri. Clients can download designated files over HTTP.
- **Goal**: Allow cable‑free offline delivery of multi‑GB files from Mac to other devices.

## 2. Terminology
| Term | Definition |
|------|-----------|
| **Host device** | Mac running the Tauri server. |
| **Client** | Browser or CLI downloading files. |
| **Session** | Period while the host server is active and serving downloads. |

## 3. Scope
### 3.1 In Scope
- File distribution via HTTP/HTTPS GET requests.
- Mac UI shows list of shared files and download progress.
- Clients access download URLs through browser or tools.
- Network auto-discovery using QR code or mDNS (`_filedrop._tcp.local`).
- Support at least three parallel downloads.
- Optional SHA‑256 checksum for each file.

### 3.2 Out of Scope
- Automatic two-way sync.
- Internet-facing transfers.
- Real-time folder mirroring.

## 4. Constraints
| Category | Details |
|----------|---------|
| **Tech stack** | Tauri (Rust, wry) targeting macOS (x64/arm64). |
| **Network** | Local Wi‑Fi or wired LAN, IPv4 required. |
| **Storage** | Files served from user-chosen directory. |
| **Security** | Local network access only by default; optional password/token. |
| **License** | Planned MIT license. |

## 5. User Stories
| ID | User | Goal | Condition | Acceptance |
|----|------|------|-----------|------------|
| **US1** | Client user | Download 500 MB file from Mac | Same LAN | Access URL and file downloads successfully with correct size. |
| **US2** | Multiple clients | Get documents at the same time | 3 clients in parallel | All receive complete files without corruption. |
| **US3** | Mac user | Confirm download history | After download | App shows file name, size, and time for each download. |

## 6. Functional Requirements
| FR-ID | Requirement | Priority |
|-------|-------------|---------|
| **FR-01** | Start HTTP server and display port/URL in UI. | ★★★ |
| **FR-02** | Serve files via `/download/<name>` endpoint. | ★★★ |
| **FR-03** | Provide progress updates via SSE or WebSocket. | ★★☆ |
| **FR-04** | Offer mDNS and QR code for connection info. | ★★☆ |
| **FR-05** | Let user choose directory and port in settings. | ★★☆ |
| **FR-06** | Optional basic authentication. | ★☆☆ |

## 7. Non‑Functional Requirements
| NFR-ID | Requirement | Metric |
|--------|------------|-------|
| **NFR-01** | Performance | 1 GB download at ≥150 Mbps (Wi‑Fi 6). |
| **NFR-02** | Resource usage | <1 CPU core, <150 MB RAM while idle. |
| **NFR-03** | Reliability | 24‑hour uptime without crashes. |
| **NFR-04** | Startup time | ≤3 s from launch to server ready. |
| **NFR-05** | Usability | Start server and show URL/QR within 3 clicks. |

## 8. UI Overview
- **Home screen**: Start/stop server button, displays URL and QR code when active, download log list.
- **Settings screen**: Directory picker for shared files, port input, authentication settings, auto-start option.

## 9. API Interface (tentative)
| Method | Path | Description | Auth |
|--------|------|-------------|------|
| **GET** | `/download/<name>` | Retrieve the specified file. | Optional basic auth |
| **GET** | `/healthz` | Liveness probe. | None |
| **GET** | `/events` | SSE progress updates. | Optional basic auth |

## 10. Acceptance Tests
- **Single file**: Download 1 GB file and verify hash.
- **Parallel downloads**: 3 clients fetch files simultaneously; all succeed.
- **Authentication**: Unauthorized download is denied when enabled; authorized request succeeds.
- **Network interruption**: Disconnect Wi‑Fi during download, reconnect, and ensure error handling.
- **Directory change**: After changing settings, new directory contents are served.

## 11. Roadmap
| Phase | Goal | Duration |
|-------|------|---------|
| **POC** | Serve one file using Rust + Hyper. | 1 week |
| **Alpha** | Integrate Tauri UI for single download. | +1 week |
| **Beta** | Parallel downloads, QR/mDNS, settings screen. | +2 weeks |
| **Release** | Authentication and checksum verification. | +2 weeks |
