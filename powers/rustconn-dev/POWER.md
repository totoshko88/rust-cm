---
name: "rustconn-dev"
displayName: "RustConn Development"
description: "Development workflow for RustConn project with automated clippy, fmt, and build checks"
keywords: ["rustconn", "rust", "clippy", "fmt", "cargo"]
author: "RustConn Team"
---

# RustConn Development

## Overview

Автоматизований workflow для розробки RustConn — перевірка коду через clippy, форматування, тести та збірка.

## Конфігурація проєкту

### Rust версія
- **MSRV:** 1.87
- **Edition:** 2021

### Clippy налаштування
- Рівень: `pedantic` + `nursery` (суворий)
- `unsafe_code = "forbid"` — unsafe заборонено
- Cognitive complexity: 25
- Max arguments: 7

### Форматування
- Max width: 100 символів
- Tabs: 4 пробіли
- Line endings: Unix (LF)
- Auto reorder imports

## Команди

### Швидка перевірка
```bash
# Перевірка компіляції (швидко)
cargo check --all-targets

# Clippy з усіма warnings
cargo clippy --all-targets

# Clippy з автовиправленням
cargo clippy --all-targets --fix --allow-dirty
```

### Форматування
```bash
# Перевірка форматування
cargo fmt --check

# Автоформатування
cargo fmt
```

### Тести
```bash
# Всі тести
cargo test

# Property тести
cargo test -p rustconn-core --test property_tests

# Конкретний тест
cargo test -p rustconn-core test_name
```

### Збірка
```bash
# Debug
cargo build

# Release
cargo build --release
```

## Workflow перед комітом

Виконай послідовно:

```bash
# 1. Форматування
cargo fmt

# 2. Clippy з виправленнями
cargo clippy --all-targets --fix --allow-dirty

# 3. Перевірка що все компілюється
cargo build --all-targets

# 4. Тести
cargo test
```

## Структура крейтів

| Крейт | Призначення | GUI залежності |
|-------|-------------|----------------|
| `rustconn-core` | Бізнес-логіка | ❌ Заборонено |
| `rustconn` | GTK4 GUI | ✅ gtk4, vte4, adw |
| `rustconn-cli` | CLI | ❌ Тільки core |

**Правило:** Якщо код не потребує GTK → `rustconn-core`

## Типові помилки Clippy

### `cognitive_complexity`
Функція занадто складна. Розбий на менші функції.

### `too_many_arguments`
Більше 7 аргументів. Створи struct для параметрів.

### `missing_errors_doc`
Додай `# Errors` секцію в документацію для функцій що повертають `Result`.

## Патерни коду

### Помилки
```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("опис: {0}")]
    Variant(String),
}
```

### Credentials
```rust
use secrecy::SecretString;
let password: SecretString = SecretString::new(value.into());
```

### Async traits
```rust
#[async_trait::async_trait]
impl MyTrait for MyStruct {
    async fn method(&self) -> Result<(), Error> { /* ... */ }
}
```

## Troubleshooting

### Clippy не бачить змін
```bash
cargo clean
cargo clippy --all-targets
```

### Конфлікт форматування
```bash
cargo fmt
git diff  # перевір зміни
```

### Тести падають після clippy --fix
Clippy іноді робить некоректні виправлення. Перевір `git diff` і відкоти якщо потрібно.
