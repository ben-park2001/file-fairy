use anyhow::Result;
use file_fairy_llm::FileLlm;
use file_fairy_llm::{LlmConfig, client};
use std::path::PathBuf;
use std::time::Instant;

/// Example usage of the LocalLlmClient showing efficiency improvements
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client with a local GGUF model
    // Replace this path with your actual model path
    let model_path = PathBuf::from("models/gemma-3n-E2B-it-Q4_K_M.gguf");

    let config = LlmConfig::new(model_path)
        .with_max_tokens(256) // Limit output to 256 tokens
        .with_context_size(4096) // Set context size to 4096
        .with_threads(8) // Use 8 threads
        .with_seed(103); // Set random seed for reproducible results

    let client = client::LocalLlmClient::new(config)?;

    println!("LocalLlmClient initialized successfully!");

    // Check if model is loaded (should be false initially)
    println!("Model loaded initially: {}", client.is_model_loaded().await);

    // Optionally, preload the model (this will be slow the first time)
    println!("Preloading model...");
    let preload_start = Instant::now();
    match client.preload_model().await {
        Ok(_) => {
            let preload_time = preload_start.elapsed();
            println!(
                "Model preloaded successfully in {:.2}s",
                preload_time.as_secs_f64()
            );
            println!(
                "Model loaded after preload: {}",
                client.is_model_loaded().await
            );
        }
        Err(e) => {
            println!("Failed to preload model: {}", e);
            println!(
                "Note: This is expected if the model file doesn't exist at the specified path."
            );
            println!("The model would be loaded automatically on first inference call.");
            return Ok(());
        }
    }

    // Random filename for testing
    let random_filename = "example.txt";

    // Sample file contents to test efficiency with multiple calls
    let file_contents = vec![
        r#"Introduction to Machine Learning

Machine learning is a subset of artificial intelligence (AI) that enables
computers to learn and make decisions from data without being explicitly
programmed for every task. It involves algorithms that can identify patterns
in data and make predictions or decisions based on these patterns.

There are three main types of machine learning:
1. Supervised Learning
2. Unsupervised Learning
3. Reinforcement Learning"#,
        r#"Rust Programming Language Overview

Rust is a systems programming language that focuses on safety, speed, and
concurrency. It achieves memory safety without using a garbage collector,
making it suitable for performance-critical applications.

Key features of Rust include:
- Memory safety without garbage collection
- Zero-cost abstractions
- Fearless concurrency
- Cross-platform support"#,
        r#"Climate Change and Environmental Impact

Climate change refers to long-term shifts in global or regional climate
patterns. Since the mid-20th century, scientists have observed that the
primary cause of climate change is human activities, particularly the
burning of fossil fuels.

The main effects include:
- Rising global temperatures
- Sea level rise
- Changes in precipitation patterns
- More frequent extreme weather events"#,
    ];

    // Test multiple inference calls to demonstrate efficiency
    // After the first call, subsequent calls should be much faster since the model is cached
    for (i, content) in file_contents.iter().enumerate() {
        println!("\n--- Processing file {} ---", i + 1);
        let start = Instant::now();

        match client.suggest_filename(random_filename, content).await {
            Ok(filename) => {
                let duration = start.elapsed();
                println!("Suggested filename: {}", filename);
                println!("Time taken: {:.2}s", duration.as_secs_f64());
            }
            Err(e) => {
                println!("Error suggesting filename: {}", e);
            }
        }
    }

    println!("\nExample completed successfully!");
    println!("Note: After the first call, subsequent calls should be much faster");
    println!("because the model is cached in memory and doesn't need to be reloaded.");

    Ok(())
}
