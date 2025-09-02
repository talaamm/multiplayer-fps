### Client networking

Key file: `client/src/network.rs`

- Socket
  - Binds ephemeral UDP (`0.0.0.0:0`), connects to server, sets nonblocking.
- Threads and channels
  - Spawns a dedicated networking thread.
  - Exposes `tx_outgoing: Sender<ClientToServer>` and `rx_incoming: Receiver<ServerToClient>` to game code.
- Startup
  - Immediately sends `Join{username}` to get `Accept{player_id, level}`.
- Outgoing loop
  - Rate limits to ~66Hz (`min_send_dt`); encodes via `protocol::encode_client` and sends.
- Incoming loop
  - `recv` into buffer; decodes via `protocol::decode_server`; pushes messages to `rx_incoming`.
- Responsibilities
  - Send `Input` updates with `(player_id, x, y, angle, action)`.
  - Handle `Accept` (initialize level), `Snapshot` (update render state), `Hit`/`Death` events (FX/UI), `Pong` (latency), `PlayerLeft` (cleanup), `Error`.

### Why this design

- Threaded client keeps networking independent of render loop; channels decouple systems.
- Nonblocking UDP ensures low-latency input and timely snapshots.
- Protocol crate guarantees message compatibility with server. 