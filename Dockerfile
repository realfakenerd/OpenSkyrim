# Use the official Microsoft Rust Dev Container image as the base
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies for Tauri and Bevy
# Tauri: libwebkit2gtk-4.1-dev, libgtk-3-dev, libayatana-appindicator3-dev, librsvg2-dev, libssl-dev
# Bevy: libasound2-dev, libudev-dev, libx11-dev, libxkbcommon-dev, libwayland-dev
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    # Build tools
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    pkg-config \
    # Tauri dependencies
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    # Bevy dependencies
    libasound2-dev \
    libudev-dev \
    libx11-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libxrandr-dev \
    libxi-dev \
    libxcursor-dev \
    libxinerama-dev \
    libgl1-mesa-dev \
    # Cleanup
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install Bun
ENV BUN_INSTALL=/usr/local
RUN curl -fsSL https://bun.sh/install | bash

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=dialog

# Set the default user
USER vscode
