# RUSTLE 🦀➕⚡

**Rustle** is a brutal, high-performance WebAssembly engine masquerading as a Wordle clone. It is an experiment in digital entropy, adversarial AI, and generational tribalism, built entirely from first-principles in Type-Safe Rust.

No JavaScript frameworks. No Node.js. No noise. Just a compiled Wasm binary communicating asynchronously with an embedded SQLite Axum backend.

---

## 🕹 The System
- **Daily Protocol**: One global string per day, mathematically synchronized for all nodes via a strict UTC backend timestamp.
- **New Game+ (Adversarial Mode)**: Unlocked after the daily protocol. Challenge "The System"—an AI that dynamically shifts the underlying truth after every input to maximize your path length. Hard mode is mandatory.
- **Generational Tribalism**: Agents must assign themselves to a biological faction (Gen Alpha, Gen Z, Millennials, Gen X, Boomers). The system tracks global metrics and actively mocks your errors using the localized dialect of your selected tribe.
- **Progressive Web App (PWA)**: The client installs locally to your mobile or desktop hardware with a built-in Service Worker for offline survival.
- **Empirical Persistence**: The backend runs an embedded, transactional `sqlx` SQLite database to guarantee total data integrity.
- **ARIA Accessibility**: Full screen-reader support ensures that all agents, regardless of optical hardware limitations, can interface with the terminal.

---

## 🐳 Deployment (Docker)

Rustle is distributed as a multi-stage, heavily optimized container. It features atomic data execution and IP-based rate limiting (`tower_governor`) to prevent DDOS noise.

### **Docker Compose (Recommended)**
Perfect for deployment behind a Cloudflare Tunnel or local network.

```yaml
version: "3.8"
services:
  rustle:
    image: ghcr.io/ubermetroid/rustle:latest
    container_name: rustle
    restart: unless-stopped
    ports:
      - "7583:7583"
    volumes:
      - ./data:/app/data
    security_opt:
      - no-new-privileges:true
```

2. Launch the container:
```bash
docker-compose up -d
```

### **Docker Run**
```bash
docker run -d \
  --name rustle \
  -p 7583:7583 \
  -v ./data:/app/data \
  --restart unless-stopped \
  ghcr.io/ubermetroid/rustle:latest
```

---

## 🛠 Local Development

If you want to build from source or contribute to the engine.

### **Prerequisites**
- **Rust**: [rustup.rs](https://rustup.rs/)
- **Wasm Target**: `rustup target add wasm32-unknown-unknown`
- **Trunk**: `cargo install trunk`

### **Build & Run**
1. Clone the repo:
   ```bash
   git clone https://github.com/UberMetroid/rustle.git
   cd rustle
   ```
2. Build the frontend:
   ```bash
   cd wordle-ui
   trunk build --release
   ```
3. Run the backend server:
   ```bash
   cd ../wordle-server
   cargo run --release
   ```
4. Open `http://127.0.0.1:7583` in your browser.

---

## 🏗 Tech Stack
- **Frontend**: [Leptos](https://leptos.dev/) (Rust CSR)
- **Backend API**: [Axum](https://github.com/tokio-rs/axum) with Tokio async runtime
- **Security**: IP-based rate limiting via `tower_governor`
- **Engine**: Custom pure-Rust library (`wordle-engine`)
- **Styling**: Tailwind CSS (Integrated via Trunk)
- **Runtime**: WebAssembly (Wasm)

---

## 📐 Engineering Standards
This project adheres to a strict set of engineering mandates to ensure maximum maintainability and readability:
- **Monolithic Deconstruction**: No single Rust source file exceeds **256 lines**. Logic is aggressively modularized into specialized components and modules.
- **Self-Documenting & Empirical**: Every public struct, field, and function is documented with standard Rust `///` doc-comments. 
- **Verifiable Logic**: The codebase features built-in testing suites for every crate:
  - `wordle-engine`: Unit tests for core matching and adversarial AI.
  - `wordle-server`: Integration tests using in-memory SQLite.
  - `wordle-ui`: WebAssembly unit tests for state and reactive logic.
- **Type-Safe Networking**: All communication between the frontend and backend is handled via shared Rust models and `serde`, ensuring zero runtime schema mismatches.

---

## 📜 Credits
While the code is 100% Rust, the visual inspiration and logic structure were informed by:
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
