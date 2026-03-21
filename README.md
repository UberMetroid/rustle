# React Wordle

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Legacy%20Docker%20CI.yml?label=Legacy%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rust-wordle)](https://github.com/UberMetroid/rust-wordle)

This repository provides containerized versions of the popular Wordle game.

---

## 🐳 Container Installation

Images are hosted on the GitHub Container Registry (**GHCR**).

### **Latest Version**
*Modern React implementation. Default container port: **7583***

#### **Docker Run**
```bash
docker run -d -p 7583:7583 --name wordle-latest ghcr.io/ubermetroid/rust-wordle/latest:latest
```

#### **Docker Compose**
```yaml
version: "3"
services:
  wordle:
    image: ghcr.io/ubermetroid/rust-wordle/latest:latest
    container_name: wordle-latest
    ports:
      - "7583:7583"
```

---

### **Legacy Version**
*Original Wordle clone. Default container port: **80***

#### **Docker Run**
```bash
docker run -d -p 8081:80 --name wordle-legacy ghcr.io/ubermetroid/rust-wordle/legacy:latest
```

#### **Docker Compose**
```yaml
version: "3"
services:
  wordle-legacy:
    image: ghcr.io/ubermetroid/rust-wordle/legacy:latest
    container_name: wordle-legacy
    ports:
      - "8081:80"
```

---

## Original Projects
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle)
