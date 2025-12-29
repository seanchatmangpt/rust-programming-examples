# Building Resilient Systems

## Learning Objectives

By the end of this chapter, you will:
- Design systems for graceful degradation under failure
- Implement circuit breaker patterns in Rust
- Build comprehensive observability into applications
- Plan for disaster recovery and production incidents
- Apply production-ready patterns to example projects

## Introduction

Resilience is the ability of a system to continue operating despite failures. As Rust systems move from prototypes to production, resilience becomes critical. This chapter explores patterns for building robust systems that fail gracefully, drawing lessons from five years of Rust in production (2021-2026) and applying them to our example projects.

## Graceful Degradation Principles

### The Failure Hierarchy

```
Total Failure (avoid)
    ↓
Degraded Service (acceptable)
    ↓
Reduced Performance (acceptable)
    ↓
Full Availability (goal)
```

**Design principle**: Systems should degrade incrementally, not catastrophically.

### Applying to actix-gcd

The `actix-gcd` web service demonstrates basic HTTP handling. Production-ready version would handle failures gracefully:

```rust
use actix_web::{web, HttpResponse, Result};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Clone)]
pub struct AppState {
    compute_timeout: Duration,
    max_input: u64,
}

async fn handle_gcd(
    state: web::Data<AppState>,
    query: web::Query<GcdParams>,
) -> Result<HttpResponse> {
    // 1. Input validation (fail fast)
    if query.n > state.max_input || query.m > state.max_input {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Input exceeds maximum allowed value",
        }));
    }

    // 2. Timeout protection (prevent resource exhaustion)
    let result = match timeout(
        state.compute_timeout,
        web::block(move || gcd(query.n, query.m))
    ).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => {
            // Computation error
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Computation failed",
            }));
        }
        Err(_) => {
            // Timeout
            return Ok(HttpResponse::RequestTimeout().json(ErrorResponse {
                error: "Computation exceeded time limit",
            }));
        }
    };

    // 3. Success response
    Ok(HttpResponse::Ok().json(GcdResponse { result }))
}
```

**Key techniques**:
1. **Input validation**: Reject invalid requests early
2. **Timeouts**: Prevent resource exhaustion
3. **Explicit error responses**: Clients know what happened

### Degradation Tiers

From `http-get` HTTP client pattern:

```rust
use reqwest::Client;
use std::time::Duration;

pub struct ResilientHttpClient {
    primary: Client,
    fallback: Option<Client>,
    cache: Cache,
}

impl ResilientHttpClient {
    pub async fn get(&self, url: &str) -> Result<String, HttpError> {
        // Tier 1: Try cache first (fastest)
        if let Some(cached) = self.cache.get(url).await {
            tracing::info!("Cache hit for {}", url);
            return Ok(cached);
        }

        // Tier 2: Try primary client (normal operation)
        match self.primary.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.text().await?;
                self.cache.set(url, &body).await;
                return Ok(body);
            }
            Ok(resp) => {
                tracing::warn!("Primary returned {}", resp.status());
            }
            Err(e) => {
                tracing::error!("Primary failed: {}", e);
            }
        }

        // Tier 3: Try fallback client (degraded service)
        if let Some(fallback) = &self.fallback {
            match fallback.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::warn!("Using fallback for {}", url);
                    let body = resp.text().await?;
                    return Ok(body);
                }
                Err(e) => {
                    tracing::error!("Fallback failed: {}", e);
                }
            }
        }

        // Tier 4: Return stale cache if available (maximum degradation)
        if let Some(stale) = self.cache.get_stale(url).await {
            tracing::warn!("Returning stale cache for {}", url);
            return Ok(stale);
        }

        // Tier 5: Total failure
        Err(HttpError::AllSourcesFailed)
    }
}
```

**Degradation levels**:
1. Fresh cache (0ms, perfect)
2. Primary HTTP (50ms, normal)
3. Fallback HTTP (200ms, degraded)
4. Stale cache (0ms, very degraded)
5. Failure (unavoidable)

## Circuit Breaker Pattern

Circuit breakers prevent cascading failures by temporarily disabling failing services.

### State Machine

```
           Failures exceed threshold
Closed ──────────────────────────────→ Open
  ↑              (reject requests)       │
  │                                      │
  │                                      │ After timeout
  │                                      ↓
  └────────────────────────────── Half-Open
        Requests succeed          (test recovery)
```

### Implementation

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,   // Operating normally
    Open,     // Failing, rejecting requests
    HalfOpen, // Testing recovery
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicUsize>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

pub struct CircuitBreakerConfig {
    failure_threshold: usize,
    timeout: Duration,
    half_open_requests: usize,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check circuit state
        let state = *self.state.read().await;

        match state {
            CircuitState::Open => {
                // Check if timeout elapsed
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() > self.config.timeout {
                        // Try recovery
                        *self.state.write().await = CircuitState::HalfOpen;
                        self.failure_count.store(0, Ordering::SeqCst);
                    } else {
                        // Still open, reject request
                        return Err(CircuitBreakerError::Open);
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests through
                if self.failure_count.load(Ordering::SeqCst) >= self.config.half_open_requests {
                    return Err(CircuitBreakerError::Open);
                }
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute the function
        match f.await {
            Ok(result) => {
                // Success - reset failure count
                if *self.state.read().await == CircuitState::HalfOpen {
                    *self.state.write().await = CircuitState::Closed;
                }
                self.failure_count.store(0, Ordering::SeqCst);
                Ok(result)
            }
            Err(e) => {
                // Failure - increment count
                let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

                if failures >= self.config.failure_threshold {
                    *self.state.write().await = CircuitState::Open;
                    *self.last_failure_time.write().await = Some(Instant::now());
                    tracing::error!(
                        "Circuit breaker opened after {} failures",
                        failures
                    );
                }

                Err(CircuitBreakerError::Failure(e))
            }
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,          // Circuit is open, request rejected
    Failure(E),    // Underlying operation failed
}
```

### Using Circuit Breakers

From `many-requests` concurrent HTTP pattern:

```rust
async fn fetch_with_circuit_breaker(
    client: &Client,
    url: &str,
    breaker: &CircuitBreaker,
) -> Result<String, Box<dyn std::error::Error>> {
    breaker
        .call(async {
            let resp = client.get(url).send().await?;
            let body = resp.text().await?;
            Ok::<_, reqwest::Error>(body)
        })
        .await
        .map_err(|e| match e {
            CircuitBreakerError::Open => {
                "Circuit breaker open, service unavailable".into()
            }
            CircuitBreakerError::Failure(e) => Box::new(e) as Box<dyn std::error::Error>,
        })
}

// Usage in many-requests pattern
async fn fetch_all(urls: Vec<String>) -> Vec<Result<String, String>> {
    let client = Client::new();
    let breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(30),
        half_open_requests: 3,
    }));

    let futures = urls.into_iter().map(|url| {
        let client = client.clone();
        let breaker = breaker.clone();
        async move {
            fetch_with_circuit_breaker(&client, &url, &breaker)
                .await
                .map_err(|e| e.to_string())
        }
    });

    futures::future::join_all(futures).await
}
```

## Observability and Monitoring

Production systems require visibility into runtime behavior.

### The Three Pillars

1. **Metrics**: What is happening? (counters, gauges, histograms)
2. **Logs**: Why did it happen? (structured events)
3. **Traces**: How did it happen? (distributed request tracking)

### Implementing with tracing

The `tracing` crate is the 2026 standard for observability:

```rust
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Application initialization
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Instrumented function
#[instrument(skip(data), fields(data_len = data.len()))]
async fn process_data(data: Vec<u8>) -> Result<ProcessResult, ProcessError> {
    debug!("Starting data processing");

    let validated = validate_data(&data)?;
    info!("Data validated successfully");

    let result = match transform_data(validated).await {
        Ok(r) => {
            info!(result_size = r.len(), "Transformation complete");
            r
        }
        Err(e) => {
            error!(error = %e, "Transformation failed");
            return Err(e.into());
        }
    };

    Ok(result)
}
```

### Metrics with prometheus

From `actix-gcd` production version:

```rust
use prometheus::{
    Counter, Histogram, HistogramOpts, IntCounter, Opts, Registry,
};

pub struct Metrics {
    requests_total: IntCounter,
    request_duration: Histogram,
    errors_total: Counter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let requests_total = IntCounter::new(
            "gcd_requests_total",
            "Total GCD requests processed",
        )?;
        registry.register(Box::new(requests_total.clone()))?;

        let request_duration = Histogram::with_opts(HistogramOpts::new(
            "gcd_request_duration_seconds",
            "GCD request duration in seconds",
        ))?;
        registry.register(Box::new(request_duration.clone()))?;

        let errors_total = Counter::new(
            "gcd_errors_total",
            "Total GCD errors encountered",
        )?;
        registry.register(Box::new(errors_total.clone()))?;

        Ok(Metrics {
            requests_total,
            request_duration,
            errors_total,
        })
    }

    pub fn record_request(&self, duration: Duration, success: bool) {
        self.requests_total.inc();
        self.request_duration.observe(duration.as_secs_f64());
        if !success {
            self.errors_total.inc();
        }
    }
}

// Middleware to record metrics
async fn handle_gcd_with_metrics(
    state: web::Data<AppState>,
    metrics: web::Data<Metrics>,
    query: web::Query<GcdParams>,
) -> Result<HttpResponse> {
    let start = Instant::now();

    let result = handle_gcd(state, query).await;

    let success = result.is_ok();
    metrics.record_request(start.elapsed(), success);

    result
}
```

### Distributed Tracing

For microservices (extending `actix-gcd` to multi-service):

```rust
use opentelemetry::{global, sdk::trace as sdktrace, trace::Tracer};
use tracing_opentelemetry::OpenTelemetryLayer;

fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("gcd-service")
        .install_simple()?;

    let opentelemetry = OpenTelemetryLayer::new(tracer);

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[instrument]
async fn handle_gcd_distributed(
    query: web::Query<GcdParams>,
) -> Result<HttpResponse> {
    // Automatic trace context propagation
    let span = tracing::Span::current();
    span.record("n", query.n);
    span.record("m", query.m);

    // Call another service (trace context propagated automatically)
    let validation = validate_service::check(query.n, query.m).await?;

    // Compute GCD (traced)
    let result = gcd(query.n, query.m);

    Ok(HttpResponse::Ok().json(GcdResponse { result }))
}
```

## Disaster Recovery

Plan for total failures before they happen.

### Backup and Recovery Strategies

For stateful systems (not directly in our examples, but principles apply):

```rust
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    timestamp: u64,
    data: Vec<u8>,
    checksum: u64,
}

pub struct RecoveryManager {
    snapshot_path: PathBuf,
}

impl RecoveryManager {
    pub async fn save_snapshot(&self, data: &[u8]) -> Result<(), io::Error> {
        let snapshot = Snapshot {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: data.to_vec(),
            checksum: calculate_checksum(data),
        };

        let json = serde_json::to_vec(&snapshot)?;

        // Atomic write: write to temp, then rename
        let temp_path = self.snapshot_path.with_extension("tmp");
        let mut file = BufWriter::new(fs::File::create(&temp_path).await?);
        file.write_all(&json).await?;
        file.flush().await?;

        fs::rename(&temp_path, &self.snapshot_path).await?;

        Ok(())
    }

    pub async fn load_snapshot(&self) -> Result<Vec<u8>, RecoveryError> {
        let json = fs::read(&self.snapshot_path).await?;
        let snapshot: Snapshot = serde_json::from_slice(&json)?;

        // Verify checksum
        if calculate_checksum(&snapshot.data) != snapshot.checksum {
            return Err(RecoveryError::CorruptedSnapshot);
        }

        Ok(snapshot.data)
    }
}
```

### Health Checks

Essential for load balancers and orchestration:

```rust
// actix-gcd health endpoint
async fn health_check(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // Check dependencies
    let db_healthy = check_database(&state.db_pool).await;
    let cache_healthy = check_cache(&state.cache).await;

    if db_healthy && cache_healthy {
        Ok(HttpResponse::Ok().json(HealthResponse {
            status: "healthy",
            timestamp: Utc::now().to_rfc3339(),
        }))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(HealthResponse {
            status: "unhealthy",
            timestamp: Utc::now().to_rfc3339(),
        }))
    }
}

async fn readiness_check(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // More strict: can we handle traffic?
    if state.ready.load(Ordering::SeqCst) {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::ServiceUnavailable().finish())
    }
}
```

## Production Readiness Checklist

For each project in this repository, production deployment requires:

### Infrastructure
- [ ] Health check endpoints
- [ ] Metrics exposure (Prometheus format)
- [ ] Structured logging (JSON output)
- [ ] Distributed tracing integration
- [ ] Graceful shutdown handling

### Reliability
- [ ] Input validation on all external data
- [ ] Timeouts on all I/O operations
- [ ] Circuit breakers for external dependencies
- [ ] Retry logic with exponential backoff
- [ ] Rate limiting to prevent abuse

### Monitoring
- [ ] Request/response latency histograms
- [ ] Error rate tracking
- [ ] Resource usage metrics (CPU, memory)
- [ ] Custom business metrics
- [ ] Alerting rules defined

### Security
- [ ] Input sanitization
- [ ] Secrets in environment variables, not code
- [ ] TLS for all network communication
- [ ] Rate limiting
- [ ] Regular dependency audits (`cargo audit`)

### Operations
- [ ] Runbook for common incidents
- [ ] Backup and recovery procedures
- [ ] Load testing results
- [ ] Capacity planning documentation
- [ ] Incident response plan

## Summary

Resilient systems in Rust leverage:
- **Graceful degradation**: Fail incrementally, not catastrophically
- **Circuit breakers**: Prevent cascading failures
- **Observability**: Metrics, logs, and traces for visibility
- **Disaster recovery**: Plan for failure before it happens

The patterns in this chapter—applied to our example projects like `actix-gcd`, `http-get`, and `many-requests`—transform educational code into production-ready systems. Resilience is not an afterthought; it's a design requirement.

## Further Reading

- Chapter 11.4: Team organization for operating resilient systems
- Chapter 9: Error handling foundations
- Chapter 8: Async patterns for resilient I/O
