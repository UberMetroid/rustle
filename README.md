# React Wordle

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rust-wordle)](https://github.com/UberMetroid/rust-wordle)

This repository provides a containerized version of the popular Wordle game.

---

## 🐳 Container Installation

Images are hosted on the GitHub Container Registry (**GHCR**).

*Modern React implementation. Default container port: **7583***

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

## Original Projects
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
