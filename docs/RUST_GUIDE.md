# Rust-гид для переноса `arithmet`

Этот документ не заменяет официальную книгу Rust, а показывает, какие темы
учить по мере переноса C-программы на Rust.

Официальная книга:
<https://doc.rust-lang.org/book/>

Стандартная библиотека:
<https://doc.rust-lang.org/std/>

Cargo:
<https://doc.rust-lang.org/cargo/>

## 1. Как думать о переносе с C

В C-версии много глобального состояния: `app_settings`, `player_name`,
`session_result`, состояние терминала, мыши и текущего экрана. В Rust лучше
перевести это в явные структуры:

- `Settings` - настройки пользователя;
- `App` - состояние интерфейса;
- `Session` - текущий сеанс;
- `SessionResult` - завершенный результат;
- `ProtocolStore` - работа с файлами результатов.

Так компилятор будет помогать видеть, кто владеет данными и кто может их
изменять.

## 2. С чего начать в Rust

Для этого проекта особенно важны главы The Rust Book:

- глава 3: переменные, типы, функции, управление потоком;
- глава 4: ownership, borrowing, slices;
- глава 5: структуры;
- глава 6: enum и `match`;
- глава 7: модули;
- глава 8: `Vec`, `String`, `HashMap`;
- глава 9: ошибки через `Result`;
- глава 10: generics и traits, когда появится необходимость;
- глава 11: тесты;
- глава 12: CLI-проект;
- глава 13: замыкания и итераторы, полезно позже;
- глава 18: pattern matching, полезно для событий TUI.

Не нужно читать все идеально перед кодом. Лучше брать тему тогда, когда она
появляется в задаче.

## 3. Cargo-команды

Основные команды:

```bash
cargo check
cargo test
cargo run
cargo fmt
cargo clippy
```

Как пользоваться:

- `cargo check` - быстро проверить, компилируется ли код;
- `cargo test` - запуск тестов;
- `cargo run` - запуск программы;
- `cargo fmt` - форматирование;
- `cargo clippy` - подсказки по качеству кода.

## 4. Модули

В Rust модуль обычно создается через файл или каталог.

Пример:

```text
src/domain/mod.rs
src/domain/operation.rs
src/domain/settings.rs
```

В `src/domain/mod.rs`:

```rust
pub mod operation;
pub mod settings;
```

В `src/main.rs`:

```rust
mod domain;
```

Если тип нужен снаружи модуля, он должен быть `pub`.

## 5. Ошибки

Для приложения лучше возвращать `Result`, а не делать `panic!`.

Пример:

```rust
fn main() -> anyhow::Result<()> {
    arithmet::run()
}
```

На раннем этапе можно использовать `anyhow`, потому что она удобна для
прикладного кода. Когда появится стабильная архитектура, часть ошибок можно
описать через `thiserror`.

## 6. Строки и русские символы

В C-версии уже была отдельная логика для UTF-8. В Rust важно помнить:

- `String` хранит UTF-8;
- индексировать строку как `text[0]` нельзя;
- `.len()` возвращает байты, а не количество символов;
- `.chars().count()` возвращает количество Unicode scalar values.

Для экранной ширины в терминале русские буквы обычно занимают одну ячейку, но
для полного учета Unicode можно использовать crate `unicode-width`.

## 7. Enum вместо числовых кодов

В C операции хранятся индексами `0..4`. В Rust лучше:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    DivisionWithRemainder,
}
```

Затем использовать `match`:

```rust
match operation {
    Operation::Addition => "+",
    Operation::Subtraction => "-",
    Operation::Multiplication => "*",
    Operation::Division => "/",
    Operation::DivisionWithRemainder => ":",
}
```

Так меньше риска перепутать `2` и `3`.

## 8. Тесты

Тесты в Rust можно писать рядом с кодом:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grade_for_98_percent_is_five() {
        assert_eq!(grade(98, 100), 5);
    }
}
```

Для этого проекта сначала нужны тесты на:

- валидацию настроек;
- расчет оценки;
- генерацию каждого типа примера;
- транслитерацию имени файла;
- выбор следующего файла результата;
- чтение старых протоколов.

## 9. Работа с файлами

Полезные типы стандартной библиотеки:

- `std::path::Path`;
- `std::path::PathBuf`;
- `std::fs`;
- `std::io`.

Пример чтения файла:

```rust
let text = std::fs::read_to_string(path)?;
```

Пример записи:

```rust
std::fs::write(path, text)?;
```

Для путей не склеивать строки руками. Использовать:

```rust
let path = results_dir.join(filename);
```

## 10. Конфиг

Для TOML-конфига нужны `serde` и `toml`.

Пример модели:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub player_name: String,
    pub results_dir: PathBuf,
    pub operations: OperationsConfig,
    pub limits: LimitsConfig,
}
```

После чтения конфига обязательно делать валидацию. Не стоит считать, что файл
всегда корректный: пользователь может изменить его руками.

## 11. TUI

Для интерфейса лучше смотреть:

- `ratatui`: <https://ratatui.rs/>
- `crossterm`: <https://docs.rs/crossterm/>

Что изучить:

- terminal raw mode;
- alternate screen;
- event loop;
- key events;
- resize events;
- layout constraints в `ratatui`;
- widgets: `Block`, `Paragraph`, `List`, `Gauge`, `Tabs`.

Главная идея: каждый кадр рисуется из текущего состояния `App`. Не нужно
вручную помнить, где стереть строку, как в C.

## 12. Как не запутаться

Рабочий цикл для каждой части:

1. Описать маленький тип.
2. Написать одну функцию.
3. Написать тест.
4. Запустить `cargo test`.
5. Только потом подключать к интерфейсу.

Хороший первый набор задач:

1. `Operation` и его символы.
2. `Settings` и дефолты.
3. Валидация `Settings`.
4. Функция `grade(correct, planned)`.
5. Генератор сложения.
6. Генератор остальных операций.
7. Запись протокола.
8. Минимальный экран настроек.

## 13. Где смотреть примеры

Официальные и практичные источники:

- Rust Book: <https://doc.rust-lang.org/book/>
- Rust by Example: <https://doc.rust-lang.org/rust-by-example/>
- Standard Library: <https://doc.rust-lang.org/std/>
- Cargo Book: <https://doc.rust-lang.org/cargo/>
- Crate docs на docs.rs: <https://docs.rs/>
- Ratatui examples: <https://github.com/ratatui-org/ratatui/tree/main/examples>

Когда подключается новый crate, первое место для чтения - его страница на
`docs.rs` и примеры в репозитории crate.
