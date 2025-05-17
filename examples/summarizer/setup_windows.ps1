# Check if running on Windows
if (-not $IsWindows -and -not $env:OS -like "*Windows*") {
  Write-Host "❌ Error: This application is designed for Windows only."
  exit 1
}

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Host "🛠️ Installing Rust..."
  try {
    iex "& { $(irm https://sh.rustup.rs) } -y"
    if (-not $?) {
      Write-Host "❌ Failed to install Rust. Please install Rust manually from https://rustup.rs/"
      exit 1
    }
  } catch {
    Write-Host "❌ Failed to install Rust: $_"
    Write-Host "Please install Rust manually from https://rustup.rs/"
    exit 1
  }
  # Refresh environment variables
  $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
}

# Check Ollama
$ollamaPath = "$env:LOCALAPPDATA\Programs\Ollama\ollama.exe"
if (-not (Test-Path $ollamaPath)) {
  Write-Host "🌐 Ollama not found. Opening download page..."
  Start-Process "https://ollama.com/download"
  Write-Host "⚠️ Please install Ollama manually, then rerun this script."
  exit 1
}

# Check if Ollama service is running
try {
  $ollamaRunning = Get-Process "ollama" -ErrorAction SilentlyContinue
  if (-not $ollamaRunning) {
    Write-Host "⚠️ Ollama is installed but not running. Starting Ollama..."
    Start-Process $ollamaPath
    Write-Host "⏳ Waiting for Ollama to start..."
    Start-Sleep -Seconds 5
  }
} catch {
  Write-Host "⚠️ Couldn't check if Ollama is running: $_"
}

# Pull the Gemma model
Write-Host "📥 Pulling Gemma3 model..."
try {
  & $ollamaPath pull gemma3
  if (-not $?) {
    Write-Host "❌ Failed to pull Gemma3 model. Please run 'ollama pull gemma3' manually."
    exit 1
  }
} catch {
  Write-Host "❌ Failed to pull Gemma3 model: $_"
  Write-Host "Please run 'ollama pull gemma3' manually."
  exit 1
}

# Build the Rust binary
Write-Host "🔨 Building Rust project..."
try {
  cd ../..  # go from examples/summarizer to project root
  cargo build --release -p summarizer
  if (-not $?) {
    Write-Host "❌ Failed to build the project."
    exit 1
  }
} catch {
  Write-Host "❌ Failed to build the project: $_"
  exit 1
}

Write-Host "`n✅ Setup complete!"
Write-Host "🚀 Run with: .\\target\\release\\summarizer.exe`n"
Write-Host "📝 Usage:"
Write-Host "1. Start the application"
Write-Host "2. Press Ctrl+J while focusing on any window to capture its UI"
Write-Host "3. The AI-generated summary will be copied to your clipboard"
Write-Host "4. Press Ctrl+C in the terminal window to exit" 