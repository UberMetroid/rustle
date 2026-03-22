# RUSTLE 🦀➕⚡

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rustle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rustle)](https://github.com/UberMetroid/rustle)

**Rustle** is a high-performance, full-stack Rust Wordle clone featuring an adversarial AI Mode (New Game+), global leaderboards, snappy WebAssembly rendering, and a responsive "Neon on Onyx" UI. 

No JavaScript. No TypeScript. Just 100% Type-Safe Rust from the Wasm frontend to the Axum backend.

---

## 🕹 Features
- **Daily Protocol**: One global word per day, mathematically synchronized for all users via a strict UTC backend timestamp.
- **New Game+ (Adversarial/Absurdle Mode)**: Unlocked after the daily game. Challenge "The System"—an AI that dynamically shifts the solution after every guess to maximize digital entropy. Hard mode is strictly enforced.
- **Strict Hard Mode**: Forces you to use revealed hints in subsequent guesses. Try to cheat the system, and you'll be mocked.
- **Generational Team Leaderboards**: Choose your faction based on your generational vibe (Red/Alpha, Orange/Z, Yellow/Millennial, Green/X, Blue/Boomer) and compete globally. Points are synced to the backend in real-time.
- **Progressive Web App (PWA)**: Fully installable to your mobile or desktop home screen with a built-in Service Worker for blazing-fast offline caching.
- **Full ARIA Accessibility**: Native screen-reader support ensures that visually impaired players can engage with the terminal and grid mechanics seamlessly.
- **SQLite Data Persistence**: The backend uses an embedded, transactional `sqlx` SQLite database to guarantee total data integrity and zero lock-contention under heavy loads.
- **Contextual Generational Snark**: The game aggressively mocks your performance, errors, and UI spam using an expanded dictionary of slang and emojis perfectly matched to your currently selected faction.
- **Expanded Dictionary**: Over 10,000 valid English words accepted as guesses, with a curated list of ~2,300 possible daily solutions.
- **Neon Interaction**: Glowing UI elements, power rings (Hard Mode), and neon wipes (Normal Mode) on every interaction.
- **Cyber Burst Celebration**: High-gravity pixel fireworks for breaching the system.
- **Privacy-First Sharing**: Theme-matched emoji blocks with sanitized, spoiler-free comments.

---

## 🐳 Deployment (Docker / Synology)

Rustle is distributed as a multi-stage optimized container containing the Wasm frontend and a lightning-fast Rust (`Axum`) backend. It features atomic file-saving and IP-based rate limiting to prevent global leaderboard abuse.

### **Docker Compose (Recommended)**
Perfect for deployment on **Synology**, **Tailscale**, or behind a **Cloudflare Tunnel**.

1. Create a `docker-compose.yml` file:
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

## 📜 Credits
While the code is 100% Rust, the visual inspiration and logic structure were informed by:
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
