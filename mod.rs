use anyhow::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub mod gemini;

/// Represents a single, non-streaming response from an LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
}

/// Represents a single chunk in a streaming LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMStreamChunk {
    pub content: String,
    /// Indicates if this is the final chunk in the stream.
    pub is_final: bool,
}

/// A common trait for Large Language Model providers.
///
/// This abstraction allows the application to interact with different LLMs
/// (like Gemini, OpenAI, etc.) through a consistent interface. It supports
/// both single-shot generation and streaming responses.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generates a complete response for a given prompt.
    async fn generate(&self, prompt: &str) -> Result<LLMResponse>;

    /// Generates a streaming response, returning a stream of chunks.
    async fn stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<LLMStreamChunk>> + Send>>>;
}