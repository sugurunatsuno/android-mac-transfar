# Requirements Specification

## 1. Purpose & Background
- **Problem**: USB transfer between Mac and Android is unstable or impossible.
- **Solution**: Run a local file receiving server on the Android device, allowing uploads from other devices via HTTP.
- **Goal**: Enable stress-free exchange of multi-GB files offline without cables or external cloud services.

## 2. Terminology
| Term | Definition |
|------|-----------|
| **Host device** | Android device running the server. |
| **Client** | Browser or CLI uploading files. |
| **Session** | Period while the host server is active and accepting uploads. |

## 3. Scope
### 3.1 In Scope
- File reception via HTTP/HTTPS POST (single or multipart uploads).
- Progress display: upload list with progress bar on Android, standard browser UI on client.
- Save completed transfers to device storage (`/storage/emulated/0/Download/<AppName>/` or app-specific directory).
- Network auto-discovery using QR code or mDNS (`_filedrop._tcp.local`).
- Support at least three parallel uploads.
- Optional SHA‑256 checksum verification.

### 3.2 Out of Scope
- Automatic bidirectional sync (real-time mirroring).
- Transfers over the internet.
- Folder-level sync (current phase handles single files only).

## 4. Constraints
| Category | Details |
|----------|---------|
| **Tech stack** | Tauri (Rust, wry) targeting Android `arm64-v8a`. |
| **Network** | Local Wi-Fi or USB tethering, IPv4 required. |
| **Storage** | Avoid `MANAGE_EXTERNAL_STORAGE`; prefer standard permissions. |
| **Security** | Local network access only by default; optional password/token. |
| **License** | Planned MIT license. |

## 5. User Stories
| ID | User | Goal | Condition | Acceptance |
|----|------|------|-----------|------------|
| **US1** | Mac user | Send 500 MB photo to Android | Same LAN | Access URL in browser → choose file → 100% completion → file created on Android. |
| **US2** | Multiple clients | Send documents concurrently | 3 clients in parallel | Each progress is independent and all succeed with correct integrity. |
| **US3** | Android user | Check transfer result | After transfer | File name, size, and date shown in app; tap opens share menu. |

## 6. Functional Requirements
| FR-ID | Requirement | Priority |
|-------|------------|---------|
| **FR-01** | Start HTTP server on host startup and display port number in UI. | ★★★ |
| **FR-02** | Allow files up to 4 GiB at `/upload` endpoint. | ★★★ |
| **FR-03** | Save multipart `file` field and auto-rename on name collision. | ★★☆ |
| **FR-04** | Notify front‑end of progress via SSE or WebSocket. | ★★☆ |
| **FR-05** | Compute SHA‑256 after transfer and match with client value. | ★★☆ |
| **FR-06** | Share connection URL via mDNS and QR code. | ★★☆ |
| **FR-07** | Allow changing save directory and port in settings. | ★★☆ |
| **FR-08** | Optional basic auth (user/password). | ★☆☆ |

## 7. Non‑Functional Requirements
| NFR-ID | Requirement | Metric |
|--------|------------|-------|
| **NFR-01** | Performance | 1 GB file transfers at ≥150 Mbps (Wi‑Fi 6). |
| **NFR-02** | Resource usage | <1 CPU core, <150 MB RAM while idle. |
| **NFR-03** | Reliability | 24‑hour sessions with zero crashes and no file loss. |
| **NFR-04** | Startup time | ≤3 s from app launch to server ready. |
| **NFR-05** | Usability | Start server and show URL/QR within 3 taps. |

## 8. UI Overview
- **Home screen**: button to start server, shows URL and QR code when running, list of transfer history.
- **Settings screen**:
  - Directory picker for save location.
  - Port number input.
  - Authentication toggle and password setting.
  - Option to auto-start server on launch.

## 9. API Interface (tentative)
| Method | Path | Description | Auth |
|--------|------|-------------|------|
| **POST** | `/upload` | `multipart/form-data` with field `file`. | Optional basic auth |
| **GET** | `/healthz` | Liveness probe. | None |
| **GET** | `/events` | SSE progress updates. | Optional basic auth |

## 10. Acceptance Tests
- **Single file**: Upload 1 GB file, verify hash match.
- **Parallel uploads**: 3 clients send files concurrently; all succeed.
- **Authentication**: With auth enabled, unauthorized access returns 401; with correct credentials, transfer succeeds.
- **Network interruption**: During transfer, disconnect Wi‑Fi and reconnect; client receives appropriate error.
- **Save path change**: After changing settings, files appear in new directory.

## 11. Roadmap
| Phase | Goal | Duration |
|-------|------|---------|
| **POC** | Receive and save one file using Rust + Hyper. | 1 week |
| **Alpha** | Integrate Tauri UI for single transfer. | +1 week |
| **Beta** | Parallel transfers, QR/mDNS, settings screen. | +2 weeks |
| **Release** | Authentication, checksum, internal Google Play test. | +2 weeks |

