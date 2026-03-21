# React Wordle

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Latest%20Docker%20CI.yml?label=Latest%20Docker%20Build)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Legacy%20Docker%20CI.yml?label=Legacy%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rust-wordle)](https://github.com/UberMetroid/rust-wordle)

This is a clone project of the popular word guessing game we all know and love. Made using React, Typescript, and Tailwind.

[**Try it out!**](https://ubermetroid.github.io/rust-wordle/)

---

## 🚀 Installation & Setup

### 🐳 Running with Docker (Recommended)

Images are automatically built and hosted on the GitHub Container Registry (**GHCR**).

#### **Latest Version**
The most up-to-date version of the React application.
*   **Docker Run:**
    ```bash
    docker run -d -p 7583:7583 --name wordle-latest ghcr.io/ubermetroid/rust-wordle/latest:latest
    ```
*   **Docker Compose:**
    ```yaml
    version: "3"
    services:
      wordle:
        image: ghcr.io/ubermetroid/rust-wordle/latest:latest
        container_name: wordle-latest
        ports:
          - "7583:7583"
    ```

#### **Legacy Version**
The original Wordle clone, closest to the classic experience.
*   **Docker Run:**
    ```bash
    docker run -d -p 8081:80 --name wordle-legacy ghcr.io/ubermetroid/rust-wordle/legacy:latest
    ```
*   **Docker Compose:**
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

### 💻 Local Development

If you want to run or modify the code locally, follow these steps:

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/UberMetroid/rust-wordle.git
    cd rust-wordle
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Start the development server:**
    ```bash
    npm start
    ```
    The app will be available at `http://localhost:3000`.

4.  **Build for production:**
    ```bash
    npm run build
    ```

---

## 📖 About the Versions

### Latest
This version is based on the [cwackerfuss/react-wordle](https://github.com/cwackerfuss/react-wordle) project. It features a modern React architecture and is actively maintained. Note that the "Word of the Day" might differ from the NYT version.
**Default Container Port: 7583**

### Legacy
This is the original clone of the Wordle website, served via Nginx. It is intended to remain as close to the original experience as possible and should align with the classic "Word of the Day".
**Default Container Port: 80**

---

## Project Screenshot

![image](https://user-images.githubusercontent.com/4349962/158677511-50faa60b-26a1-4880-a580-b433389f03aa.png)

## Original Projects
*   [Cwackerfuss/React-Wordle](https://github.com/cwackerfuss/react-wordle)
*   [modem7/react-wordle](https://github.com/modem7/react-wordle) (Base for this fork)
