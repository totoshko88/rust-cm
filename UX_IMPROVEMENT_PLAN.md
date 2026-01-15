# RustConn UX Improvement Plan

## Статус виконання

### Завдання 1: Кнопка "Connection History" в sidebar toolbar ✅
- [x] Додати кнопку після Group Operations Mode
- [x] Підключити до існуючого action `win.show-history`
- Файл: `rustconn/src/sidebar_ui.rs` - `create_sidebar_bottom_toolbar()`

### Завдання 2: Run Snippet в context menu ✅
- [x] Додати пункт "Run Snippet..." в context menu для з'єднань
- [x] Створити action `win.run-snippet-for-connection`
- [x] Показувати snippet picker при виборі
- Файли:
  - `rustconn/src/sidebar_ui.rs` - `show_context_menu_for_item()`
  - `rustconn/src/window.rs` - `setup_snippet_actions()`

### Завдання 3: Персистенція історії пошуку ✅
- [x] Додати `search_history: Vec<String>` в `UiSettings`
- [x] Додати методи `add_search_history()`, `clear_search_history()` в `UiSettings`
- [x] Додати `settings_mut()` та `save_settings()` в `AppState`
- [x] Завантажувати історію при створенні sidebar
- [x] Зберігати історію при додаванні нового запиту
- Файли:
  - `rustconn-core/src/config/settings.rs` - `UiSettings`
  - `rustconn/src/state.rs` - `settings_mut()`, `save_settings()`
  - `rustconn/src/sidebar.rs` - `load_search_history()`
  - `rustconn/src/window.rs` - завантаження та збереження

---

## Зміни у файлах

### rustconn-core/src/config/settings.rs
- Додано `search_history: Vec<String>` в `UiSettings`
- Додано методи `add_search_history()`, `clear_search_history()`
- Константа `MAX_SEARCH_HISTORY_ENTRIES = 20`

### rustconn/src/state.rs
- Додано `settings_mut(&mut self) -> &mut AppSettings`
- Додано `save_settings(&self) -> Result<(), String>`

### rustconn/src/sidebar.rs
- Додано `load_search_history(&self, history: &[String])`

### rustconn/src/sidebar_ui.rs
- Додано кнопку History в `create_sidebar_bottom_toolbar()`
- Додано "Run Snippet..." в `show_context_menu_for_item()`

### rustconn/src/window.rs
- Завантаження історії пошуку при створенні sidebar
- Збереження історії при Enter та втраті фокусу
- Додано action `run-snippet-for-connection`

### rustconn/src/dialogs/settings/ui_tab.rs
- Додано `search_history: Vec::new()` в конструктор `UiSettings`

---

## Майбутні покращення (з аналізу)

### Високий пріоритет
- [ ] Command Palette (Ctrl+P для з'єднань, Ctrl+Shift+P для команд)
- [ ] Favorites/Pinning з'єднань

### Середній пріоритет
- [ ] Tab Grouping за проектами/середовищами
- [ ] Custom Keybindings

### Низький пріоритет
- [ ] Vim/Emacs keybinding modes
- [ ] Snippet versioning
