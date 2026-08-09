# 🎙️ Voice Assistant WebSocket Server

Голосовой ассистент с поддержкой WebSocket, локальным распознаванием речи (Whisper), синтезом речи (eSpeak-NG) и AI на базе Ollama.

---

## 📋 Оглавление
- [Возможности](#-возможности)
- [Требования](#-требования)
- [Установка](#-установка)
- [Настройка](#-настройка)
- [Запуск](#-запуск)
- [Использование](#-использование)
- [Архитектура](#-архитектура)
- [Устранение проблем](#-устранение-проблем)

---

## ✨ Возможности

- 🎤 **Голосовой ввод** — запись с микрофона по нажатию клавиши `+`
- 🧠 **Распознавание речи** — локальный Whisper STT (поддержка русского языка)
- 🤖 **AI обработка** — интеграция с Ollama (локальные LLM)
- 🔊 **Синтез речи** — eSpeak-NG (локальный, без интернета, русский голос)
- 🌐 **WebSocket** — двусторонняя связь в реальном времени
- 💻 **Кроссплатформенность** — работает на Linux (Arch, Ubuntu, Debian)

---

## 📦 Требования

### Системные
- **ОС**: Arch Linux / Ubuntu 20.04+ / Debian 11+
- **Процессор**: x86_64 с поддержкой AVX2
- **ОЗУ**: минимум 4 ГБ (рекомендуется 8+)
- **Место на диске**: ~3-5 ГБ

### Программное обеспечение
- [Rust](https://rustup.rs/) (1.70+)
- [Ollama](https://ollama.com/)
- [eSpeak-NG](https://github.com/espeak-ng/espeak-ng)
- [Git](https://git-scm.com/)

---

## 🚀 Установка

### 1. Клонирование репозитория
```bash
git clone https://github.com/your-username/voice-assistant-websocket.git
cd voice-assistant-websocket
```

### 2. Установка Ollama и модели
```bash
# Установка Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Скачивание модели
ollama pull llama2:7b
# или
ollama pull mistral:7b
```

### 3. Установка eSpeak-NG
```bash
# Arch Linux
sudo pacman -S espeak-ng

# Ubuntu/Debian
sudo apt update
sudo apt install espeak-ng
```

### 4. Установка Whisper модели
```bash
# Создание папки для моделей
mkdir -p models

# Скачивание мультиязычной модели Whisper (поддерживает русский)
wget -O models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# Проверка
ls -lh models/ggml-base.bin  # ~142 MB
```

### 5. Настройка проекта
```bash
# Копирование файла конфигурации
cp .env.example .env

# Редактирование .env
nano .env
```

### 6. Сборка проекта
```bash
cargo build --release
```

---

## ⚙️ Настройка

### Конфигурация `.env`

| Переменная | Описание | Значение по умолчанию |
|------------|----------|----------------------|
| `SERVER_HOST` | Хост для WebSocket сервера | `127.0.0.1` |
| `SERVER_PORT` | Порт для WebSocket сервера | `8080` |
| `WHISPER_MODEL_PATH` | Путь к модели Whisper | `./models/ggml-base.bin` |
| `OLLAMA_URL` | URL Ollama API | `http://localhost:11434` |
| `OLLAMA_MODEL` | Модель Ollama | `llama2` |
| `OLLAMA_TIMEOUT` | Таймаут для Ollama (сек) | `30` |
| `AUDIO_SAMPLE_RATE` | Частота дискретизации (Гц) | `16000` |
| `AUDIO_CHANNELS` | Количество каналов | `1` |
| `RECORD_DURATION_SECONDS` | Длительность записи (сек) | `5` |

### Пример `.env`
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
WHISPER_MODEL_PATH=./models/ggml-base.bin
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=llama2
OLLAMA_TIMEOUT=30
AUDIO_SAMPLE_RATE=16000
AUDIO_CHANNELS=1
RECORD_DURATION_SECONDS=5
```

---

## 🎯 Запуск

### Терминал 1 — Ollama:
```bash
ollama serve
```

### Терминал 2 — Сервер:
```bash
cargo run
```

### Терминал 3 — Клиент (для тестирования):
```bash
cargo run --example websocket_client
```

### Запуск в фоновом режиме (Linux/macOS):
```bash
nohup ollama serve > ollama.log 2>&1 &
nohup cargo run --release > server.log 2>&1 &
tail -f server.log
```

---

## 💬 Использование

### Команды клиента

| Команда | Описание |
|---------|----------|
| `+` | Активация голосового ввода (запись 5 секунд) |
| `текст` | Отправка текстового сообщения AI |
| `exit` / `quit` | Выход из клиента |

### Пример сессии
```
🔄 Connecting to WebSocket server...
✅ Connected!
💡 Press '+' and speak to ask a question

> +
📨 Server: 🎙️ Listening...
📨 Server: 🧠 Transcribing...
📨 Server: 📝 You said: привет как дела
📨 Server: 🤔 Thinking...
📨 Server: 🤖 Привет! У меня всё отлично, спасибо. Чем могу помочь?
📨 Server: 🔊 Speaking...
🎵 Received audio: 609160 bytes
📨 Server: ✅ Done! Press '+' to ask another question
```

### Тестирование через браузер
```javascript
const ws = new WebSocket('ws://localhost:8080');

ws.onmessage = (event) => {
    if (typeof event.data === 'string') {
        console.log('📨', event.data);
    } else {
        console.log('🎵 Audio received:', event.data.size, 'bytes');
        const audio = new Audio(URL.createObjectURL(event.data));
        audio.play();
    }
};

ws.onopen = () => ws.send('+');
```

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────────┐
│                        Клиенты                              │
├─────────────┬───────────────────┬──────────────────────────┤
│   Rust CLI  │   Web Browser    │   Мобильное приложение   │
└─────────────┴───────────────────┴──────────────────────────┘
                            │
                            ▼
                  ┌─────────────────┐
                  │   WebSocket     │
                  │   (port 8080)   │
                  └─────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                  Voice Assistant Server                    │
├──────────────┬──────────────┬──────────────┬──────────────┤
│   Recorder   │   Whisper    │   Ollama     │   eSpeak-NG  │
│   (cpal)     │   (STT)      │   (AI)       │   (TTS)      │
└──────────────┴──────────────┴──────────────┴──────────────┘
```

### Компоненты
1. **WebSocket Server** — обработка клиентских соединений
2. **Audio Recorder** — запись с микрофона (cpal)
3. **Whisper STT** — распознавание речи
4. **Ollama AI** — генерация ответов
5. **eSpeak-NG TTS** — синтез речи

---

## 🐛 Устранение проблем

### Ошибка: "Failed to load Whisper model"
```bash
# Проверка наличия модели
ls -lh models/ggml-base.bin

# Скачивание модели
wget -O models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

### Ошибка: "No input device found"
```bash
# Arch Linux
arecord -l
sudo pacman -S alsa-utils

# Ubuntu/Debian
sudo apt install alsa-utils
```

### Ошибка: "Connection refused"
```bash
# Проверка запущен ли сервер
netstat -an | grep 8080

# Проверка firewall (Linux)
sudo ufw allow 8080
```

### Ошибка: "Ollama timeout"
```bash
# Увеличение таймаута в .env
OLLAMA_TIMEOUT=60

# Проверка Ollama
ollama list
ollama ps
```

### Ошибка: "espeak-ng: command not found"
```bash
# Arch Linux
sudo pacman -S espeak-ng

# Ubuntu/Debian
sudo apt install espeak-ng
```

### Ошибка: первая запись не работает
Проблема с инициализацией аудио устройства. Повторная попытка обычно работает. Код автоматически делает до 3 попыток записи.

---

## 📁 Структура проекта

```
voice-assistant-websocket/
├── src/
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── recorder.rs
│   │   ├── stt.rs
│   │   └── tts.rs
│   ├── ai/
│   │   ├── mod.rs
│   │   └── ollama.rs
│   ├── server/
│   │   ├── mod.rs
│   │   └── websocket.rs
│   ├── utils/
│   │   ├── mod.rs
│   │   └── audio_utils.rs
│   ├── config.rs
│   ├── main.rs
│   └── lib.rs
├── models/
│   └── ggml-base.bin
├── examples/
│   └── websocket_client.rs
├── .env
├── Cargo.toml
└── README.md
```

---

## 📝 Лицензия

MIT License
