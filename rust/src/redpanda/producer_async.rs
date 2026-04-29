use anyhow::Result;
use log::info;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;

use crate::redpanda::config::KafkaConfig;
use std::time::Duration;
use futures::Future;

#[derive(Clone)]
pub struct RedpandaProducerAsync {
    producer: FutureProducer,
    topic: String,
}

impl RedpandaProducerAsync {
    pub fn new(config: &KafkaConfig) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            // --- connection ---
            .set("bootstrap.servers", &config.broker)

            // --- SASL/PLAIN ---
            // .set("security.protocol", "SASL_PLAINTEXT")
            // .set("sasl.mechanism", "PLAIN")
            // .set("sasl.username", &config.username)
            // .set("sasl.password", &config.password)

            // --- Optional tuning ---
            // .set("message.timeout.ms", "5000")

            // durability (IMPORTANT)
            // .set("acks", "all")
            // .set("acks", "1")

            // batch
            // .set("linger.ms", "5")
            // .set("batch.size", "65536")
            // .set("compression.type", "lz4")
            .create()?;

        info!("🧩 Async Producer connected to {}", &config.broker);

        Ok(Self {
            producer,
            topic: config.topic.clone(),
        })
    }

    /// 🔥 Async send: mengembalikan DeliveryReport lengkap
    pub async fn send(&self, message: &str) -> Result<()> {
        let record = FutureRecord::to(&self.topic)
            .payload(message)
            .key("");

        // FutureProducer → .send() returns a Future
        let delivery_status = self
            .producer
            .send(record, Duration::from_secs(1))
            .await;

        match delivery_status {
            Ok((partition, offset)) => {
                info!(
                    "📦 delivered: '{}' → partition {}, offset {}",
                    message, partition, offset
                );
                Ok(())
            }
            Err((err, _msg)) => {
                Err(anyhow::anyhow!("❌ delivery failed: {}", err))
            }
        }
    }

    
    pub fn send_join(
        &self,
        message: String,
    ) -> impl Future<Output = Result<()>> + '_ {
        let topic = self.topic.clone();
        let producer = self.producer.clone();

        async move {
            let record = FutureRecord::to(&topic)
                .payload(&message)
                .key("");

            let delivery_status = producer
                .send(record, Duration::from_secs(5))
                .await;

            match delivery_status {
                Ok(_) => Ok(()),
                Err((err, _)) => Err(anyhow::anyhow!("delivery failed: {}", err)),
            }
        }
    }
    
}
