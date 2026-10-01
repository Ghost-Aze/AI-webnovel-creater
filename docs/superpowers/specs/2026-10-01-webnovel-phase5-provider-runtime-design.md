# WEBNOVEL AI STUDIO Phase 5 Provider Runtime Design

## Amaç

Phase 5, Phase 4 adapterını uygulama çalışma zamanında kontrollü biçimde
kaydetme ve kaldırma sınırına bağlar. Provider credential'ı yalnızca process
memory'de tutulur; SQLite, revision log, diagnostics ve sync hiçbir secret
değeri görmez.

Bu faz platformun Windows Credential Manager veya Android Keystore
entegrasyonunu taklit etmez. Bunun yerine ileride bu backend'lerle değiştirile-
cek `CredentialStore` trait'i ve test edilebilir bir `EphemeralCredentialStore`
sağlar.

## Kapsam

- `CredentialStore` trait'i
- Process memory ile sınırlı ephemeral credential store
- `ProviderRuntime` configure/remove lifecycle'ı
- OpenAI-compatible provider'ın credential reference ile oluşturulması
- Provider registry unregister işlemi
- Tauri `provider_configure` ve `provider_remove` commands
- Frontend typed setup/remove payload'ları
- Secret redaction, rollback, duplicate provider ve lifecycle testleri

## Kapsam dışı

- Credential'ın SQLite, cloud sync veya revision log'a yazılması
- Windows Credential Manager veya Android Keystore production backend'i
- Provider setup ekranı ve kullanıcı ayarları UI'ı
- Model routing, quality mode ve orchestrator
- Retry, rate limit, failover ve health monitoring
- Yeni provider wire formatları

## Credential sınırı

```text
ProviderConfigureInput (IPC, secret yalnızca request içinde)
        |
        v
ProviderRuntime -> CredentialStore.put(credential_id, secret)
        |
        v
OpenAiCompatibleProvider(secret loaded in memory)
        |
        v
ProviderRegistry.register(adapter)
```

`ProviderConfigureInput` response olarak geri dönmez. Input'ın `Debug`
uygulaması secret'ı, URL credential'larını ve model payload'ını göstermez.
`ProviderRuntime` configure başarısız olursa credential store rollback yapılır.
Duplicate provider veya invalid model registry'yi ve store'u değiştirmeden
hata döndürür.

Ephemeral store `Arc<RwLock<BTreeMap<CredentialId, SecretValue>>>` kullanır.
`SecretValue` serialize edilemez ve `Debug` çıktısı redacted'dır. Bu store
uygulama yeniden başlatıldığında boşalır; bu davranış kasıtlıdır.

## Runtime lifecycle

- `configure`: input validation, secret store write, adapter construction,
  registry register. Herhangi bir aşama başarısızsa store rollback.
- `remove`: registry'den provider'ı çıkarır ve credential reference'ı store'dan
  siler. Bilinmeyen provider id güvenli not-found sonucu verir.
- `list`/`model_list`/`generate`: Phase 4 command helper'ları aynı registry
  instance'ını kullanmaya devam eder.
- Startup: runtime registry ve ephemeral store boştur; otomatik provider
  registration yapılmaz.

## Komut sözleşmesi

`provider_configure` yalnızca descriptor, base URL, model profiles, credential
id ve credential value alır; response descriptor döndürür. `provider_remove`
provider id alır ve kaldırılan descriptor'ı döndürür. Secret hiçbir response,
log veya error detail'de görünmez.

Frontend bu komutları typed wrapper olarak sunar ama Phase 5'te setup formu
render etmez. Böylece UI tasarımı platform secure store kararıyla birlikte
ayrı bir fazda yapılabilir.

## Test stratejisi

### Rust

- secret `Debug`/serde yoluna sızmaması
- blank credential id/value validation
- configure success ve registry/model discovery
- duplicate provider rollback
- invalid profile rollback
- remove provider + credential deletion
- unknown remove safe error
- restart-equivalent yeni ephemeral store'un boş olması

### Frontend

- snake_case configure/remove payload'ları
- response tipleri
- safe provider error mapping; credential değerinin hata mesajına girmemesi

## Kabul kriterleri

- Provider configure/remove gerçek adapter registry'sini paylaşır.
- Secret değeri kalıcı storage veya sync katmanına girmez.
- Configure başarısızlıkları kısmi runtime state bırakmaz.
- Phase 0–4 test/lint matrisi bozulmaz.
- Native secure store backend'i sonraki platform fazına açık bir trait ile
  bırakılır.

## Sonraki faz

Windows Credential Manager ve Android Keystore için `CredentialStore`
implementasyonları, setup UI ve kullanıcı hesap/backup politikası ayrı bir
onaylı fazda ele alınır.
