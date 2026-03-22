# RUSTLE

A high-performance WebAssembly word game with competitive team mechanics and adversarial AI.
Built entirely in Rust with 100% type-safe code between frontend and backend.

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
- **Logging**: tracing + tracing-subscriber

## Project Structure

```
rustle/
├── wordle-engine/           # Core game logic (Wasm-compatible)
│   ├── src/
│   │   ├── lib.rs          # Solution, status matching, adversarial AI
│   │   ├── words.rs        # Word lists
│   │   ├── tests.rs        # Unit tests
│   │   └── prop_tests.rs   # Property-based tests
│   └── benches/            # Criterion benchmarks
├── wordle-server/           # Backend API server
│   ├── src/
│   │   ├── main.rs         # Entry point + logging setup
│   │   ├── lib.rs          # Router, middleware, rollover task
│   │   ├── handlers.rs     # API endpoint handlers
│   │   ├── models.rs       # Data types
│   │   └── db.rs           # Database initialization
│   └── tests/              # Integration tests
├── wordle-ui/               # Frontend Wasm application
│   ├── src/                # Leptos components
│   └── dist/               # Built assets (after trunk build)
└── Cargo.toml              # Workspace configuration
```

## Quick Start

### Prerequisites

- Rust (latest stable)
- `wasm32-unknown-unknown` target
- Trunk: `cargo install trunk`
- Node.js (for Tailwind CSS)

### Build & Run

```bash
# Add wasm target
rustup target add wasm32-unknown-unknown

# Build frontend
cd wordle-ui
trunk build --release

# Run server (from project root)
cd ../wordle-server
cargo run --release
```

Open `http://127.0.0.1:7583` in your browser.

## Environment Variables

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:data/wordle.db?mode=rwc` | SQLite database path |
| `DIST_PATH` | `../wordle-ui/dist` | Path to static frontend files |
| `API_KEY` | (none) | API key for scoring endpoint authentication |
| `ALLOWED_ORIGINS` | `*` | Comma-separated CORS origins (e.g., `https://example.com,https://app.example.com`) |
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |

### Security Notes

- **API_KEY**: When set, scoring endpoint requires `X-API-Key` header
- **ALLOWED_ORIGINS**: Set to specific domains in production to prevent CSRF
- Default CORS is permissive for development convenience

## API Reference

### GET /global-stats.json

Returns current team scores and game state.

**Response:**
```json
{
  "yellow": { "points": 150, "players": 12, "yesterday_total": 200 },
  "red": { "points": 120, "players": 8, "yesterday_total": 180 },
  "green": { "points": 200, "players": 15, "yesterday_total": 160 },
  "blue": { "points": 90, "players": 6, "yesterday_total": 220 },
  "purple": { "points": 180, "players": 10, "yesterday_total": 140 },
  "orange": { "points": 110, "players": 7, "yesterday_total": 190 },
  "yesterday_winner": "green",
  "current_date": "2024-01-15",
  "server_utc_timestamp": 1705276800000
}
```

### POST /api/score

Submit player score (requires API_KEY when configured).

**Headers:**
```
Content-Type: application/json
X-API-Key: your-api-key (when API_KEY is set)
```

**Request:**
```json
{
  "player_id": "player123",
  "team": "red",
  "points_delta": 5
}
```

**Response:**
```json
{ "success": true }
```

**Validation:**
- `player_id`: 1-64 characters, alphanumeric + `_` + `-`
- `team`: One of: red, orange, yellow, green, blue, purple
- `points_delta`: Integer between -5 and 10

## Testing

```bash
# Run all tests
cargo test

# Run engine tests only
cargo test -p wordle-engine

# Run server integration tests only
cargo test -p wordle-server

# Run with output capture
cargo test -- --nocapture

# Run benchmarks
cargo bench -p wordle-engine
```

### Test Coverage

- **wordle-engine**: 14 unit tests + property-based tests
- **wordle-server**: 9 integration tests

## Docker Deployment

```bash
# Using Docker Compose
docker-compose up -d

# Or with Docker Run
docker run -d --name rustle \
  -p 7583:7583 \
  -v ./data:/app/data \
  -e API_KEY=your-secret-key \
  -e ALLOWED_ORIGINS=https://yourdomain.com \
  ghcr.io/ubermetroid/rustle:latest
```

## Architecture

### Game Logic (wordle-engine)

Pure Rust library that compiles to both native (tests) and Wasm (frontend):

- `get_solution(timestamp)` - Deterministic daily word calculation
- `calculate_statuses(solution, guess)` - Returns correct/present/absent
- `check_hard_mode(guess, prev_guesses, prev_statuses)` - Validates Hard Mode rules
- `get_adversarial_step(guess, pool)` - AI that picks worst-case patterns

### Adversarial AI (New Game+)

The AI uses a bucketing algorithm:
1. For each candidate solution, calculate status mask for the current guess
2. Group solutions by their status mask
3. Return the pattern from the largest bucket (worst case for player)

### Daily Rollover

A background task runs every 10 seconds checking for UTC midnight:
1. Archives winning team's score to `yesterday_total`
2. Resets all team points and player counts
3. Clears player team assignments for new day

## Engineering Standards

- **File Size**: No source file exceeds 256 lines
- **Documentation**: All public functions have rustdoc comments
- **Testing**: Unit tests + property-based tests + integration tests
- **Error Handling**: Structured logging via `tracing`, never silent failures

## License

MIT

## Credits

Inspired by:
- [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
- [modem7/react-wordle](https://github.com/modem7/react-wordle)
