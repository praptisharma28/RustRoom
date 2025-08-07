# 🦀 RustRoom - Classroom Live Streaming App

RustRoom is a real-time classroom streaming application built with Rust and WebRTC. It allows students to join virtual classrooms, share live video streams, and chat in real-time.

---

## Features

* Room-based system: Join classrooms by name (e.g., `AIML3B`, `CSE2A`)
* Live video streaming via WebRTC (peer-to-peer)
* Real-time chat with room participants
* Online user tracking and live stream indicators
* Minimal UI with black-and-white glassmorphism design
* High performance using Rust and async concurrency

---

## Getting Started

### Run Locally

1. Clone and build:

```bash
git clone <your-repo-url>
cd rustroom
cargo run
```

2. Open your browser at: [http://localhost:3030](http://localhost:3030)

3. Join a classroom by name and test chat/stream in multiple tabs.

---

## Technical Stack

### Backend (Rust)

* **WebSocket server**: Handles real-time communication.
* **Room Manager**: Dynamically manages user rooms and sessions.
* **Tokio Runtime**: For concurrent, async networking.

### Frontend (HTML/JS)

* **WebRTC**: For video/audio peer-to-peer streaming.
* **WebSocket client**: Connects to Rust backend.
* **Media API**: Accesses camera and mic.

---

## Usage

### Join a Room

* Type a classroom name (e.g., `AIML3B`) and join.
* You'll see a chat box and live video panel.

### Start Streaming

* Click "Start Stream" and allow camera/mic access.
* Other users in the room will see your live stream.

### Chat

* Send messages to all users in the same room.
* Get real-time join/leave notifications.

---

## Deployment Options

### Localhost

* Run: `cargo run`
* Access via `http://localhost:3030`

### Cloud (Docker + Cloud provider)

1. Build Docker image:

```bash
docker build -t rustroom .
```

2. Run with Docker:

```bash
docker run -p 3030:3030 rustroom
```

3. Use reverse proxy for HTTPS in production.

## License

Licensed under the [MIT License](./LICENSE).
Feel free to use and modify for learning or internal purposes.
