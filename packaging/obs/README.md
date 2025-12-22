# RustConn OBS Packaging

Файли для автоматичної збірки на [Open Build Service](https://build.opensuse.org/).

## Структура файлів

| Файл                  | Призначення                                            |
|-----------------------|--------------------------------------------------------|
| `_service`            | Автоматичне завантаження з Git та vendoring залежностей |
| `rustconn.spec`       | RPM spec для openSUSE, Fedora, RHEL                    |
| `rustconn.changes`    | Changelog для RPM                                      |
| `debian.*`            | Файли для збірки DEB (Ubuntu, Debian)                  |
| `AppImageBuilder.yml` | Конфігурація для AppImage                              |

## Налаштування OBS

### 1. Створення проєкту

1. Увійдіть на https://build.opensuse.org/
2. Натисніть "Your Home Project" → "Create Subproject"
3. Назва: `rustconn`
4. Опис: "Modern connection manager for Linux"

### 2. Налаштування репозиторіїв

В налаштуваннях проєкту додайте репозиторії для збірки:

**RPM:**
- openSUSE Tumbleweed
- openSUSE Leap 15.6
- Fedora 40
- Fedora 39

**DEB:**
- Debian 12 (Bookworm)
- Ubuntu 24.04 (Noble)
- Ubuntu 22.04 (Jammy)

### 3. Завантаження файлів

```bash
# Встановіть osc (OBS command-line client)
# openSUSE: sudo zypper install osc
# Fedora: sudo dnf install osc
# Ubuntu: sudo apt install osc

# Налаштуйте osc
osc config

# Checkout проєкту
osc checkout home:YOUR_USERNAME:rustconn

# Скопіюйте файли
cd home:YOUR_USERNAME:rustconn
cp /path/to/packaging/obs/* .

# Відредагуйте _service - замініть YOUR_USERNAME на ваш GitHub username
# Відредагуйте email в .changes та debian.changelog

# Завантажте файли
osc add *
osc commit -m "Initial package"
```

### 4. Запуск збірки

Після commit OBS автоматично запустить збірку. Статус можна переглянути:
- Веб-інтерфейс: https://build.opensuse.org/package/show/home:YOUR_USERNAME:rustconn/rustconn
- CLI: `osc results home:YOUR_USERNAME:rustconn rustconn`

### 5. AppImage

AppImage збирається окремо через GitHub Actions або локально:

```bash
# Встановіть appimage-builder
pip install appimage-builder

# Зберіть AppImage
appimage-builder --recipe AppImageBuilder.yml
```

## Оновлення версії

1. Оновіть версію в `Cargo.toml`
2. Додайте запис в `rustconn.changes` та `debian.changelog`
3. Створіть git tag: `git tag v0.2.0`
4. Push: `git push --tags`
5. OBS автоматично підхопить нову версію через `_service`

## Корисні команди osc

```bash
# Перегляд логів збірки
osc buildlog openSUSE_Tumbleweed x86_64

# Локальна збірка для тестування
osc build openSUSE_Tumbleweed x86_64

# Перегляд статусу всіх репозиторіїв
osc results

# Перезапуск збірки
osc rebuild
```

## Публікація

Після успішної збірки пакети доступні для встановлення:

```bash
# openSUSE
sudo zypper addrepo https://download.opensuse.org/repositories/home:YOUR_USERNAME:rustconn/openSUSE_Tumbleweed/home:YOUR_USERNAME:rustconn.repo
sudo zypper refresh
sudo zypper install rustconn

# Fedora
sudo dnf config-manager --add-repo https://download.opensuse.org/repositories/home:YOUR_USERNAME:rustconn/Fedora_40/home:YOUR_USERNAME:rustconn.repo
sudo dnf install rustconn

# Ubuntu/Debian
echo "deb https://download.opensuse.org/repositories/home:YOUR_USERNAME:rustconn/xUbuntu_24.04/ /" | sudo tee /etc/apt/sources.list.d/rustconn.list
curl -fsSL https://download.opensuse.org/repositories/home:YOUR_USERNAME:rustconn/xUbuntu_24.04/Release.key | sudo apt-key add -
sudo apt update
sudo apt install rustconn
```
