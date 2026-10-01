# WEBNOVEL AI STUDIO Phase 3 Provider Abstraction and Context Compiler Design

## Amaç

Phase 3, canonical memory ile gelecekteki AI çağrıları arasında provider bağımsız
ve test edilebilir bir sınır kurar. Bu slice gerçek bir sağlayıcıya bağlanmaz;
uygulamanın model seçimini, provider yeteneklerini ve göreve göre sınırlı context
üretimini aynı core domain içinde tanımlar.

Başarı ölçütü, bir provider adapter'ı eklenmeden önce bile şu akışların typed ve
deterministik olarak test edilebilmesidir:

1. Provider ve model profilleri registry üzerinden listelenir.
2. Bir modelin yetenekleri desteklenmeyen işlemleri açıkça bildirir.
3. Context Compiler yalnızca istenen structured memory kayıtlarını seçer.
4. Model context penceresi ve output rezervi, hardcoded toplam geçmiş yerine
   hesaplanabilir bir input bütçesi oluşturur.
5. Bu sınırlar Tauri komutlarına provider-specific kod sızdırmadan taşınır.

## Kapsam

### Dahil

- Rust core içinde provider descriptor, capability matrix ve model profile tipleri
- `AIProvider` ortak async interface'i ve unsupported capability davranışı
- Bellek içi `ProviderRegistry`; provider/model discovery ve duplicate kontrolü
- Provider-independent `GenerateRequest`, `GenerateResponse` ve güvenli provider
  hata tipleri
- Deterministik, injectable token estimator ve `ContextBudget`
- Project, seçilmiş Character/CharacterState ve çağrı tarafından verilen geçici
  working-memory block'lerinden `CompiledContext` üretimi
- Öncelik sırası, truncation/omission metadata'sı ve context kullanım bilgisi
- Registry ve Context Compiler için mock provider, unit ve integration testleri
- Tauri command adapter'ları: provider/model listesi ve context derleme preview'sı
- Provider/Context modüllerinin architecture dokümantasyonu

### Hariç

- OpenAI, Anthropic, Gemini, OpenRouter veya başka uzak provider adapter'ı
- Ollama, llama.cpp, GGUF veya başka local model adapter'ı
- API key, secure credential store veya provider credential sync
- Narrative Orchestrator, quality mode execution veya logical-agent workflow'u
- Semantic/vector retrieval, relationship graph ve embedding index'i
- Chat/manuscript entity'leri, kalıcı working memory veya tüm sohbet geçmişi
- Streaming network çağrısı, retry/rate-limit politikası ve prompt caching
- Model profillerini SQLite'a kaydetme veya kullanıcıya profile editor sunma

## Tasarım kararları

### Provider boundary

Provider-specific HTTP/SDK kodu `src-tauri/src/provider` dışına çıkmaz ve domain
entity'lerine provider id veya raw response alanı eklemez. Ortak interface async
olur; gerçek adapter'lar daha sonraki fazda aynı sözleşmeyi uygular.

`AIProvider` şu operasyonları tanımlar:

- descriptor ve capability metadata'sı
- `list_models`
- `generate`
- `stream` ve `embed` için explicit unsupported sonucu

Phase 3'te `stream` ve `embed` için ortak typed request/response sözleşmesi ve
capability kontrolü bulunur, ancak network adapter'ı bulunmaz. Böylece UI veya
orchestrator bir özelliği varsaymak yerine capability sonucunu kontrol eder.

Provider hataları (`provider_not_found`, `model_not_found`, `unsupported_capability`,
`invalid_request`, `provider_failure`) güvenli kodlara çevrilir. Raw HTTP body,
API key, prompt veya provider stack trace hata mesajına yazılmaz.

### ModelProfile ve registry

`ModelProfile` runtime metadata'dır; SQLite canonical state değildir. Profile şu
alanları taşır:

- provider id ve model id
- display name
- context window token sayısı
- varsayılan output rezervi
- strengths, weaknesses ve strategy etiketleri
- capability override'ları

`ProviderRegistry` provider instance'larını provider id ile tutar. Aynı id ikinci
kez kaydedilmez; model seçimi provider id + model id çiftiyle çözülür. Registry
`Arc<dyn AIProvider>` değerleriyle provider adapter'ını Tauri state'ten ayırır.
Phase 3 runtime'ında registry boş olabilir; mock provider yalnızca testlerde
kullanılır. Böylece gerçek provider varmış gibi davranan sahte production akışı
oluşmaz.

### Context kaynağı ve seçim

Context Compiler SQL'e veya Tauri'ye doğrudan bağlanmaz. `ContextSource` trait'i
Project ve Character servislerinin typed çıktısını sağlar. Production adapter'ı
mevcut `ProjectService` ve `CharacterService` üzerinden çalışır; repository SQL'i
Context Compiler'a sızmaz.

Bir `ContextCompileRequest` şu bilgileri açıkça taşır:

- project id ve görev türü
- seçilmiş model profili
- system instructions
- seçilecek character id'leri
- CharacterState dahil etme seçeneği
- caller tarafından verilen geçici working-memory blocks
- output rezervi ve güvenlik payını içeren bütçe override'ı

Character id listesi boşsa karakter kayıtları otomatik eklenmez. Compiler bütün
Project Memory'yi veya geçmiş chat'i kendiliğinden yüklemez. Seçilen bir id
bulunamazsa işlem güvenli `not_found` hatasıyla durur.

### Context formatı ve bütçe

Compiler sonucu provider'a özel prompt string'i değil, sıralı typed block'lar ve
ölçüm metadata'sı olan `CompiledContext` değeridir. Block türleri en azından
`system`, `project`, `character`, `character_state` ve `working_memory` olur.

Öncelik sırası şöyledir:

1. system instructions
2. project core
3. seçilmiş character kayıtları
4. seçilmiş CharacterState kayıtları
5. working-memory blocks

Model profile'ın `context_window_tokens` değeri ve output rezervi toplam input
bütçesini belirler. Varsayılan estimator Unicode karakterlerini muhafazakâr bir
`ceil(chars / 4)` hesabıyla tahmin eder; estimator trait olduğu için gerçek
tokenizer ileride değiştirilebilir. Compiler önce yüksek öncelikli block'ları
korur, düşük öncelikli block'ları bütçeye sığacak şekilde kırpar veya omission
metadata'sına taşır. Hiçbir block sessizce kaybolmaz.

Context sonucu şu bilgileri verir:

- dahil edilen block'lar ve source id'leri
- tahmini input token toplamı
- izin verilen input budget
- kırpılan block ve nedenleri
- model id ve görev

Bu değer provider çağrısını çalıştırmaz; yalnızca provider'a gönderilecek
minimum ve denetlenebilir context'i hazırlar.

## Modül ve veri akışı

```text
Tauri command / future orchestrator
        |
        v
  ContextCompileRequest -----> ContextCompiler
        |                            |
        v                            v
  ContextSource                 CompiledContext
  (ProjectService +             (typed blocks + budget metadata)
   CharacterService)

ProviderRegistry ---> AIProvider ---> future adapter (Phase 4+)
       |
       +--> ProviderDescriptor / ModelProfile / capabilities
```

Önerilen Rust modülleri:

- `src-tauri/src/provider/mod.rs`: provider domain types, trait, errors,
  registry ve mock provider test helper'ı
- `src-tauri/src/context/mod.rs`: request, source, budget, estimator ve compiler
- `src-tauri/src/commands.rs`: typed provider/model listesi ve context preview
  adapter'ları
- `src-tauri/src/lib.rs`: AppState'e registry ve context service bağlanması

Frontend yalnızca `apps/client/src/types/provider.ts`,
`apps/client/src/types/context.ts` ve `apps/client/src/lib/commands.ts` typed
fonksiyonlarını görür. Phase 3'te bu veriler için yeni bir kullanıcı ekranı
gerekmez; sonraki chat UI bu komutları kullanır.

## Hata ve güvenlik politikası

- Provider registry bulunmayan provider/model için güvenli not-found kodu döner.
- Capability false ise operasyon başlamadan `unsupported_capability` döner.
- Context request'teki boş system instructions veya negatif/zero budget
  validation hatasıdır.
- Raw prompt, context block içeriği ve credential değerleri loglanmaz.
- Registry profile'ları credential taşımaz; API key yönetimi sonraki fazın
  secure storage sınırında kalır.
- Context Compiler canonical memory yazmaz. Herhangi bir AI sonucu daha sonra
  proposal/revision akışına girmek zorundadır.

## Test stratejisi

### Rust unit testleri

- capability ve model profile serde/validation
- registry duplicate provider/model ve model resolution
- mock provider generate ve unsupported stream/embed davranışı
- token estimator ve bütçe hesaplama
- block önceliği, truncation ve omission metadata'sı
- compiler'ın seçilmemiş karakteri ve otomatik chat geçmişini dahil etmemesi

### Rust integration/command testleri

- in-memory SQLite üzerinde Project/Character/CharacterState kaynaklarının
  context'e seçilerek derlenmesi
- provider/model listesi command argüman ve safe error sözleşmesi
- context preview komutunun provider-specific SQL veya secret açığa
  çıkarmaması

### Frontend testleri

- provider/model ve context typed command'larının snake_case payload'ları
- provider_not_found, model_not_found ve unsupported_capability mapping'i
- JSON response tiplerinin Rust serde şekliyle uyumu

Phase 3 sonunda mevcut frontend format/typecheck/lint/test/build ve Rust
format/clippy/test matrisi yeniden çalıştırılır. Native Tauri build, mevcut
cloud imajındaki `glib` eksikliği nedeniyle yine environment-limited olabilir;
bu durum ürün kodu hatası olarak yorumlanmaz ve `docs/CLOUD_DEVELOPMENT.md`'de
raporlanır.

## Faz sınırı ve sonraki adım

Bu tasarım provider adapter'ı veya orchestrator çalıştırmaz. Bir sonraki faz,
önce tek bir mock/HTTP adapter ile capability ve streaming sözleşmesini gerçek
çalışan bir provider'a bağlayabilir; ardından API key secure storage ve model
routing politikaları ayrı bir onaylı slice olarak eklenebilir.
