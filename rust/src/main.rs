mod redpanda;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use chrono::Utc;
use dotenvy;
use env_logger::Env;
use log::info;
use rand::Rng;
use std::time::Instant;

use redpanda::config::KafkaConfig;
use redpanda::producer_async::RedpandaProducerAsync;
use redpanda::consumer_async::RedpandaConsumerAsync;

#[derive(ValueEnum, Clone, Debug)]
enum Mode {
    AsyncProducer,
    AsyncProducerJoin,
    AsyncProducerBackpressure,
    AsyncProducerBatch,
    AsyncConsumer,
    AsyncConsumerProcessingDelay,
}

#[derive(Parser)]
#[command(name = "rust_redpanda", version, about = "redpanda demo in Rust with (Async)", long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

fn main() -> Result<()> {
    println!("📂 Current dir: {:?}", std::env::current_dir());

    // Load .env
    if dotenvy::dotenv().is_err() {
        panic!("⚠️ .env file not found");
    }

    // Init logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    let config = KafkaConfig::new();

    match cli.mode {
        Mode::AsyncProducer => run_async_producer(config)?,
        Mode::AsyncProducerJoin => run_async_producer_join(config)?,    
        Mode::AsyncProducerBackpressure => run_async_producer_backpressure(config)?,  
        Mode::AsyncProducerBatch => run_async_producer_batch(config)?,
        Mode::AsyncConsumer => run_async_consumer(config)?,
        Mode::AsyncConsumerProcessingDelay => run_async_consumer_prosessing_delay(config)?,
    }

    Ok(())
}

fn make_random_json() -> String {
    let temp = rand::thread_rng().gen_range(20.0..30.0);
    let humidity = rand::thread_rng().gen_range(50.0..80.0);
    let timestamp = Utc::now().to_rfc3339();

    format!(
        r#"{{"id":"A001","time":"{}","temp":{:.2},"humidity":{:.2}}}"#,
        timestamp, temp, humidity
    )
}

fn run_async_producer(config: KafkaConfig) -> Result<()> {
    info!("🚀 Running ASYNC producer (baseline test)...");
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let producer = RedpandaProducerAsync::new(&config)?;

        let total_messages = 100_000;
        let start = Instant::now();

        for i in 0..total_messages {
            let msg = make_random_json();
            producer.send(&msg).await?;

            if i % 10_000 == 0 && i != 0 {
                println!("Sent {} messages...", i);
            }
        }

        let duration = start.elapsed();
        let throughput = total_messages as f64 / duration.as_secs_f64();

        println!("\n✅ Done!");
        println!("Total: {}", total_messages);
        println!("Time: {:.2?}", duration);
        println!("Throughput: {:.2} msg/sec", throughput);

        Ok(())
    })
}

fn run_async_producer_join(config: KafkaConfig) -> Result<()> {
    info!("🚀 Running ASYNC producer (concurrent join)...");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        use futures::future::join_all;

        let producer = RedpandaProducerAsync::new(&config)?;

        let total_messages = 1_000_000;
        let start = Instant::now();

        let mut tasks = Vec::with_capacity(total_messages);

        for i in 0..total_messages {
            let msg = make_random_json();

            // IMPORTANT: move ownership (no &str)
            let fut = producer.send_join(msg);
            tasks.push(fut);

            if i % 10_000 == 0 && i != 0 {
                println!("Queued {} messages...", i);
            }
        }

        println!("⏳ Waiting for all messages to be delivered...");

        let results = join_all(tasks).await;

        // handle errors (important for observability)
        let mut error_count = 0;
        for res in results {
            if let Err(e) = res {
                error_count += 1;
                eprintln!("❌ {}", e);
            }
        }

        let duration = start.elapsed();
        let throughput = total_messages as f64 / duration.as_secs_f64();

        println!("\n✅ Done!");
        println!("Total: {}", total_messages);
        println!("Errors: {}", error_count);
        println!("Time: {:.2?}", duration);
        println!("Throughput: {:.2} msg/sec", throughput);

        Ok(())
    })

}

fn run_async_producer_backpressure(config: KafkaConfig) -> Result<()> {
    info!("🚀 Running ASYNC producer (backpressure)...");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        use futures::future::join_all;
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        let producer = RedpandaProducerAsync::new(&config)?;

        let total_messages = 100_000;
        let start = Instant::now();

        // 🔥 limit concurrency (core of backpressure)
        let semaphore = Arc::new(Semaphore::new(5000));

        let mut tasks = Vec::with_capacity(total_messages);

        for i in 0..total_messages {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let producer = producer.clone();
            let msg = make_random_json();

            let task = tokio::spawn(async move {
                let _permit = permit; // release automatically when dropped
                producer.send_join(msg).await
            });

            tasks.push(task);

            if i % 10_000 == 0 && i != 0 {
                println!("Queued {} messages...", i);
            }
        }

        println!("⏳ Waiting for all messages to be delivered...");

        let results = join_all(tasks).await;

        // 🔍 proper error handling (join + delivery)
        let mut error_count = 0;

        for res in results {
            match res {
                Ok(inner) => {
                    if let Err(e) = inner {
                        error_count += 1;
                        eprintln!("❌ delivery error: {}", e);
                    }
                }
                Err(join_err) => {
                    error_count += 1;
                    eprintln!("❌ task join error: {}", join_err);
                }
            }
        }

        let duration = start.elapsed();
        let throughput = total_messages as f64 / duration.as_secs_f64();

        println!("\n✅ Done!");
        println!("Total: {}", total_messages);
        println!("Errors: {}", error_count);
        println!("Time: {:.2?}", duration);
        println!("Throughput: {:.2} msg/sec", throughput);

        Ok(())
    })

}

fn run_async_producer_batch(config: KafkaConfig) -> Result<()> {
    info!("🚀 Running ASYNC producer (batching + backpressure)...");
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        use futures::future::join_all;
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        // 🔥 Producer dengan batching config
        let producer = RedpandaProducerAsync::new(&config)?;

        let total_messages = 100_000;
        let start = Instant::now();

        // 🔥 tetap pakai backpressure
        let semaphore = Arc::new(Semaphore::new(1000));

        let mut tasks = Vec::with_capacity(total_messages);

        for i in 0..total_messages {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let producer = producer.clone();
            let msg = make_random_json();

            let task = tokio::spawn(async move {
                let _permit = permit;
                producer.send_join(msg).await
            });

            tasks.push(task);

            if i % 10_000 == 0 && i != 0 {
                println!("Queued {} messages...", i);
            }
        }

        println!("⏳ Waiting for all messages to be delivered...");

        let results = join_all(tasks).await;

        // error handling
        let mut error_count = 0;

        for res in results {
            match res {
                Ok(inner) => {
                    if let Err(e) = inner {
                        error_count += 1;
                        eprintln!("❌ delivery error: {}", e);
                    }
                }
                Err(join_err) => {
                    error_count += 1;
                    eprintln!("❌ task join error: {}", join_err);
                }
            }
        }

        let duration = start.elapsed();
        let throughput = total_messages as f64 / duration.as_secs_f64();

        println!("\n✅ Done!");
        println!("Total: {}", total_messages);
        println!("Errors: {}", error_count);
        println!("Time: {:.2?}", duration);
        println!("Throughput: {:.2} msg/sec", throughput);

        Ok(())
    })


}

fn run_async_consumer(config: KafkaConfig) -> Result<()> {
    info!("📥 Running ASYNC consumer...");
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let consumer = RedpandaConsumerAsync::new(&config)?;
        consumer.run().await?;
        Ok(())
    })
}


fn run_async_consumer_prosessing_delay(config: KafkaConfig) -> Result<()> {
    info!("📥 Running ASYNC consumer with processing delay...");
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let consumer = RedpandaConsumerAsync::new(&config)?;
        consumer.run_processing_delay().await?;
        Ok(())
    })
}
