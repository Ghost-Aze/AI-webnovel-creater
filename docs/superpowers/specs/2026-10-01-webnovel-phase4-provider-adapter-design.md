# WEBNOVEL AI STUDIO Phase 4 Provider Adapter Design

## Amaç

Phase 4, Phase 3'te tanımlanan provider sözleşmesini gerçek bir
OpenAI-compatible HTTP adapterına bağlar. Adapter, chat completion isteğini
wire formatına çevirir, güvenli bir sonuç üretir ve streaming SSE olaylarını
typed `StreamEvent` değerlerine dönüştürür.

Bu faz provider bağımsız domain sınırını korur. Adapter yalnızca provider
katmanında bulunur; Context Compiler, canonical memory ve React bileşenleri
OpenAI wire formatını bilmez.

## Kapsam

- `OpenAiCompatibleProvider` adapterı
- Reqwest tabanlı production HTTP transportı
- Testlerde kullanılacak deterministic fake transport
- Chat completion request/response JSON codec'i
- SSE streaming parserı
- Bellekte verilen bearer credential; kalıcı saklama yok
- Provider registry üzerinden async `provider_generate` command sınırı
- Frontend için typed generate request/response command wrapperı
- HTTP, JSON, stream parse, capability ve güvenli hata testleri

## Kapsam dışı

- OpenAI, Anthropic veya Gemini'ye özel wire formatları
- API key secure storage, hesap ayarları veya cloud sync
- API key'in SQLite'a veya revision log'a yazılması
- Orchestrator, quality mode ve model routing
- Embedding, tools, vision veya structured-output uygulaması
- Chat/manuscript UI
- Streaming verisinin React ekranında gösterilmesi

## Provider sözleşmesi

Phase 3'teki `AIProvider` aynen korunur. Adapter şu davranışları sağlar:

```text
GenerateRequest
  -> OpenAI-compatible JSON POST /chat/completions
  -> GenerateResponse

GenerateRequest (stream)
  -> JSON POST with stream=true
  -> data: {...delta.content...}\n\n
  -> StreamEvent(text_delta, done)
```

Bir `ModelProfile` yalnızca adaptera ait model id'si ile kabul edilir.
`max_output_tokens` pozitif olmalı, temperature 0 ile 2 arasında bulunmalı ve
mesaj listesi boş olmamalıdır. Bu kontroller HTTP çağrısından önce yapılır.

## Transport sınırı

Adapter doğrudan `reqwest::Client` kullanmaz. `HttpTransport` şu iki işlemi
sağlar:

- JSON response döndüren `post_json`
- byte/text chunk stream döndüren `post_stream`

Production `ReqwestTransport` bu trait'i uygular. Unit testleri HTTP server
veya ağ gerektirmeden `FakeTransport` ile request gövdesini ve header'ları
inceleyebilir.

Transport request tipi secret içerebilir ancak `Debug`, `Serialize` ve log
çıktısı üretmez. Bearer token yalnızca adapter instance'ının belleğinde
tutulur; registry profile'larına veya SQLite'a girmez.

## HTTP ve hata politikası

- Base URL trim edilir, son slash normalize edilir ve endpoint
  `/chat/completions` olarak eklenir.
- `Authorization` yalnızca credential verilmişse eklenir.
- 2xx dışı response, JSON parse hatası, boş choices veya stream parse hatası
  `ProviderError::ProviderFailure` olarak dışarı çıkar.
- HTTP status, response body, request promptu ve token hata detayına/log'a
  taşınmaz.
- Provider/model/capability/request validation hataları Phase 3 güvenli kod
  eşlemesini korur.

## SSE parser

Parser chunk sınırlarının `data:` satırını bölebileceğini varsayar. Buffer'da
tam satırlar biriktirilir; boş satır event'i sonlandırır. Yalnızca
`choices[0].delta.content` okunur. `data: [DONE]` tek bir `done=true` olayı
üretir ve stream'i bitirir. Content içermeyen role veya finish reason olayları
sessizce atlanır.

## Command boundary

`provider_generate` request'i registry'de model çözümleyip provider'ın
`generate` metodunu çağırır. Command async olur ve yalnızca typed
`GenerateResponse` döndürür. Stream IPC endpoint'i bu fazda eklenmez; typed
provider stream'i orchestrator için Rust sınırında hazırdır.

Registry başlangıçta boş kalır. Credential secure storage fazı gelmeden
uygulama açılışında gerçek provider otomatik kaydedilmez.

## Test stratejisi

### Rust

- request validation'ın HTTP çağrısından önce durması
- endpoint, model, messages, temperature ve authorization wire şekli
- başarılı JSON response ve usage mapping'i
- non-2xx, malformed JSON ve empty choices safe error mapping'i
- stream chunk sınırları, `[DONE]`, boş delta ve malformed event davranışı
- capability ve model mismatch davranışı
- fake transport ile secret'ın log/Debug/serde yoluna sızmaması
- command helper'ın registry resolve ve provider çağrısı

### Frontend

- `provider_generate` snake_case payload'ı
- response tiplerinin serde şekli
- provider error mapping'inin raw HTTP body göstermemesi

## Kabul kriterleri

- OpenAI-compatible adapter gerçek `reqwest` transport ile derlenir.
- Fake transport testleri ağ olmadan tüm request/response ve stream yollarını
  kapsar.
- `provider_generate` typed command sınırından geçer.
- Phase 0–3 test ve lint matrisi bozulmaz.
- API key kalıcı depolama veya sync kapsamına girmez.
- Native Tauri build cloud `glib` sınırı nedeniyle çalışmazsa bu durum açıkça
  raporlanır; headless Rust kontrolleri başarılı olmalıdır.

## Sonraki faz

Secure credential storage, provider setup UI, gerçek runtime registration ve
model routing ayrı bir onaylı fazdır.
