# ⚡ Team Roles & Responsibilities

### 👩‍💻 Tala Amm – **Server & Networking Lead**

* Implement the **UDP-based server**.
* Handle:

  * Accepting multiple client connections.
  * Broadcasting player positions to all clients.
  * Keeping track of game state (maze layout, player positions, scores).
* Make sure it supports **10+ players**.

**GitHub task**: Create `server/` module and define networking protocol (e.g. JSON messages: `{player_id, x, y, action}`).

---

### 👨‍💻 Zaki Awdallah – **Client Networking & Integration**

* Implement **UDP client** (connect to server with IP + username).
* Sync with server to receive maze + other players’ positions.
* Send player input (movement, shooting, etc.) to server.
* Work closely with Person 1 to define protocol.

**GitHub task**: Create `client/network.rs` with socket handling.

---

### 👨‍💻 Moaz Razem – **Graphics & Rendering Lead**

* Use a Rust game engine (recommend **ggez** for 2D or **Bevy** for 3D-like rendering).
* Implement:

  * Maze rendering (walls).
  * Player’s first-person view.
  * Mini-map overlay.
  * FPS counter.
* Optimize so FPS > 50.

**GitHub task**: Create `client/render.rs`.

---

### 👨‍💻 Amro Khweis – **Game Logic & Levels**

* Implement:

  * Maze generation & 3 increasing difficulty levels.
  * Player movement mechanics (walking, collision with walls).
  * Dead ends, exits, etc.
* Bonus: Auto-maze generator algorithm.

**GitHub task**: Create `game/logic.rs`.

---

### 👨‍💻 Noor Halabi – **Testing & Performance**

* Write **integration tests** for client-server communication.
* Stress-test with multiple fake clients.
* Measure FPS and optimize where possible.
* Write CI/CD workflows on GitHub (e.g. build on push, run tests).

**GitHub task**: Setup `tests/` + GitHub Actions workflow.

---

### 👨‍💻 Jehad Alami – **Documentation**

* Write README and document how to run the server/client.
* Implement **username input + IP input screen**.
* Optionally: display the connected players list.

**GitHub task**: Work on `docs/README.md` + `client/ui.rs` (basic input).
