# 15. Engineering Principles

1. **Keep Crates Isolated:** Пакеты в `crates/` не должны зависеть от Tauri. Они должны спокойно компилироваться в чистый CLI или фоновый сервис.
2. **Explicit Errors:** Отказ от `unwrap()` в продакшн-коде. Все ошибки обрабатываются через `thiserror` / `anyhow`.
3. **No Unbounded Memory Streams:** Все каналы `tokio::sync::mpsc` должны иметь ограничение по размеру (bounded buffer), чтобы падение связи с мобилкой не привело к переполнению ОЗУ.