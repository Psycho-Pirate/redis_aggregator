use redis::{AsyncCommands, Client, RedisResult};
use tokio::{sync::mpsc, signal, select};
use log::{info, error};
use env_logger;
use std::collections::HashMap;

/// List of input channels to subscribe to.
const CHANNELS: [&str; 3] = ["inputA", "inputB", "inputC"];
/// The output Redis channel where aggregated results will be published.
const OUTPUT_CHANNEL: &str = "outputChannel";

#[tokio::main]
async fn main() -> RedisResult<()> {
    env_logger::init();
    info!("Starting Redis Aggregator...");

    // Initialize Redis client
    let client = Client::open("redis://127.0.0.1/")?;
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Spawn tasks to read from Redis Pub/Sub for each channel
    for &channel in &CHANNELS {
        let client = client.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut pubsub = match client.get_async_connection().await {
                Ok(conn) => conn.into_pubsub(),
                Err(e) => {
                    error!("Failed to get Redis connection: {}", e);
                    return;
                }
            };
            
            // Subscribe to the channel
            if let Err(e) = pubsub.subscribe(channel).await {
                error!("Failed to subscribe to {}: {}", channel, e);
                return;
            }
            info!("Subscribed to channel: {}", channel);
            let mut stream = pubsub.into_on_message();
            
            // Process incoming messages
            while let Some(msg) = tokio_stream::StreamExt::next(&mut stream).await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    if tx_clone.send(payload).await.is_err() {
                        error!("Failed to send message from {}", channel);
                        break;
                    }
                }
            }
        });
    }

    // Create a separate client for processing and output publishing
    let processing_client = Client::open("redis://127.0.0.1/")?;
    let mut output_conn = processing_client.get_async_connection().await?;
    let mut state = HashMap::new();

    // Spawn a task to process received messages and publish results
    let processor_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let aggregated = process_message(msg, &mut state);
            if let Err(e) = output_conn.publish::<&str, String, ()>(OUTPUT_CHANNEL, aggregated.clone()).await {
                error!("Failed to publish message: {}", e);
            } else {
                info!("Published processed message: {}", aggregated);
            }
        }
    });

    // Graceful shutdown handling
    select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, cleaning up...");
        }
    }

    processor_handle.abort();
    info!("Shutting down...");
    Ok(())
}

/// Processes an incoming message by counting occurrences.
/// 
/// # Arguments
/// * `msg` - The received message string.
/// * `state` - A mutable reference to a HashMap storing message counts.
/// 
/// # Returns
/// A formatted string indicating how many times the message has appeared.
fn process_message(msg: String, state: &mut HashMap<String, i32>) -> String {
    let count = state.entry(msg.clone()).or_insert(0);
    *count += 1;
    format!("{} appeared {} times", msg, count)
}