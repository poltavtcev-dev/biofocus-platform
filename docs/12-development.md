# 12. Local Development Guide

## Prerequisites
- Rust stable (edition 2024)
- Node.js >= 20.x, pnpm
- Tauri CLI v2 (`cargo install tauri-cli --version "^2.0"`)

## Commands
```bash
# Клонирование репозитория
git clone [https://github.com/poltavtcev-dev/biofocus-platform.git](https://github.com/poltavtcev-dev/biofocus-platform.git)
cd biofocus-platform

# Запуск Core Runtime & Desktop App в dev-режиме
pnpm install
pnpm tauri dev