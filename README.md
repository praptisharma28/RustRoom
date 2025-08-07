# RustRoom
A chat + livestreaming app

User Browser ↔ WebSocket ↔ Rust Server ↔ WebSocket ↔ Other Users
     ↓                                           ↑
  WebRTC Video Stream ←→←→←→←→←→←→←→←→←→←→ WebRTC Video Stream