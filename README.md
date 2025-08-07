# RustRoom
A chat + livestreaming app

Core Features & Modules
👥 User Logic:
Join a room (class name).

If first to join = become broadcaster.

Else = become viewer (consume stream).

📹 Webcam Video Streaming:
Capture video from broadcaster.

Encode (probably using WebRTC, GStreamer, or ffmpeg bindings).

Transmit to server.

📡 Server Side (Rust):
Accept incoming video stream from broadcaster.

Broadcast to all viewers in the same room.

Manage connected clients per room.

🧱 Frontend:
Web interface for users to:

Enter room name.

Watch stream.

If broadcaster: allow webcam feed input.

🔄 Tech Stack
Layer	Tool/Library	Purpose
Frontend	HTML + JS + CSS	Simple UI, room selection
Communication	WebSocket (via tokio-tungstenite or warp)	Real-time bi-directional communication
Video Processing	GStreamer or FFmpeg bindings	Encode/decode video
Server	Rust + Tokio + Warp or Actix-web	Handle connections and streams
State Management	DashMap or Mutex/Arc	Room/user state tracking
