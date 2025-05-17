# UI Context Summarizer

This example demonstrates how to use Terminator with Ollama to:
1. 🖥️ Capture the UI Automation tree from the active window
2. 🤖 Send it to a local LLM (Gemma 3 via Ollama) for summarization
3. 📋 Copy the summarized content to your clipboard

## Features

- Global hotkey (`Ctrl+J`) to trigger capturing the UI context
- Full UI Automation tree capturing using Terminator
- Local processing with Ollama (no data leaves your computer)
- Smart summarization of complex UI structures
- Clipboard integration for easy pasting into any application
- Comprehensive error handling and graceful shutdown
- Platform compatibility validation

## Requirements

- Windows 10/11 (UI Automation features are Windows-specific)
- [Rust](https://www.rust-lang.org/tools/install)
- [Ollama](https://ollama.com/download) with the Gemma3 model

## Quick Start

```bash
# Clone the repository
git clone https://github.com/mediar-ai/terminator
cd terminator/examples/summarizer

# Run the setup script (installs dependencies & builds the app)
powershell -ExecutionPolicy Bypass -File setup_windows.ps1

# Run the application
.\target\release\summarizer.exe
```

## Usage

1. Start the application
2. Press `Ctrl+J` while focused on any window
3. Wait briefly for the AI summary to be generated
4. The summary will be automatically copied to your clipboard
5. Paste it wherever you need (e.g., ChatGPT, Slack, email)
6. To exit the application, press `Ctrl+C` in the terminal window

## Error Handling

The application includes comprehensive error checks:
- Windows platform verification
- Ollama installation and running status
- Gemma3 model availability
- UI Automation accessibility
- Graceful shutdown with proper resource cleanup

## How It Works

The application uses Terminator's UI Automation capabilities to extract the full UI tree from the focused window, including accessibility information. This rich context is processed through these steps:

1. **Hotkey Detection**: Listens for Ctrl+J using global-hotkey
2. **UI Capture**: Uses Terminator to get the active window's UI Automation tree
3. **AI Processing**: Sends the UI tree to Ollama's Gemma3 model with a structured prompt
4. **Result Handling**: Copies the AI-generated summary to clipboard for easy use

## Troubleshooting

- **"Ollama is not running"**: Start the Ollama application
- **"Gemma3 model not found"**: Run `ollama pull gemma3` to download the model
- **"Failed to get current window"**: Make sure you have a window in focus
- **Clipboard issues**: Ensure no other applications are blocking clipboard access 