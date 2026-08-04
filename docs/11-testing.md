# 11. Testing Strategy

## 1. Test Layers
1. **Unit Tests (Rust):** Покрытие алгоритмов расчете `FocusScore` и `StressIndex` изолированными синтетическими данными.
2. **Integration Tests (`storage`):** Тестирование миграций и транзакционной записи в SQLite (in-memory mode).
3. **Collector integration (`macos-collector`):** mock probes → bounded channel → `spawn_persist_worker` → SQLite; stop/idle (no busy-loop after `stop_stream`). See `docs/12-development.md` § Collector test suite.
4. **Companion → ingest (`companion`):** sample `heart_rate` Observation → loopback `POST /v1/ingest`; 401/network explicit. See `apps/companion/README.md`.
5. **Pipeline E2E Tests:** Подача потока mock-файл `Observation` -> проверка правильности генерации `Signal` и `Feature`.