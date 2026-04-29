# 🚀 Rust Performance & Backpressure Engineering with Redpanda

This project explores how a streaming system behaves under load using:

* **Rust (Tokio + rdkafka)**
* **Redpanda (Kafka-compatible broker)**

Focus areas:

* Throughput
* Concurrency
* Backpressure
* Batching
* Consumer Lag

---

## 📦 Setup

### 1. Start Redpanda

```bash
make up
```

Check status:

```bash
make ps
make logs-redpanda
```

---

### 2. Create Topic

Via UI:

```
http://localhost:8080
```

Or CLI:

```bash
make test-topic
```

---

## ⚙️ Environment

```env
REDPANDA_BROKER=localhost:19092
REDPANDA_TOPIC=Backpressure-Rust
REDPANDA_GROUP_ID=group-1
```

---

# 🧪 Experiments

---

## 1. Baseline (Sequential)

* `.await` on every send
* no concurrency

👉 Result:

* extremely slow (~150 msg/sec)

---

## 2. Concurrent Producer (Unbounded)

* `join_all`
* no concurrency limit

👉 Result:

* massive throughput increase (~90K+ msg/sec)
* but uncontrolled resource usage

---

## 3. Backpressure (Semaphore)

```rust
Semaphore::new(1000)
```

👉 Result:

* stable throughput
* controlled resource usage

---

## 4. Batching

```rust
.set("linger.ms", "5")
.set("batch.size", "65536")
.set("compression.type", "lz4")
```

👉 Result:

* ~2x throughput improvement without increasing concurrency

---

## 📊 Example Results

### Backpressure (Concurrency = 1000)

```
Time: 622.86ms
Throughput: 160,548 msg/sec
```

### Backpressure + Batching

```
Time: 290.35ms
Throughput: 344,416 msg/sec
```

---

# 🧠 Key Insights

### 1. Async ≠ High Throughput

Awaiting each send results in sequential execution.

---

### 2. Concurrency Unlocks Throughput

Removing per-message waits drastically increases performance.

---

### 3. Unbounded Concurrency Is Dangerous

High throughput without limits can lead to:

* memory pressure
* CPU spikes
* instability

---

### 4. Backpressure Stabilizes the System

Limiting in-flight requests keeps the system predictable.

---

### 5. Batching Improves Efficiency

Concurrency increases parallelism.
Batching increases efficiency.

---

# 📥 Consumer (Lag Simulation)

The consumer simulates slow processing:

```rust
tokio::time::sleep(Duration::from_millis(5)).await;
```

👉 This creates:

* consumer lag
* backlog buildup
* real-world pressure scenario

---

# 🚀 Run Commands

### Producer (Backpressure)

```bash
cargo run --release -- async-producer-backpressure
```

### Producer (Batching)

```bash
cargo run --release -- async-producer-batch
```

### Consumer

```bash
cargo run --release -- async-consumer-processing-delay
```

---

# 🔥 What This Project Shows

This is not just a performance test.

It demonstrates:

> How bottlenecks shift under load:

* from latency
* to CPU
* to memory
* to efficiency

---

# 📌 Conclusion

High throughput alone is not the goal.

The real goal is:

> sustaining high throughput without breaking the system

---

# 📚 Related Article

Full explanation and deep dive:

👉 *Rust Performance & Backpressure Engineering: What Breaks Under Load?*
👉 *[Article](https://andriantriputra.medium.com/redpanda-rust-performance-backpressure-engineering-what-breaks-under-load-232b1cba79a1)

---

# 🧠 Final Insight

> The fastest system is not the one with the highest throughput,
> but the one that remains stable under pressure.


---

## 👤 Author

**Andrian Tri Putra**

* Medium: https://andriantriputra.medium.com/
* GitHub: https://github.com/andriantp
* GitHub (alt): https://github.com/AndrianTriPutra

---

## 📄 License

Licensed under the Apache License 2.0
