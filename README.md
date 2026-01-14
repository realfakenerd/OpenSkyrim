# OpenSkyrim

OpenSkyrim is a modern, open-source project building a robust toolset and runtime environment for Skyrim, leveraging the power of **Rust**, **Tauri**, and **Svelte 5**.

## 🚀 Tech Stack

This project uses a high-performance monorepo architecture:

-   **Core Backend:** [Rust](https://www.rust-lang.org/) with [Tauri v2](https://tauri.app/)
-   **Game Engine / ECS:** [Bevy](https://bevyengine.org/)
-   **Frontend UI:** [Svelte 5](https://svelte.dev/) with [TypeScript](https://www.typescriptlang.org/)
-   **Build System:** [Turbo Repo](https://turbo.build/)
-   **Runtime & Package Manager:** [Bun](https://bun.sh/)
-   **Documentation:** [Astro Starlight](https://starlight.astro.build/)

## 📂 Project Structure

```text
.
├── packages/
│   └── ui/          # Svelte 5 frontend application
├── src-tauri/       # Rust backend (Tauri + Bevy)
├── docs/            # Documentation site (Astro)
├── conductor/       # Project management & specifications
└── turbo.json       # Build pipeline configuration
```

## 🛠️ Getting Started

### Prerequisites

-   **Rust:** Latest stable release ([Install](https://www.rust-lang.org/tools/install))
-   **Bun:** Latest version ([Install](https://bun.sh/))
-   **System Dependencies:** Requirements for Tauri (Linux/macOS/Windows) -> [See Tauri Docs](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/OpenSkyrim.git
    cd OpenSkyrim
    ```

2.  **Install dependencies:**
    ```bash
    bun install
    ```

### Development

To start the development environment for all services (UI, Tauri app, Docs):

```bash
bun dev
```

Or run specific parts:

-   **Desktop App:** `cd packages/ui && bun tauri dev`
-   **Documentation:** `cd docs && bun dev`

## 🤝 Contributing

We follow a strict spec-driven development process managed by **Conductor**.

Please read the following guides before contributing:
-   [Workflow Guide](conductor/workflow.md)
-   [Code Style Guides](conductor/code_styleguides/)
-   [Product Guidelines](conductor/product-guidelines.md)

## 📄 License

[MIT](LICENSE)