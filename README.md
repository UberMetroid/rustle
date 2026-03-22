# RUSTLE 🦀➕⚡

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rustle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rustle)](https://github.com/UberMetroid/rustle)

**Rustle** is a high-performance, pure Rust Wordle clone featuring an adversarial AI Mode (New Game+), 80s arcade physics, and a responsive "Neon on Onyx" UI. 

No JavaScript. No TypeScript. Just 100% Type-Safe Rust.

---

## 🕹 Features
- **Daily Protocol**: One global word per day, synchronized for all users via UTC timestamp.
- **New Game+ (Adversarial/Absurdle Mode)**: Unlocked after the daily game. Challenge "The System"—an AI that dynamically shifts the solution after every guess to maximize digital entropy. Hard mode is strictly enforced.
- **Strict Hard Mode**: Forces you to use revealed hints in subsequent guesses. Try to cheat the system, and you'll be mocked.
- **Team Leaderboards**: Choose your faction (Red, Orange, Yellow, Green, Blue) and compete globally for the highest average score. Points are awarded based on how few guesses you use.
- **Expanded Dictionary**: Over 10,000 valid English words accepted as guesses, with a curated list of ~2,300 possible daily solutions.
- **80s Radical Snark**: Contextual mocking, error messages, and victory comments based on your performance and UI interactions.
- **Neon Interaction**: Glowing UI elements, power rings (Hard Mode), and neon wipes (Normal Mode) on every interaction.
- **Cyber Burst Celebration**: High-gravity pixel fireworks for breaching the system.
- **Privacy-First Sharing**: Theme-matched emoji blocks with sanitized, spoiler-free comments.

---

## 🐳 Deployment (Docker / Synology)

Rustle is distributed as a multi-stage optimized container containing the Wasm frontend and a hardened Nginx Alpine server.

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
2. Start the development server:
   ```bash
   cd wordle-ui
   trunk serve --port 7583
   ```
3. Open `http://localhost:7583` in your browser.

---

## 🏗 Tech Stack
- **Frontend**: [Leptos](https://leptos.dev/) (Rust CSR)
- **Engine**: Custom Rust library (`wordle-engine`)
- **Styling**: Tailwind CSS (Integrated via Trunk)
- **Runtime**: WebAssembly (Wasm)
- **Production Server**: Nginx (Alpine) with optimized SPA routing

---

## 📜 Credits
While the code is 100% Rust, the visual inspiration and logic structure were informed by:
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
