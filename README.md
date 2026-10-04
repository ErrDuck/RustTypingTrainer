<div align="center">

# 🦀 RustTypingTrainer

**Терминальный аналог MonkeyType с генерацией текста через LLM**<br>
**A terminal MonkeyType alternative with LLM-powered text generation**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)](#)
[![TUI](https://img.shields.io/badge/UI-ratatui-green)](https://github.com/ratatui/ratatui)
[![Author](https://img.shields.io/badge/Author-ErrDuck-purple)](https://github.com/ErrDuck)
[![AI](https://img.shields.io/badge/Co--Author-Qwen%20(Alibaba)-blue)](https://github.com/QwenLM)

**[🇷🇺 Русский](#-русский)** • **[🇬🇧 English](#-english)**

</div>

---

<!-- ==================== РУССКАЯ ЧАСТЬ ==================== -->

# 🇷🇺 Русский

## 📖 Описание

**RustTypingTrainer** — быстрый и настраиваемый терминальный тренажёр слепой печати, вдохновлённый [MonkeyType](https://monkeytype.com). Написан на Rust с использованием [ratatui](https://github.com/ratatui/ratatui).

Главная фишка — **генерация текста через LLM по вашему промту**: вместо случайных слов вы можете печатать уникальный текст, сгенерированный нейросетью (OpenAI, LM Studio, Ollama, Anthropic или любой OpenAI-совместимый API).

## ✨ Возможности

- 🎯 **Режимы тренировки**
  - `time` — печать на время (15 / 30 / 60 / любые секунды)
  - `words` — печать фиксированного количества слов
- 🌍 **Два языка**
  - русский и английский интерфейс
  - русские и английские словари случайных слов
  - язык генерации текста через LLM (`ru` / `en` / `auto`)
- 🤖 **Генерация текста через LLM**
  - свой промт для ИИ прямо в настройках
  - поддержка OpenAI, LM Studio, Ollama, Anthropic (Claude)
  - автоопределение типа API по адресу (`auto`)
  - автоматическая подстановка `/v1` в адрес API
- ⚙️ **Настройки внутри приложения**
  - меню настроек по клавише `F2`
  - язык, режим, время, промт, модель, API base, API-ключ и т.д.
  - автосохранение в `settings.toml`
- 📊 **Статистика как в MonkeyType**
  - WPM, Raw WPM, точность (%), время
  - верные / введённые символы, ошибки ввода
- 🎨 **TUI-интерфейс**
  - подсветка верных (зелёный) и ошибочных (красный) символов
  - курсор на текущей позиции
  - прогресс по словам
- 🔒 **Блокировка результата на 2 секунды** — защита от случайного скипа результата по инерции
- 📝 **Свои словари** — файл со своим списком слов (`--words-file`)
- 🖥️ **Кроссплатформенность** — Linux, macOS, Windows

## 🚀 Установка

### Требования

- Rust 1.70+ и Cargo ([rustup.rs](https://rustup.rs))

### Сборка из исходников

```bash
git clone https://github.com/ErrDuck/RustTypingTrainer.git
cd RustTypingTrainer
cargo build --release

# запуск
./target/release/RustTypingTrainer        # Linux / macOS
target\release\RustTypingTrainer.exe      # Windows
```

### Быстрый запуск без установки

```bash
cargo run
```

## 📦 Использование

```bash
# тест на 30 секунд (по умолчанию)
cargo run

# тест на 60 секунд
cargo run -- --mode time --amount 60

# тест из 50 слов
cargo run -- --mode words --amount 50

# английский интерфейс
cargo run -- --lang en
```

### Генерация текста через LLM

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."
cargo run -- --llm --prompt "Короткая история про кота-программиста"

# LM Studio (локальный сервер, порт 1234)
cargo run -- --llm \
  --prompt "Текст про космос без знаков препинания" \
  --api-provider lmstudio \
  --api-base http://127.0.0.1:1234 \
  --model google/gemma-4-e4b

# Ollama
cargo run -- --llm \
  --prompt "Рассказ про робота" \
  --api-provider ollama \
  --api-base http://127.0.0.1:11434 \
  --model llama3.1

# Anthropic (Claude)
export ANTHROPIC_API_KEY="sk-ant-..."
cargo run -- --llm \
  --prompt "Dark fantasy story" \
  --api-provider anthropic \
  --api-base https://api.anthropic.com \
  --model claude-3-5-sonnet-20241022 \
  --text-lang en
```

> 💡 Всё то же самое можно настроить **внутри приложения** через `F2` → «Генерация через LLM: да», «Промт для ИИ», «Тип API», «API base», «API ключ».

### Свой список слов

```bash
cargo run -- --words-file words.txt
```

## ⚙️ Настройки (`settings.toml`)

Файл создаётся автоматически при первом запуске. Пример:

```toml
language = "ru"                 # язык интерфейса: "ru" | "en"
words_language = "auto"         # язык случайных слов: "auto" | "ru" | "en"
mode = "time"                   # режим: "time" | "words"
amount = 30                     # секунды (time) или количество слов (words)
use_llm = false                 # генерация текста через LLM
llm_words = 120                 # сколько слов просить у LLM в режиме time
random_words = 300              # сколько случайных слов генерировать
regenerate_on_restart = false   # регенерировать текст при перезапуске (Tab)

[llm]
prompt = ""                     # промт для ИИ (пусто = промт по умолчанию)
text_language = "auto"          # язык генерации: "auto" | "ru" | "en"
model = "gpt-4o-mini"           # модель
api_provider = "auto"           # auto | openai | lmstudio | ollama | anthropic
api_base = "https://api.openai.com/v1"
api_key = ""                    # ключ (лучше держать в переменной окружения!)
api_key_env = "OPENAI_API_KEY"  # переменная окружения с ключом
```

### Меню настроек внутри приложения

Нажмите **F2**:

| Клавиша | Действие |
|---|---|
| `↑` / `↓` | выбор пункта |
| `←` / `→` | изменить значение |
| `Enter` | переключить / редактировать поле |
| `Space` | переключить да/нет |
| `Esc` | отмена редактирования / выход из настроек |
| `F2` | вернуться к тесту |

Пункт **«Сохранить»** записывает настройки в `settings.toml` и сразу применяет их.

## ⌨️ Горячие клавиши

### Тест

| Клавиша | Действие |
|---|---|
| любой символ | начать тест / печатать |
| `Space` | следующее слово |
| `Backspace` | удалить символ / вернуться к прошлому слову |
| `Tab` | сброс теста |
| `Ctrl+R` | новый текст (регенирация) |
| `F2` | настройки |
| `Esc` | выход |
| `Ctrl+C` | выход |

### Результат

> 🔒 Первые **2 секунды** после финиша все клавиши заблокированы — результат нельзя случайно скипнуть.

| Клавиша | Действие |
|---|---|
| `Tab` / `Enter` | начать заново |
| `N` | новый текст |
| `F2` | настройки |
| `Q` / `Esc` | выход |

## 🤖 Поддерживаемые API

| Провайдер | `api_base` | Ключ обязателен? |
|---|---|---|
| `openai` | `https://api.openai.com/v1` | да |
| `lmstudio` | `http://127.0.0.1:1234` | нет |
| `ollama` | `http://127.0.0.1:11434` | нет |
| `anthropic` | `https://api.anthropic.com` | да |
| `auto` | определяется по адресу (порт 1234 → LM Studio, 11434 → Ollama и т.д.) | — |

Программа сама добавляет `/v1` к адресу, если его нет, и использует:
- OpenAI-совместимый формат `POST {base}/v1/chat/completions` для openai / lmstudio / ollama;
- формат `POST {base}/v1/messages` с заголовками `x-api-key` и `anthropic-version` для Anthropic.

## 🛠️ CLI-опции

```
--config <PATH>             путь к файлу настроек [по умолчанию: settings.toml]
--lang <ru|en>              язык интерфейса
--words-lang <auto|ru|en>   язык случайных слов
--mode <time|words>         режим
--amount <N>                секунды или количество слов
--llm-words <N>             сколько слов просить у LLM (режим time)
--random-words <N>          сколько случайных слов генерировать (режим time)
--words-file <PATH>         файл со своим списком слов
--prompt <TEXT>             промт для LLM (включает LLM-генерацию)
--llm                       принудительно включить LLM-генерацию
--random                    принудительно случайные слова
--model <MODEL>             модель LLM
--api-base <URL>            адрес API
--api-provider <P>          auto | openai | lmstudio | ollama | anthropic
--api-key <KEY>             API-ключ
--api-key-env <VAR>         переменная окружения с ключом
--text-lang <auto|ru|en>    язык генерации текста
--regenerate-on-restart     регенерировать текст при перезапуске (Tab)
-h, --help                  помощь
-V, --version               версия
```

## 📊 Как считается статистика

- **WPM** = (верные символы / 5) / минуты
- **Raw WPM** = (все нажатия / 5) / минуты
- **Точность** = верные нажатия / все нажатия × 100%

## 🤝 Вклад

Баги, идеи и pull request приветствуются!

1. Fork
2. `git checkout -b feature/my-feature`
3. `git commit -m "Add my feature"`
4. `git push origin feature/my-feature`
5. Pull Request

## 🤖 Создано с помощью ИИ / Credits

Проект полностью сгенерирован и спроектирован с помощью **Qwen** (Alibaba Cloud).
- **Идея и Промпт-инжиниринг:** [ErrDuck](https://github.com/ErrDuck)
- **Генерация кода:** [Qwen](https://github.com/QwenLM) — AI-ассистент от Alibaba Cloud
- **Архитектура и отладка:** Совместная работа ErrDuck и Qwen

## 📄 Лицензия

MIT — см. [LICENSE](LICENSE).

---

<!-- ==================== ENGLISH PART ==================== -->

# 🇬🇧 English

## 📖 Description

**RustTypingTrainer** is a fast, customizable terminal typing trainer inspired by [MonkeyType](https://monkeytype.com), written in Rust with [ratatui](https://github.com/ratatui/ratatui).

Its killer feature is **LLM-powered text generation from your prompt**: instead of random words you can type unique text generated by a neural network (OpenAI, LM Studio, Ollama, Anthropic, or any OpenAI-compatible API).

## ✨ Features

- 🎯 **Training modes**
  - `time` — type for a duration (15 / 30 / 60 / any seconds)
  - `words` — type a fixed number of words
- 🌍 **Bilingual**
  - Russian and English UI
  - Russian and English random word dictionaries
  - LLM output language (`ru` / `en` / `auto`)
- 🤖 **LLM text generation**
  - custom AI prompt right in the settings
  - OpenAI, LM Studio, Ollama, Anthropic (Claude) support
  - automatic API type detection by URL (`auto`)
  - automatic `/v1` suffix handling
- ⚙️ **In-app settings**
  - settings menu on `F2`
  - language, mode, time, prompt, model, API base, API key, etc.
  - auto-saved to `settings.toml`
- 📊 **MonkeyType-style stats**
  - WPM, Raw WPM, accuracy (%), time
  - correct / typed characters, input errors
- 🎨 **TUI interface**
  - green/red highlighting of correct/incorrect characters
  - live caret on the current position
  - word progress counter
- 🔒 **2-second result lock** — protects the results screen from accidental skip-by-inertia keystrokes
- 📝 **Custom dictionaries** — your own word list file (`--words-file`)
- 🖥️ **Cross-platform** — Linux, macOS, Windows

## 🚀 Installation

### Requirements

- Rust 1.70+ and Cargo ([rustup.rs](https://rustup.rs))

### Build from source

```bash
git clone https://github.com/ErrDuck/RustTypingTrainer.git
cd RustTypingTrainer
cargo build --release

# run
./target/release/RustTypingTrainer        # Linux / macOS
target\release\RustTypingTrainer.exe      # Windows
```

### Quick run without installing

```bash
cargo run
```

## 📦 Usage

```bash
# 30-second test (default)
cargo run

# 60-second test
cargo run -- --mode time --amount 60

# 50-word test
cargo run -- --mode words --amount 50

# English UI
cargo run -- --lang en
```

### LLM text generation

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."
cargo run -- --llm --prompt "A short story about a cat who codes"

# LM Studio (local server, port 1234)
cargo run -- --llm \
  --prompt "Text about space without punctuation" \
  --api-provider lmstudio \
  --api-base http://127.0.0.1:1234 \
  --model google/gemma-4-e4b

# Ollama
cargo run -- --llm \
  --prompt "A story about a robot" \
  --api-provider ollama \
  --api-base http://127.0.0.1:11434 \
  --model llama3.1

# Anthropic (Claude)
export ANTHROPIC_API_KEY="sk-ant-..."
cargo run -- --llm \
  --prompt "Dark fantasy story" \
  --api-provider anthropic \
  --api-base https://api.anthropic.com \
  --model claude-3-5-sonnet-20241022 \
  --text-lang en
```

> 💡 Everything above can also be configured **inside the app** via `F2` → "Use LLM: yes", "AI prompt", "API provider", "API base", "API key".

### Custom word list

```bash
cargo run -- --words-file words.txt
```

## ⚙️ Configuration (`settings.toml`)

The file is created automatically on first run. Example:

```toml
language = "en"                 # UI language: "ru" | "en"
words_language = "auto"         # random words language: "auto" | "ru" | "en"
mode = "time"                   # mode: "time" | "words"
amount = 30                     # seconds (time) or word count (words)
use_llm = false                 # generate text via LLM
llm_words = 120                 # words to request from LLM in time mode
random_words = 300              # random words to generate in time mode
regenerate_on_restart = false   # regenerate text on restart (Tab)

[llm]
prompt = ""                     # AI prompt (empty = default prompt)
text_language = "auto"          # generation language: "auto" | "ru" | "en"
model = "gpt-4o-mini"           # model
api_provider = "auto"           # auto | openai | lmstudio | ollama | anthropic
api_base = "https://api.openai.com/v1"
api_key = ""                    # key (better keep it in an env variable!)
api_key_env = "OPENAI_API_KEY"  # env variable holding the key
```

### In-app settings menu

Press **F2**:

| Key | Action |
|---|---|
| `↑` / `↓` | select item |
| `←` / `→` | change value |
| `Enter` | toggle / edit field |
| `Space` | toggle yes/no |
| `Esc` | cancel editing / leave settings |
| `F2` | back to test |

The **"Save"** item writes settings to `settings.toml` and applies them immediately.

## ⌨️ Hotkeys

### Test

| Key | Action |
|---|---|
| any character | start test / type |
| `Space` | next word |
| `Backspace` | delete char / go back one word |
| `Tab` | reset test |
| `Ctrl+R` | new text (regenerate) |
| `F2` | settings |
| `Esc` | quit |
| `Ctrl+C` | quit |

### Results

> 🔒 For the first **2 seconds** after finishing, all keys are locked — the result screen cannot be skipped by inertia.

| Key | Action |
|---|---|
| `Tab` / `Enter` | restart |
| `N` | new text |
| `F2` | settings |
| `Q` / `Esc` | quit |

## 🤖 Supported APIs

| Provider | `api_base` | Key required? |
|---|---|---|
| `openai` | `https://api.openai.com/v1` | yes |
| `lmstudio` | `http://127.0.0.1:1234` | no |
| `ollama` | `http://127.0.0.1:11434` | no |
| `anthropic` | `https://api.anthropic.com` | yes |
| `auto` | detected from URL (port 1234 → LM Studio, 11434 → Ollama, etc.) | — |

The app appends `/v1` to the base URL when missing and uses:
- the OpenAI-compatible `POST {base}/v1/chat/completions` format for openai / lmstudio / ollama;
- the `POST {base}/v1/messages` format with `x-api-key` and `anthropic-version` headers for Anthropic.

## 🛠️ CLI options

```
--config <PATH>             path to settings file [default: settings.toml]
--lang <ru|en>              UI language
--words-lang <auto|ru|en>   random words language
--mode <time|words>         mode
--amount <N>                seconds or word count
--llm-words <N>             words to request from LLM (time mode)
--random-words <N>          random words to generate (time mode)
--words-file <PATH>         custom word list file
--prompt <TEXT>             LLM prompt (enables LLM generation)
--llm                       force LLM generation
--random                    force random words
--model <MODEL>             LLM model
--api-base <URL>            API URL
--api-provider <P>          auto | openai | lmstudio | ollama | anthropic
--api-key <KEY>             API key
--api-key-env <VAR>         env variable holding the key
--text-lang <auto|ru|en>    generated text language
--regenerate-on-restart     regenerate text on restart (Tab)
-h, --help                  help
-V, --version               version
```

## 📊 How stats are calculated

- **WPM** = (correct chars / 5) / minutes
- **Raw WPM** = (all keystrokes / 5) / minutes
- **Accuracy** = correct keystrokes / all keystrokes × 100%

## 🤝 Contributing

Bug reports, ideas and pull requests are welcome!

1. Fork
2. `git checkout -b feature/my-feature`
3. `git commit -m "Add my feature"`
4. `git push origin feature/my-feature`
5. Pull Request

## 🤖 Credits & AI Acknowledgement

This project was fully generated and architected using **Qwen** (Alibaba Cloud).
- **Concept & Prompt Engineering:** [ErrDuck](https://github.com/ErrDuck)
- **Code Generation:** [Qwen](https://github.com/QwenLM) — AI assistant by Alibaba Cloud
- **Architecture & Debugging:** Collaborative effort between ErrDuck and Qwen

## 📄 License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

**Made with ❤️ and 🦀 by [ErrDuck](https://github.com/ErrDuck) & [Qwen](https://github.com/QwenLM)**

</div>
