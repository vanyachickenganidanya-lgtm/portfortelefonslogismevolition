# LogismEvo Phone

Мобильный аналог [Logisim Evolution](https://github.com/logisim-evolution/logisim-evolution) — опенсорсного Java-редактора цифровых схем для ПК.

Здесь схема рисуется пальцем, симуляция идёт сразу, а **поведение элементов задаётся на языке Funo** ([funo-studio](https://github.com/vanyachickenganidanya-lgtm/funo-studio)), а не зашито только в Java.

## Чем лучше десктопного Logisim на телефоне

- Жесты: тап — поставить, перетащить — провод, долгое нажатие — свойства
- Панель снизу (большой палец), не меню как на ПК
- Хронограмма сигналов прямо под холстом
- Элементы — обычные `.fun` файлы, их можно править без пересборки движка
- Bevy рисует схему, Slint — телефонный UI

## Структура

```
funo/components/   элементы на Funo (AND, OR, XOR, NOT, NAND, NOR, XOR, MUX, DFF, CLK, PIN, LED)
crates/sim         симулятор + интерпретатор Funo
crates/app         Bevy 0.15 + Slint overlay
ui/phone.slint     мобильный интерфейс
web/               живой превью для телефона / браузера
```

## Запуск превью

```bash
cd web && python3 -m http.server 4173
```

## Нативная сборка (когда есть Rust + Android NDK)

```bash
cd crates/app
cargo run
# Android:
cargo apk run --target aarch64-linux-android
```
