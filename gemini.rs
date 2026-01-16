use super::{LLMProvider, LLMResponse, LLMStreamChunk};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use circuit_breaker::{CircuitBreaker, State};
use futures::stream::{self, Stream};
use metrics::{counter, histogram};
use crate::safety::content_filter::{
    BasicContentFilter, ContentFilter, FilterAction,
};
use std::{
    env, pin::Pin, sync::Mutex, time::Duration, time::Instant,
};
use tokio::{sync::Semaphore, time::timeout};

const CONCURRENT_REQUEST_LIMIT: usize = 10;
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

/// A client for interacting with the Google Gemini API.
pub struct GeminiClient {
    _api_key: String,
    _http_client: reqwest::Client,
    concurrency_limiter: Semaphore,
    circuit_breaker: Mutex<CircuitBreaker>,
    content_filter: Box<dyn ContentFilter>,
}

impl GeminiClient {
    const PROVIDER_NAME: &'static str = "gemini";

    /// Creates a new Gemini client.
    ///
    /// It reads the `GEMINI_API_KEY` from the environment.
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY environment variable not set")?;

        Ok(Self {
            _api_key: api_key,
            _http_client: reqwest::Client::new(),
            concurrency_limiter: Semaphore::new(CONCURRENT_REQUEST_LIMIT),
            // - Open after 5 consecutive failures
            // - Reset to half-open after 30 seconds
            circuit_breaker: Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            content_filter: Box::new(BasicContentFilter),
        })
    }
}

#[async_trait]
impl LLMProvider for GeminiClient {
    /// Generates a response from the Gemini API.
    ///
    /// TODO: Implement the actual API call to the Gemini `generateContent` endpoint.
    async fn generate(&self, prompt: &str) -> Result<LLMResponse> {
        // Check circuit breaker state before proceeding.
        if self.circuit_breaker.lock().unwrap().is_open() {
            counter!("llm_circuit_breaker_opens_total", 1, "provider" => Self::PROVIDER_NAME);
            return Err(anyhow!("Circuit breaker is open. Gemini API is likely unavailable."));
        }

        // Acquire a permit from the semaphore, waiting if all are taken.
        let _permit = self.concurrency_limiter.acquire().await.context("Failed to acquire concurrency permit")?;

        println!("🤖 [Gemini] Generating response for prompt: '{}'", prompt);

        let start_time = Instant::now();

        // Wrap the core logic in a timeout.
        let timeout_duration = Duration::from_secs(REQUEST_TIMEOUT_SECONDS);
        let operation = async {
            // Placeholder for the actual API call.
            Err(anyhow!("Gemini `generate` method is not yet implemented."))
        };

        let result: Result<LLMResponse> = match timeout(timeout_duration, operation).await {
            Ok(res) => res, // Result from the operation itself
            Err(_) => Err(anyhow!("Request timed out after {} seconds", REQUEST_TIMEOUT_SECONDS)), // Timeout error
        };

        // Record metrics
        histogram!("llm_request_duration_seconds", start_time.elapsed().as_secs_f64(), "provider" => Self::PROVIDER_NAME, "method" => "generate");
        counter!("llm_requests_total", 1, "provider" => Self::PROVIDER_NAME, "method" => "generate");

        // Report failure outcome to the circuit breaker and record errors.
        if result.is_err() {
            self.circuit_breaker.lock().unwrap().fail();
            counter!("llm_errors_total", 1, "provider" => Self::PROVIDER_NAME, "method" => "generate");
            return result;
        }

        let response = result.unwrap();

        // Post-processing: Run the response through the content filter.
        match self.content_filter.check_content(&response.content)? {
            FilterAction::Allow => {
                self.circuit_breaker.lock().unwrap().success();
                Ok(response)
            }
            FilterAction::Flag => {
                counter!("llm_content_filter_flags_total", 1, "provider" => Self::PROVIDER_NAME);
                // The request succeeded but was flagged. This is not a failure for the circuit breaker.
                self.circuit_breaker.lock().unwrap().success();
                Err(anyhow!("Response was flagged by content filter and has been quarantined."))
            }
        }
    }

    /// Generates a streaming response from the Gemini API.
    ///
    /// TODO: Implement the streaming API call to the Gemini `streamGenerateContent` endpoint.
    async fn stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LLMStreamChunk>> + Send>>> {
        // Also protect streaming calls with the circuit breaker.
        if self.circuit_breaker.lock().unwrap().is_open() {
            counter!("llm_circuit_breaker_opens_total", 1, "provider" => Self::PROVIDER_NAME);
            return Err(anyhow!("Circuit breaker is open. Gemini API is likely unavailable."));
        }

        // Acquire a permit from the semaphore.
        let _permit = self.concurrency_limiter.acquire().await.context("Failed to acquire concurrency permit")?;

        println!("🤖 [Gemini] Streaming response for prompt: '{}'", prompt);

        // Record metrics for the stream initiation
        counter!("llm_requests_total", 1, "provider" => Self::PROVIDER_NAME, "method" => "stream");

        // Placeholder implementation
        let stream = stream::iter(vec![Err(anyhow!("Gemini `stream` method is not yet implemented."))]);

        // Note: For streaming, latency and error metrics would need to be handled
        // within the stream itself as chunks are processed. The circuit breaker would
        // also need to be notified of failures within the stream.
        // For now, we only report success on stream initiation.
        self.circuit_breaker.lock().unwrap().success();

        Ok(Box::pin(stream))
    }
}