use std::{
    env,
    process::{Command, exit},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Result, Context, bail, anyhow};
use arboard::Clipboard;
use global_hotkey::{
    hotkey::{HotKey, Modifiers},
    GlobalHotKeyManager,
};
use terminator::{Desktop, Selector};
use tokio::{time::sleep, signal, sync::oneshot};
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;

const HOTKEY_ID: u32 = 1;

/// Check if Ollama is installed and running
fn check_ollama() -> Result<()> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .context("Failed to execute Ollama. Is it installed and in your PATH?")?;
    
    if !output.status.success() {
        bail!("Ollama is not running properly. Please ensure the Ollama service is running.");
    }
    
    Ok(())
}

/// Check if the Gemma3 model is available in Ollama
fn check_model_availability() -> Result<()> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .context("Failed to check model availability")?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    if !output_str.contains("gemma3") {
        bail!("Gemma3 model not found in Ollama. Please run: ollama pull gemma3");
    }
    
    Ok(())
}

/// Check if the current platform is Windows
fn check_platform() -> Result<()> {
    #[cfg(not(target_os = "windows"))]
    {
        bail!("This application currently only supports Windows. UI Automation features are primarily designed for Windows platforms.");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Starting UI Context Summarizer");
    
    // Check platform compatibility
    check_platform().context("Platform compatibility check failed")?;
    
    // Check if Ollama is installed and running
    if let Err(e) = check_ollama() {
        error!("❌ {}", e);
        exit(1);
    }
    
    // Check if the Gemma3 model is available
    if let Err(e) = check_model_availability() {
        error!("❌ {}", e);
        warn!("💡 Run 'ollama pull gemma3' to download the model");
        exit(1);
    }

    info!("⌨️ Press Ctrl+J to capture and summarize the current window");

    // Initialize hotkey manager
    let manager = GlobalHotKeyManager::new()
        .context("Failed to initialize hotkey manager")?;
    let hotkey = HotKey::new(Some(Modifiers::CONTROL), 'J');
    manager.register(hotkey, HOTKEY_ID)
        .context("Failed to register Ctrl+J hotkey")?;

    // Shared flag to track when hotkey is pressed
    let hotkey_pressed = Arc::new(Mutex::new(false));
    let hotkey_pressed_clone = hotkey_pressed.clone();

    // Create a channel to signal shutdown
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

    // Setup signal handlers for graceful shutdown
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down gracefully...");
                if let Some(tx) = shutdown_tx_clone.lock().unwrap().take() {
                    let _ = tx.send(());
                }
            }
            Err(err) => {
                error!("Failed to listen for Ctrl+C: {}", err);
            }
        }
    });

    // Listen for hotkey events
    tokio::spawn(async move {
        info!("🎧 Listening for hotkey events");
        let receiver = GlobalHotKeyManager::register_listener();
        while let Ok(event) = receiver.recv() {
            if event.id == HOTKEY_ID {
                info!("🔥 Hotkey triggered!");
                let mut flag = hotkey_pressed_clone.lock().unwrap();
                *flag = true;
            }
        }
    });

    // Main processing loop
    loop {
        // Check if hotkey was pressed
        {
            let mut flag = hotkey_pressed.lock().unwrap();
            if *flag {
                *flag = false;
                drop(flag); // Release lock before async work

                // Capture and process UI context
                match process_ui_context().await {
                    Ok(_) => {
                        info!("✅ Process completed successfully");
                        info!("📋 Summary copied to clipboard");
                    }
                    Err(e) => {
                        error!("❌ Error: {}", e);
                        
                        // Provide more specific error messages based on error kind
                        if e.to_string().contains("ollama") {
                            error!("💡 Ensure Ollama is running and try again");
                        } else if e.to_string().contains("desktop") || e.to_string().contains("window") {
                            error!("💡 Failed to access UI information. Try focusing on a different window.");
                        }
                    }
                }
            }
        }
        
        // Check for shutdown signal
        if let Ok(()) = shutdown_rx.try_recv() {
            info!("Shutdown signal received, cleaning up...");
            
            // Unregister hotkey
            if let Err(e) = manager.unregister(HOTKEY_ID) {
                warn!("Failed to unregister hotkey: {}", e);
            }
            
            info!("Goodbye! 👋");
            break;
        }
        
        // Sleep to prevent busy waiting
        sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}

async fn process_ui_context() -> Result<()> {
    info!("🖥️ Capturing UI context");
    
    // Initialize terminator desktop automation
    let desktop = Desktop::new(false, true).await
        .context("Failed to initialize desktop automation")?;
    
    // Get the current active window
    let window = desktop.get_current_browser_window().await
        .context("Failed to get current window. Ensure a window is in focus.")?;
    
    let window_name = window.name().unwrap_or_default();
    info!("🪟 Active window: {}", window_name);
    
    // Gather window information
    let window_attributes = format!("{:?}", window.attributes());
    
    // Get more details about the window structure
    let mut ui_details = String::new();
    
    // Find all elements in the window to create a rich UI tree
    let elements = window.locator(Selector::Role { 
        role: "".to_string(), 
        name: None 
    }).unwrap().all(Some(Duration::from_secs(5)), None).await
        .context("Failed to locate UI elements in the window")?;
    
    info!("🌳 Found {} UI elements", elements.len());
    
    // Build a UI tree representation
    ui_details.push_str(&format!("Window: {}\n", window_name));
    ui_details.push_str(&format!("Attributes: {}\n", window_attributes));
    ui_details.push_str("UI Elements:\n");
    
    for (i, element) in elements.iter().enumerate() {
        if i > 50 {
            // Limit to prevent overwhelming the LLM
            ui_details.push_str(&format!("... and {} more elements\n", elements.len() - 50));
            break;
        }
        
        // Get element details
        let name = element.name().unwrap_or_default();
        let role = element.role().unwrap_or_default();
        let text = element.text(5).unwrap_or_default();
        
        // Add element to UI tree
        ui_details.push_str(&format!("- Element {}: [Role: {}] [Name: {}]\n", i, role, name));
        if !text.is_empty() {
            ui_details.push_str(&format!("  Text: {}\n", text));
        }
    }
    
    info!("🤖 Sending context to Ollama for summarization");
    
    // Create prompt for Ollama
    let prompt = format!(
        r#"
You're helping build a local AI assistant that can understand what the user is doing based on their active screen context.
Here is raw UI Automation data extracted from the user's currently focused window:

```
{ui_details}
```

Your task is to:
1. Analyze this data and explain exactly what the user is doing
2. Identify the application/website being used
3. Summarize the key content visible on screen
4. Extract any important data points (names, numbers, references)

Format your response in markdown with sections:
- **Current Activity**: One-line summary
- **Application**: What program/site they're using
- **Content**: Brief summary of visible content
- **Key Data**: Important information extracted (names, numbers, etc.)
- **Potential Uses**: How this context could be used (1-2 suggestions)

Keep your entire response under 300 words and focus only on what's clearly evident.
"#
    );
    
    // Call Ollama with the prompt
    let output = Command::new("ollama")
        .args(["run", "gemma3", &prompt])
        .output()
        .context("Failed to run Ollama. Is it running and is the Gemma3 model available?")?;
    
    if !output.status.success() {
        return Err(anyhow!("Ollama command failed: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    let summary = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if summary.is_empty() {
        return Err(anyhow!("Ollama returned an empty response"));
    }
    
    // Copy to clipboard
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    clipboard.set_text(summary.clone())
        .context("Failed to copy summary to clipboard")?;
    
    info!("📋 Copied to clipboard:\n{}", summary);
    
    Ok(())
} 