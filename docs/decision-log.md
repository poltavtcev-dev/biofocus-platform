# Decision Log (Architecture Decision Records)

| ID | Date | Decision | Reason | Rejected Alternatives |
| :--- | :--- | :--- | :--- | :--- |
| ADR-001 | 2026-07-27 | Modular Monolith in Rust | Низкое потребление ОЗУ (<30MB), безопасность памяти, скорость. | Node.js, Go, Python |
| ADR-002 | 2026-07-27 | SQLite с режимом WAL | Local-first концепция, отсутствие сложных внешних серверов. | PostgreSQL, MongoDB |
| ADR-003 | 2026-07-27 | Tauri v2 для Desktop UI | Компактный размер дистрибутива, минимальная нагрузка на CPU/RAM. | Electron |
| ADR-004 | 2026-07-27 | Лицензия AGPLv3 | Защита открытого кода от создания закрытых коммерческих форков корпорациями. | MIT, Apache 2.0 |
| ADR-005 | 2026-08-05 | Opt-in LAN ingest bind for companion dogfood; default remains loopback (`127.0.0.1`) | Physical iPhone on Wi-Fi cannot reach loopback-only Desktop; LAN must be explicit so local-only stays the safe default. Knobs: `BIOFOCUS_INGEST_LAN=1` → `0.0.0.0`, optional `BIOFOCUS_INGEST_BIND_HOST`. Bearer pairing token still required. | Always-on `0.0.0.0`; mDNS/TLS in Phase 5; cloud relay |