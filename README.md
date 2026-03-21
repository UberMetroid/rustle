# Rustle

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rust-wordle)](https://github.com/UberMetroid/rust-wordle)

This is a high-performance, pure Rust clone of the popular Wordle game. Built with **Leptos** (Rust Frontend) and a custom Rust business logic engine.

---

## 🐳 Container Installation

The application is distributed as a single container image containing the WebAssembly frontend and Nginx server.

*Pure Rust/Leptos implementation. Default container port: **7583***

### **Docker Run**
```bash
docker run -d -p 7583:7583 --name wordle ghcr.io/ubermetroid/rust-wordle/latest:latest
```

### **Docker Compose**
```yaml
version: "3"
services:
  wordle:
    image: ghcr.io/ubermetroid/rust-wordle/latest:latest
    container_name: wordle
    ports:
      - "7583:7583"
```

---

## 🛠 Tech Stack
*   **Frontend**: [Leptos](https://leptos.dev/) (Rust + Wasm)
*   **Engine**: Custom Rust library (`wordle-engine`)
*   **Build Tool**: [Trunk](https://trunkrs.dev/)
*   **Styles**: Tailwind CSS
*   **Server**: Nginx (Alpine)

---

## Original Projects
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
