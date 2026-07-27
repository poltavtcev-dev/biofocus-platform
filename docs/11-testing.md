# 11. Testing Strategy

## 1. Test Layers
1. **Unit Tests (Rust):** Покрытие алгоритмов расчете `FocusScore` и `StressIndex` изолированными синтетическими данными.
2. **Integration Tests (`storage`):** Тестирование миграций и транзакционной записи в SQLite (in-memory mode).
3. **Pipeline E2E Tests:** Подача потока mock-файл `Observation` -> проверка правильности генерации `Signal` и `Feature`.