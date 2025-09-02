### Testing & Running

#### Run the server

```bash
cd server
cargo run --release
```

- Env: set `SERVER_BIND` to override bind address (default `0.0.0.0:34254`).
- Single-player debug (ASCII):
```bash
cargo run --release -- --single-player
```

#### Run the client

```bash
cd client
cargo run --release
```

- The client connects to the configured server address inside your app UI/args; update as needed.

#### Protocol sanity checks

- Use `netcat`/`socat` or a small script to send a `Join` JSON to the server UDP port and inspect the `Accept` response.

#### Integration test note

- `tests/integration_udp.rs` appears to be an older example referencing `multiplayer_fps::net::{ClientMsg, ServerMsg}` which are not present now. Treat it as legacy; do not run as-is. Prefer end-to-end manual run or add new tests against `protocol` types.

#### Tips

- Check `debugs/server.txt` for logs if you add logging.
- If snapshots seem choppy, confirm 20Hz broadcast task is running and client is decoding messages. 