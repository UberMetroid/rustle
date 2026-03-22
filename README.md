# RUSTLE

A high-performance WebAssembly word game with competitive team mechanics and adversarial AI.

## Features

- **Daily Protocol**: One global word per day, synchronized via UTC timestamp
- **New Game+ (Adversarial Mode)**: AI opponent that shifts the answer to maximize challenge
- **Team Competition**: 6 color teams competing for daily glory
- **PWA Support**: Installable with offline capability
- **Type-Safe Architecture**: 100% Rust with shared models between frontend and backend

## Tech Stack

- **Frontend**: Leptos (Rust CSR) → WebAssembly
- **Backend**: Axum with Tokio async runtime
- **Database**: SQLite via sqlx
- **Styling**: Tailwind CSS

## Project Structure

```
rustle/
├── wordle-engine/     # Core game logic (Wasm-compatible)
│   ├── src/lib.rs    # Solution calculation, status matching, adversarial AI
│   ├── src/tests.rs   # Unit tests
│   └── benches/       # Benchmarks
├── wordle-server/     # Backend API server
│   ├── src/main.rs    # Entry point
│   ├── src/lib.rs    # Router, middleware, rollover task
│   ├── src/handlers.rs # API endpoints
│   ├── src/models.rs  # Data types
│   ├── src/db.rs     # Database initialization
│   └── tests/        # Integration tests
├── wordle-ui/         # Frontend Wasm application
│   ├── src/          # Leptos components
│   └── dist/         # Built assets (after trunk build)
└── Cargo.toml        # Workspace configuration
```

## Local Development

### Prerequisites

- Rust (latest stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`
- Node.js (for Tailwind CSS processing)

### Build & Run

```bash
# Build frontend
cd wordle-ui
trunk build --release

# Run backend (from project root or wordle-server directory)
cd wordle-server
cargo run --release
```

Open `http://127.0.0.1:7583` in your browser.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:data/wordle.db?mode=rwc` | SQLite database path |
| `DIST_PATH` | `../wordle-ui/dist` | Path to static frontend files |

### Running Tests

```bash
# Run all tests (engine + server integration)
cargo test

# Run only engine tests
cargo test -p wordle-engine

# Run only server tests
cargo test -p wordle-server

# Run benchmarks
cargo bench -p wordle-engine
```

### Docker Deployment

```bash
# Using Docker Compose
docker-compose up -d

# Or with Docker Run
docker run -d --name rustle \
  -p 7583:7583 \
  -v ./data:/app/data \
  ghcr.io/ubermetroid/rustle:latest
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/global-stats.json` | Team scores and game state |
| POST | `/api/score` | Submit player score |

### Score Submission

```json
{
  "player_id": "string (max 64 chars, alphanumeric/_/-)",
  "team": "red|orange|yellow|green|blue|purple",
  "points_delta": -5 to 10
}
```

## Architecture Notes

- **Game Logic**: Pure Rust in `wordle-engine`, compiles to both native (tests) and Wasm (frontend)
- **Adversarial AI**: Uses status mask bucketing to find worst-case word patterns
- **Rate Limiting**: 500 requests per second, burst of 10 via `tower_governor`
- **Database**: SQLite with transactional daily rollover at midnight UTC

## Credits

Inspired by:
- [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
- [modem7/react-wordle](https://github.com/modem7/react-wordle)
