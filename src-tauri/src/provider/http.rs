use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{
    AIProvider, GenerateRequest, GenerateResponse, ModelProfile, ProviderCapabilities,
    ProviderDescriptor, ProviderError, ProviderResult, ProviderStream, ProviderUsage, StreamEvent,
};

const CHAT_COMPLETIONS_PATH: &str = "/chat/completions";

#[derive(Clone)]
pub(crate) struct HttpRequest {
    pub(crate) url: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) body: Value,
}

impl HttpRequest {
    fn new(url: String, headers: BTreeMap<String, String>, body: Value) -> Self {
        Self { url, headers, body }
    }
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header_names: Vec<&str> = self.headers.keys().map(String::as_str).collect();
        formatter
            .debug_struct("HttpRequest")
            .field("url", &"<redacted>")
            .field("header_names", &header_names)
            .field("body", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HttpTransportError {
    RequestFailed,
    ResponseReadFailed,
    Status(u16),
}

pub(crate) type HttpChunkStream =
    Pin<Box<dyn Stream<Item = Result<Vec<u8>, HttpTransportError>> + Send>>;

#[async_trait]
pub(crate) trait HttpTransport: Send + Sync {
    async fn post_json(&self, request: HttpRequest) -> Result<HttpResponse, HttpTransportError>;

    async fn post_stream(
        &self,
        request: HttpRequest,
    ) -> Result<HttpChunkStream, HttpTransportError>;
}

#[derive(Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn post_json(&self, request: HttpRequest) -> Result<HttpResponse, HttpTransportError> {
        let response = self
            .client
            .post(request.url)
            .headers(headers(&request.headers)?)
            .json(&request.body)
            .send()
            .await
            .map_err(|_| HttpTransportError::RequestFailed)?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|_| HttpTransportError::ResponseReadFailed)?;
        Ok(HttpResponse { status, body })
    }

    async fn post_stream(
        &self,
        request: HttpRequest,
    ) -> Result<HttpChunkStream, HttpTransportError> {
        let response = self
            .client
            .post(request.url)
            .headers(headers(&request.headers)?)
            .json(&request.body)
            .send()
            .await
            .map_err(|_| HttpTransportError::RequestFailed)?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(HttpTransportError::Status(status));
        }
        let chunks = response.bytes_stream().map(|chunk| {
            chunk
                .map(|bytes| bytes.to_vec())
                .map_err(|_| HttpTransportError::ResponseReadFailed)
        });
        Ok(Box::pin(chunks))
    }
}

fn headers(
    values: &BTreeMap<String, String>,
) -> Result<reqwest::header::HeaderMap, HttpTransportError> {
    values
        .iter()
        .map(|(name, value)| {
            Ok((
                reqwest::header::HeaderName::try_from(name)
                    .map_err(|_| HttpTransportError::RequestFailed)?,
                reqwest::header::HeaderValue::try_from(value)
                    .map_err(|_| HttpTransportError::RequestFailed)?,
            ))
        })
        .collect()
}

pub struct OpenAiCompatibleProvider {
    descriptor: ProviderDescriptor,
    profiles: Vec<ModelProfile>,
    endpoint: String,
    bearer_token: Option<String>,
    transport: Arc<dyn HttpTransport>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        descriptor: ProviderDescriptor,
        profiles: Vec<ModelProfile>,
        base_url: impl Into<String>,
        bearer_token: Option<String>,
    ) -> ProviderResult<Self> {
        Self::with_transport(
            descriptor,
            profiles,
            base_url,
            bearer_token,
            Arc::new(ReqwestTransport::new()),
        )
    }

    pub(crate) fn with_transport(
        descriptor: ProviderDescriptor,
        profiles: Vec<ModelProfile>,
        base_url: impl Into<String>,
        bearer_token: Option<String>,
        transport: Arc<dyn HttpTransport>,
    ) -> ProviderResult<Self> {
        if descriptor.id.trim().is_empty() || descriptor.display_name.trim().is_empty() {
            return Err(ProviderError::InvalidRequest);
        }
        let base_url = base_url.into();
        let base_url = base_url.trim().trim_end_matches('/');
        if base_url.is_empty() || !base_url.starts_with("http") {
            return Err(ProviderError::InvalidRequest);
        }
        for profile in &profiles {
            profile.validate()?;
            if profile.provider_id != descriptor.id {
                return Err(ProviderError::InvalidRequest);
            }
        }
        Ok(Self {
            descriptor,
            profiles,
            endpoint: format!("{base_url}{CHAT_COMPLETIONS_PATH}"),
            bearer_token,
            transport,
        })
    }

    fn validate_request(&self, request: &GenerateRequest) -> ProviderResult<()> {
        if request.model.provider_id != self.descriptor.id
            || !self
                .profiles
                .iter()
                .any(|profile| profile.model_id == request.model.model_id)
            || request.messages.is_empty()
            || request.max_output_tokens == 0
            || request
                .temperature
                .is_some_and(|value| !(0.0..=2.0).contains(&value))
        {
            return Err(ProviderError::InvalidRequest);
        }
        Ok(())
    }

    fn request(&self, request: &GenerateRequest, stream: bool) -> HttpRequest {
        let mut headers = BTreeMap::from([(
            String::from("content-type"),
            String::from("application/json"),
        )]);
        if let Some(token) = &self.bearer_token {
            headers.insert(String::from("authorization"), format!("Bearer {token}"));
        }
        let messages = request
            .messages
            .iter()
            .map(|message| json!({ "role": message.role, "content": message.content }))
            .collect::<Vec<_>>();
        let mut body = json!({
            "model": request.model.model_id,
            "messages": messages,
            "max_tokens": request.max_output_tokens,
            "stream": stream,
        });
        if let Some(temperature) = request.temperature {
            body["temperature"] = json!(temperature);
        }
        HttpRequest::new(self.endpoint.clone(), headers, body)
    }

    fn profile(&self, model_id: &str) -> Option<&ModelProfile> {
        self.profiles
            .iter()
            .find(|profile| profile.model_id == model_id)
    }
}

#[async_trait]
impl AIProvider for OpenAiCompatibleProvider {
    fn descriptor(&self) -> ProviderDescriptor {
        self.descriptor.clone()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.profiles
            .iter()
            .fold(ProviderCapabilities::default(), |mut result, profile| {
                result.streaming |= profile.capabilities.streaming;
                result.embeddings |= profile.capabilities.embeddings;
                result.tools |= profile.capabilities.tools;
                result.vision |= profile.capabilities.vision;
                result.structured_output |= profile.capabilities.structured_output;
                result.prompt_caching |= profile.capabilities.prompt_caching;
                result
            })
    }

    fn list_models(&self) -> ProviderResult<Vec<ModelProfile>> {
        Ok(self.profiles.clone())
    }

    async fn generate(&self, request: GenerateRequest) -> ProviderResult<GenerateResponse> {
        self.validate_request(&request)?;
        let response = self
            .transport
            .post_json(self.request(&request, false))
            .await
            .map_err(|_| ProviderError::ProviderFailure)?;
        if !(200..300).contains(&response.status) {
            return Err(ProviderError::ProviderFailure);
        }
        let payload: ChatCompletionResponse =
            serde_json::from_str(&response.body).map_err(|_| ProviderError::ProviderFailure)?;
        let choice = payload
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .ok_or(ProviderError::ProviderFailure)?;
        Ok(GenerateResponse {
            model: request.model,
            text: choice,
            usage: payload
                .usage
                .map(|usage| ProviderUsage {
                    input_tokens: usage.prompt_tokens,
                    output_tokens: usage.completion_tokens,
                })
                .unwrap_or(ProviderUsage {
                    input_tokens: None,
                    output_tokens: None,
                }),
        })
    }

    async fn stream(&self, request: GenerateRequest) -> ProviderResult<ProviderStream> {
        self.validate_request(&request)?;
        if !self
            .profile(&request.model.model_id)
            .is_some_and(|profile| profile.capabilities.streaming)
        {
            return Err(ProviderError::UnsupportedCapability {
                capability: "streaming".into(),
            });
        }
        let chunks = self
            .transport
            .post_stream(self.request(&request, true))
            .await
            .map_err(|_| ProviderError::ProviderFailure)?;
        Ok(Box::pin(SseResponseStream::new(chunks)))
    }
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChunk {
    choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChoice {
    delta: ChatDelta,
}

#[derive(Debug, Deserialize)]
struct ChatDelta {
    content: Option<String>,
}

struct SseParser {
    buffer: Vec<u8>,
    event_data: Vec<String>,
    done: bool,
}

impl SseParser {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            event_data: Vec::new(),
            done: false,
        }
    }

    fn feed(&mut self, chunk: &[u8]) -> ProviderResult<Vec<StreamEvent>> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();
        while let Some(newline) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line = self.buffer.drain(..=newline).collect::<Vec<_>>();
            let line = std::str::from_utf8(&line)
                .map_err(|_| ProviderError::ProviderFailure)?
                .trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                if let Some(event) = self.finish_event()? {
                    events.push(event);
                }
            } else if line.starts_with(':') {
                continue;
            } else if let Some(data) = line.strip_prefix("data:") {
                self.event_data
                    .push(data.strip_prefix(' ').unwrap_or(data).to_string());
            }
        }
        Ok(events)
    }

    fn finish(&mut self) -> ProviderResult<Vec<StreamEvent>> {
        if !self.buffer.is_empty() || !self.event_data.is_empty() {
            return Err(ProviderError::ProviderFailure);
        }
        if self.done {
            Ok(Vec::new())
        } else {
            Err(ProviderError::ProviderFailure)
        }
    }

    fn finish_event(&mut self) -> ProviderResult<Option<StreamEvent>> {
        if self.event_data.is_empty() || self.done {
            self.event_data.clear();
            return Ok(None);
        }
        let data = self.event_data.join("\n");
        self.event_data.clear();
        if data == "[DONE]" {
            self.done = true;
            return Ok(Some(StreamEvent {
                text_delta: String::new(),
                done: true,
            }));
        }
        let chunk: ChatStreamChunk =
            serde_json::from_str(&data).map_err(|_| ProviderError::ProviderFailure)?;
        if let Some(content) = chunk
            .choices
            .first()
            .and_then(|choice| choice.delta.content.clone())
        {
            if !content.is_empty() {
                return Ok(Some(StreamEvent {
                    text_delta: content,
                    done: false,
                }));
            }
        }
        Ok(None)
    }
}

struct SseResponseStream {
    upstream: HttpChunkStream,
    parser: SseParser,
    pending: VecDeque<ProviderResult<StreamEvent>>,
    finished: bool,
}

impl SseResponseStream {
    fn new(upstream: HttpChunkStream) -> Self {
        Self {
            upstream,
            parser: SseParser::new(),
            pending: VecDeque::new(),
            finished: false,
        }
    }
}

impl Stream for SseResponseStream {
    type Item = ProviderResult<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = self.pending.pop_front() {
                return Poll::Ready(Some(event));
            }
            if self.finished || self.parser.done {
                return Poll::Ready(None);
            }
            match self.upstream.as_mut().poll_next(context) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Err(_))) => {
                    self.finished = true;
                    return Poll::Ready(Some(Err(ProviderError::ProviderFailure)));
                }
                Poll::Ready(Some(Ok(chunk))) => match self.parser.feed(&chunk) {
                    Ok(events) => {
                        self.pending.extend(events.into_iter().map(Ok));
                    }
                    Err(error) => {
                        self.finished = true;
                        return Poll::Ready(Some(Err(error)));
                    }
                },
                Poll::Ready(None) => match self.parser.finish() {
                    Ok(events) => {
                        self.finished = true;
                        self.pending.extend(events.into_iter().map(Ok));
                    }
                    Err(error) => {
                        self.finished = true;
                        return Poll::Ready(Some(Err(error)));
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use futures::executor::block_on;
    use futures_util::StreamExt;

    use super::*;
    use crate::provider::{ModelRef, PromptMessage, PromptRole};

    #[derive(Clone)]
    struct FakeTransport {
        json_response: HttpResponse,
        stream_chunks: Vec<Result<Vec<u8>, HttpTransportError>>,
        requests: Arc<Mutex<Vec<HttpRequest>>>,
    }

    #[async_trait]
    impl HttpTransport for FakeTransport {
        async fn post_json(
            &self,
            request: HttpRequest,
        ) -> Result<HttpResponse, HttpTransportError> {
            self.requests.lock().unwrap().push(request);
            Ok(self.json_response.clone())
        }

        async fn post_stream(
            &self,
            request: HttpRequest,
        ) -> Result<HttpChunkStream, HttpTransportError> {
            self.requests.lock().unwrap().push(request);
            let chunks = self.stream_chunks.clone();
            Ok(Box::pin(futures_util::stream::iter(chunks)))
        }
    }

    fn profile(streaming: bool) -> ModelProfile {
        ModelProfile {
            provider_id: "openai-compatible".into(),
            model_id: "writer-small".into(),
            display_name: "Writer Small".into(),
            context_window_tokens: 4096,
            default_output_tokens: 512,
            strengths: vec!["prose".into()],
            weaknesses: Vec::new(),
            strategy: Vec::new(),
            tier: crate::provider::ModelTier::Medium,
            capabilities: ProviderCapabilities {
                streaming,
                ..ProviderCapabilities::default()
            },
        }
    }

    fn request() -> GenerateRequest {
        GenerateRequest {
            model: ModelRef {
                provider_id: "openai-compatible".into(),
                model_id: "writer-small".into(),
            },
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: "Write a scene".into(),
            }],
            max_output_tokens: 64,
            temperature: Some(0.7),
        }
    }

    fn provider(transport: Arc<FakeTransport>) -> OpenAiCompatibleProvider {
        OpenAiCompatibleProvider::with_transport(
            ProviderDescriptor {
                id: "openai-compatible".into(),
                display_name: "OpenAI-compatible".into(),
            },
            vec![profile(true)],
            "  https://example.test/v1/  ",
            Some("secret-token".into()),
            transport,
        )
        .unwrap()
    }

    fn chunk(value: &str) -> Result<Vec<u8>, HttpTransportError> {
        Ok(value.as_bytes().to_vec())
    }

    #[test]
    fn contract_types_are_present() {
        let _ = std::any::type_name::<HttpRequest>();
        let _ = std::any::type_name::<OpenAiCompatibleProvider>();
    }

    #[test]
    fn validates_before_transport_and_normalizes_endpoint() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: r#"{"choices":[{"message":{"content":"ok"}}]}"#.into(),
            },
            stream_chunks: Vec::new(),
            requests: requests.clone(),
        });
        let provider = provider(transport);
        let mut invalid = request();
        invalid.max_output_tokens = 0;
        assert_eq!(
            block_on(provider.generate(invalid)),
            Err(ProviderError::InvalidRequest)
        );
        assert!(requests.lock().unwrap().is_empty());

        let response = block_on(provider.generate(request())).unwrap();
        assert_eq!(response.text, "ok");
        let requests = requests.lock().unwrap();
        assert_eq!(requests[0].url, "https://example.test/v1/chat/completions");
        assert_eq!(requests[0].headers["authorization"], "Bearer secret-token");
        assert_eq!(requests[0].body["model"], "writer-small");
        assert_eq!(requests[0].body["max_tokens"], 64);
        assert!((requests[0].body["temperature"].as_f64().unwrap() - 0.7).abs() < 0.0001);
    }

    #[test]
    fn maps_http_and_malformed_response_to_safe_failure() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 401,
                body: "secret response body".into(),
            },
            stream_chunks: Vec::new(),
            requests,
        });
        assert_eq!(
            block_on(provider(transport).generate(request())),
            Err(ProviderError::ProviderFailure)
        );

        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: "not json".into(),
            },
            stream_chunks: Vec::new(),
            requests,
        });
        assert_eq!(
            block_on(provider(transport).generate(request())),
            Err(ProviderError::ProviderFailure)
        );
    }

    #[test]
    fn parses_split_sse_chunks_and_done_event() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: String::new(),
            },
            stream_chunks: vec![
                chunk("data: {\"choices\":[{\"delta\":{\"content\":\"Hel"),
                chunk("lo\"}}]}\n\n"),
                chunk("data: {\"choices\":[{\"delta\":{}}]}\n\n"),
                chunk("data: [DONE]\n\n"),
            ],
            requests,
        });
        let events = block_on(async {
            provider(transport)
                .stream(request())
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        });
        assert_eq!(
            events,
            vec![
                Ok(StreamEvent {
                    text_delta: "Hello".into(),
                    done: false,
                }),
                Ok(StreamEvent {
                    text_delta: String::new(),
                    done: true,
                }),
            ]
        );
    }

    #[test]
    fn preserves_utf8_characters_split_across_transport_chunks() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: String::new(),
            },
            stream_chunks: vec![
                Ok(b"data: {\"choices\":[{\"delta\":{\"content\":\"caf\xc3".to_vec()),
                Ok(b"\xa9\"}}]}\n\n".to_vec()),
                chunk("data: [DONE]\n\n"),
            ],
            requests,
        });
        let events = block_on(async {
            provider(transport)
                .stream(request())
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        });
        assert_eq!(events[0].as_ref().unwrap().text_delta, "café");
    }

    #[test]
    fn joins_multiline_sse_data_at_event_boundary() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: String::new(),
            },
            stream_chunks: vec![
                chunk("data: {\"choices\":\r\n"),
                chunk("data: [{\"delta\":{\"content\":\"Hi\"}}]}\r\n\r\n"),
                chunk(": keep-alive\r\n\r\n"),
                chunk("data: [DONE]\r\n\r\n"),
            ],
            requests,
        });
        let events = block_on(async {
            provider(transport)
                .stream(request())
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        });
        assert_eq!(events[0].as_ref().unwrap().text_delta, "Hi");
        assert!(events[1].as_ref().unwrap().done);
    }

    #[test]
    fn reports_truncated_stream_without_done_event() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: String::new(),
            },
            stream_chunks: vec![chunk(
                "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
            )],
            requests,
        });
        let events = block_on(async {
            provider(transport)
                .stream(request())
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
        });
        assert_eq!(events[0].as_ref().unwrap().text_delta, "partial");
        assert_eq!(events.last(), Some(&Err(ProviderError::ProviderFailure)));
    }

    #[test]
    fn rejects_streaming_when_profile_does_not_support_it() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(FakeTransport {
            json_response: HttpResponse {
                status: 200,
                body: String::new(),
            },
            stream_chunks: Vec::new(),
            requests: requests.clone(),
        });
        let provider = OpenAiCompatibleProvider::with_transport(
            ProviderDescriptor {
                id: "openai-compatible".into(),
                display_name: "OpenAI-compatible".into(),
            },
            vec![profile(false)],
            "https://example.test",
            None,
            transport,
        )
        .unwrap();
        assert!(matches!(
            block_on(provider.stream(request())),
            Err(ProviderError::UnsupportedCapability { capability }) if capability == "streaming"
        ));
        assert!(requests.lock().unwrap().is_empty());
    }

    #[test]
    fn request_debug_does_not_expose_secret_or_body() {
        let request = HttpRequest::new(
            "https://user:password@example.test/chat?api_key=query-secret".into(),
            BTreeMap::from([(String::from("authorization"), String::from("Bearer secret"))]),
            json!({ "messages": [{ "content": "secret prompt" }] }),
        );
        let debug = format!("{request:?}");
        assert!(!debug.contains("secret"));
        assert!(!debug.contains("password"));
        assert!(!debug.contains("query-secret"));
        assert!(!debug.contains("prompt"));
    }

    #[test]
    fn rejects_invalid_header_values_before_network_execution() {
        let headers = BTreeMap::from([(String::from("authorization"), String::from("bad\nvalue"))]);
        assert!(matches!(
            super::headers(&headers),
            Err(HttpTransportError::RequestFailed)
        ));
    }
}
