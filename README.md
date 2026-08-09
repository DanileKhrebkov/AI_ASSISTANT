```markdown
# 🎙️ Voice Assistant WebSocket Server

Голосовой ассистент с поддержкой WebSocket, локальным распознаванием речи (Whisper), синтезом речи (Edge TTS) и AI на базе Ollama.

---

## 📋 Оглавление
- [Возможности](#-возможности)
- [Требования](#-требования)
- [Установка](#-установка)
- [Настройка](#-настройка)
- [Запуск](#-запуск)
- [Использование](#-использование)
- [Архитектура](#-архитектура)
- [API](#-api)
- [Примеры](#-примеры)
- [Устранение проблем](#-устранение-проблем)
- [Планы по улучшению](#-планы-по-улучшению)

---

## ✨ Возможности

- 🎤 **Голосовой ввод** - запись с микрофона по нажатию клавиши
- 🧠 **Распознавание речи** - локальный Whisper STT (русский язык)
- 🤖 **AI обработка** - интеграция с Ollama (локальные LLM)
- 🔊 **Синтез речи** - Edge TTS (бесплатный, русский голос)
- 🌐 **WebSocket** - двусторонняя связь в реальном времени
- 💻 **Кроссплатформенность** - работает на Windows, Linux, macOS
- 📦 **Минималистичный дизайн** - все компоненты локальные, без облачных зависимостей

---

## 📦 Требования

### Системные требования
- **ОС**: Windows 10+, Linux (Ubuntu 20.04+), macOS 10.15+
- **Процессор**: x86_64 или ARM (с поддержкой AVX2 для Whisper)
- **ОЗУ**: минимум 4 ГБ (рекомендуется 8+ ГБ для моделей 7B+)
- **Место на диске**: ~3-5 ГБ (модели + зависимости)

### Программное обеспечение
- [Rust](https://rustup.rs/) (1.70+)
- [Ollama](https://ollama.com/) (для AI)
- [Git](https://git-scm.com/) (для скачивания моделей)

---

## 🚀 Установка

### 1. Клонирование репозитория
```bash
git clone https://github.com/your-username/voice-assistant-websocket.git
cd voice-assistant-websocket
```

### 2. Установка Ollama и моделей
```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull llama2:7b
```

### 3. Установка Whisper
```bash
git clone https://github.com/ggerganov/whisper.cpp
cd whisper.cpp
make
./models/download-ggml-model.sh base-ru
mkdir -p ../voice-assistant-websocket/models
cp models/ggml-base-ru.bin ../voice-assistant-websocket/models/
cd ..
```

### 4. Настройка проекта
```bash
cp .env.example .env
```

### 5. Сборка проекта
```bash
cargo build --release
cargo test
```

---

## ⚙️ Настройка

### Конфигурация `.env`

| Переменная | Описание | Значение по умолчанию |
|------------|----------|----------------------|
| `SERVER_HOST` | Хост для WebSocket сервера | `127.0.0.1` |
| `SERVER_PORT` | Порт для WebSocket сервера | `8080` |
| `WHISPER_MODEL_PATH` | Путь к модели Whisper | `./models/ggml-base-ru.bin` |
| `OLLAMA_URL` | URL Ollama API | `http://localhost:11434` |
| `OLLAMA_MODEL` | Модель Ollama | `llama2` |
| `OLLAMA_TIMEOUT` | Таймаут для Ollama (сек) | `30` |
| `TTS_VOICE` | Голос Edge TTS | `ru-RU-DmitryNeural` |
| `TTS_LANGUAGE` | Язык TTS | `ru-RU` |
| `AUDIO_SAMPLE_RATE` | Частота дискретизации (Гц) | `16000` |
| `AUDIO_CHANNELS` | Количество каналов | `1` |
| `RECORD_DURATION_SECONDS` | Длительность записи (сек) | `5` |

---

## 🎯 Запуск

### Способ 1: Раздельный запуск

**Терминал 1 - Ollama:**
```bash
ollama serve
```

**Терминал 2 - Сервер:**
```bash
cargo run
```

**Терминал 3 - Клиент:**
```bash
cargo run --example websocket_client
```

### Способ 2: Запуск в фоновом режиме (Linux/macOS)
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
📨 Server: 📝 You said: привет как дела
📨 Server: 🤔 Thinking...
📨 Server: 🤖 Привет! У меня всё отлично, спасибо.
📨 Server: 🔊 Speaking...
🎵 Received audio: 32768 bytes
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
│   Recorder   │   Whisper    │   Ollama     │   Edge TTS   │
│   (cpal)     │   (STT)      │   (AI)       │   (TTS)      │
└──────────────┴──────────────┴──────────────┴──────────────┘
```

### Компоненты
1. **WebSocket Server** - обработка клиентских соединений
2. **Audio Recorder** - запись с микрофона
3. **Whisper STT** - распознавание речи
4. **Ollama AI** - генерация ответов
5. **Edge TTS** - синтез речи

---

## 📡 API

### WebSocket сообщения

**Клиент → Сервер:**
```
"привет мир"     # Текстовое сообщение
"+"              # Команда записи
"exit"           # Закрытие соединения
```

**Сервер → Клиент:**
```
"📝 You said: привет"     # Текстовое сообщение
Binary(16-bit PCM WAV)    # Аудио данные
"🎙️ Listening..."        # Статус
```

---

## 📝 Примеры

### 1. Веб-клиент на JavaScript
```html
<!DOCTYPE html>
<html>
<head>
    <title>Voice Assistant</title>
</head>
<body>
    <button id="recordBtn">🎤 Говорить</button>
    <div id="messages"></div>
    
    <script>
        const ws = new WebSocket('ws://localhost:8080');
        const messages = document.getElementById('messages');
        const recordBtn = document.getElementById('recordBtn');
        
        ws.onmessage = (event) => {
            if (typeof event.data === 'string') {
                messages.innerHTML += `<p>📨 ${event.data}</p>`;
            } else {
                messages.innerHTML += `<p>🎵 Получено аудио (${event.data.size} байт)</p>`;
            }
        };
        
        recordBtn.onclick = () => {
            ws.send('+');
            messages.innerHTML += '<p>🎙️ Запись началась...</p>';
        };
    </script>
</body>
</html>
```

### 2. Python клиент
```python
import asyncio
import websockets

async def client():
    uri = "ws://localhost:8080"
    async with websockets.connect(uri) as websocket:
        await websocket.send("+")
        async for message in websocket:
            if isinstance(message, str):
                print(f"📨 {message}")
            else:
                print(f"🎵 Audio: {len(message)} bytes")

asyncio.run(client())
```

---

## 🐛 Устранение проблем

### Ошибка: "No input device found"
```bash
# Linux
arecord -l

# macOS - разрешить доступ к микрофону
# Системные настройки → Безопасность → Микрофон

# Windows - проверить в Панели управления → Звук
```

### Ошибка: "Failed to load Whisper model"
```bash
ls -la ./models/ggml-base-ru.bin
cd whisper.cpp
./models/download-ggml-model.sh base-ru
cp models/ggml-base-ru.bin ../voice-assistant-websocket/models/
```

### Ошибка: "Connection refused"
```bash
netstat -an | grep 8080
sudo ufw allow 8080  # Linux
```

### Ошибка: Ollama timeout
```bash
# Увеличить таймаут в .env
OLLAMA_TIMEOUT=60

# Проверить Ollama
ollama list
ollama ps
```
