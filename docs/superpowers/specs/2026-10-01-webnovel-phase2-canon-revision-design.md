# WEBNOVEL AI STUDIO Phase 2 Canon and Revision Design

## Amaç

Bu slice, Project Memory içindeki kabul edilmiş değişiklikleri güvenli ve
izlenebilir hale getirir. `Character` ve `CharacterState` güncel canonical
snapshot olarak kalır; her mutation immutable revision kaydı üretir. Tasarım,
ileride AI önerilerinin canonical veriye doğrudan yazılmasını engelleyecek
proposal sınırını da kurar.

## Kapsam

### Dahil

- Generic `memory_revisions` tablosu ve repository/service altyapısı
- Generic `memory_proposals` tablosu ve provider bağımsız lifecycle işlemleri
- `Character` ve `CharacterState` revision entegrasyonu
- `CANON` ve `LOCKED_CANON` canonical durumları
- Optimistic concurrency ile `expected_revision` kontrolü
- History listesi, field-level before/after diff ve restore UI’sı
- Güvenli conflict, locked-canon ve proposal durum hataları
- Rust integration/unit testleri ve frontend component/command testleri

### Hariç

- AI provider veya tool calling
- Memory extraction ve automatic proposal üretimi
- Semantic retrieval veya relationship graph
- Cloud sync, account/auth ve PostgreSQL
- Developer Chat, manuscript editor ve orchestration

## Tasarım kararları

### Snapshot + immutable log

Canonical tablolar hızlı sorgular için güncel snapshot’ı tutar. Her başarılı
canonical mutation aynı SQLite transaction’ı içinde bir `memory_revisions`
satırı oluşturur. Revision satırları değiştirilmez veya silinmez.

Tam event sourcing kullanılmaz. Canonical state revision’lardan yeniden
oluşturulmaz; revision log audit, diff, restore ve gelecekteki sync geçmişi
için kullanılır.

### Canon durumu ve proposal ayrımı

Canonical snapshot’larda yalnızca `canon` veya `locked_canon` bulunur.
`draft` veriler ayrı `memory_proposals` satırlarında tutulur ve normal
Character/CharacterState sorgularına girmez. Bir proposal promote edildiğinde
ilgili snapshot ve revision aynı transaction içinde güncellenir.

`locked_canon` kayıtlarını değiştirmek için açık bir kullanıcı işlemi gerekir.
AI veya ilerideki otomatik süreçler locked kaydı sessizce değiştiremez.

### Optimistic concurrency

Her update, restore ve canon-status mutation işlemi çağrının okuduğu
`expected_revision` değerini taşır. Veritabanındaki revision farklıysa işlem
uygulanmaz ve `conflict` hatası döner. Bu davranış aynı cihazdaki çift form
gönderimlerini ve gelecekteki cihazlar arası yarışları korur.

## Veri modeli

### `characters` ve `character_states` değişiklikleri

Her iki canonical snapshot tablosuna şu alanlar eklenir:

| Alan           | Politika                                                           |
| -------------- | ------------------------------------------------------------------ |
| `revision`     | İlk snapshot için `0`; her başarılı mutation sonrası bir artırılır |
| `canon_status` | `canon` veya `locked_canon`; varsayılan `canon`                    |

`CharacterState` fiziksel satırı henüz yoksa boş state `revision = 0` ve
`canon_status = canon` ile temsil edilir. İlk state upsert’i revision `1`
oluşturur.

### `memory_revisions`

| Alan             | Politika                                                                          |
| ---------------- | --------------------------------------------------------------------------------- |
| `id`             | UUID text primary key                                                             |
| `project_id`     | `projects(id)` foreign key; proje silinirse revision geçmişi de silinir           |
| `entity_type`    | İlk slice’ta `character` veya `character_state`                                   |
| `entity_id`      | İlgili canonical entity kimliği                                                   |
| `revision`       | Entity başına monoton sayı; `(entity_type, entity_id, revision)` benzersiz        |
| `operation`      | `create`, `update`, `archive`, `restore`, `canon_status` veya `promote`           |
| `actor_type`     | `user`, `ai` veya `system`; ilk UI yalnızca `user` üretir                         |
| `actor_id`       | İleride user/device kimliği için nullable text                                    |
| `base_revision`  | Mutation’ın beklediği önceki revision                                             |
| `previous_value` | Önceki structured snapshot’ın serialized JSON değeri; ilk create için null        |
| `new_value`      | Mutation sonrası structured snapshot; nullable yalnızca silme semantiği eklenirse |
| `source_type`    | `manual`, `proposal`, `restore` veya nullable                                     |
| `source_id`      | Proposal veya kaynak işlem kimliği; nullable                                      |
| `created_at`     | UTC ISO-8601 text                                                                 |

Canonical storage JSON blob değildir. `previous_value` ve `new_value` yalnızca
revision geçmişinin immutable snapshot payload’larıdır; normal sorgular typed
canonical sütunlardan yapılır.

### `memory_proposals`

| Alan                        | Politika                                                    |
| --------------------------- | ----------------------------------------------------------- |
| `id`                        | UUID text primary key                                       |
| `project_id`                | `projects(id)` foreign key                                  |
| `entity_type`               | İlk slice’ta `character` veya `character_state`             |
| `entity_id`                 | Mevcut entity için id; yeni entity proposal’ı için nullable |
| `operation`                 | `create`, `update`, `archive` veya `canon_status`           |
| `payload`                   | Canonical olmayan structured proposal snapshot’ı            |
| `base_revision`             | Proposal’ın üretildiği canonical revision                   |
| `status`                    | `draft`, `accepted` veya `rejected`                         |
| `actor_type` / `actor_id`   | Proposal kaynağı                                            |
| `created_at` / `updated_at` | UTC ISO-8601 text                                           |

Proposal payload’ı typed domain doğrulamasından geçmeden canonical tabloya
yazılamaz. Promote işlemi base revision kontrolü, payload doğrulaması,
canonical update ve revision kaydını tek transaction’da yapar.

## Domain ve service sınırları

- `CanonStatus`, `MemoryEntityType`, `RevisionOperation`, `Revision` ve
  `MemoryProposal` serde ile typed Rust domain değerleri olur.
- Repository SQL ve row mapping’den sorumludur; revision numarası, lock policy,
  proposal transition ve conflict kontrolü service katmanında uygulanır.
- `CharacterService` ve `CharacterState` service mutation’ları generic
  `RevisionService` üzerinden canonical snapshot + revision işlemi yapar.
- UI veya provider katmanı doğrudan SQLite’a erişmez.
- Hatalar güvenli kodlarla (`conflict`, `locked_canon`, `invalid_proposal`)
  döner; SQL ayrıntısı veya payload ham hata metnine girmez.

## Command sözleşmeleri

Mevcut Character mutation komutları `expected_revision` alacak şekilde
genişletilir:

- `character_create(project_id, input) -> Character` (ilk revision `1` üretir)
- `character_update(id, input, expected_revision) -> Character`
- `character_archive(id, expected_revision) -> Character`
- `character_state_update(character_id, input, expected_revision) -> CharacterState`

`character_get` ve `character_list` sonuçları `revision` ve `canon_status`
alanlarını da taşır. `character_state_get` fiziksel state yoksa revision `0`
ve `canon` varsayılanını döndürür.

Generic history ve status komutları:

- `memory_history_list(entity_type, entity_id) -> Vec<Revision>`
- `memory_restore(entity_type, entity_id, revision, expected_revision) -> Revision`
- `memory_set_canon_status(entity_type, entity_id, status, expected_revision) -> Revision`

Provider bağımsız proposal komutları:

- `memory_proposal_create(input) -> MemoryProposal`
- `memory_proposal_list(project_id, filter) -> Vec<MemoryProposal>`
- `memory_proposal_promote(id, expected_revision) -> Revision`
- `memory_proposal_reject(id) -> MemoryProposal`

Entity type ve operation değerleri serbest string olarak kabul edilmez; Rust
enum doğrulamasından geçirilir.

## UI akışı

Character workspace’te seçili Character veya CharacterState için History
kontrolü bulunur.

1. History paneli revision numarası, operation, actor ve timestamp listeler.
2. Bir revision seçildiğinde yalnızca değişen typed alanlar before/after olarak
   gösterilir.
3. Restore eylemi seçilen snapshot’ı forma sessizce kopyalamaz; mevcut formun
   beklenen revision’ı ile yeni `restore` mutation’ı gönderir.
4. Başarılı restore sonrası liste ve current snapshot yenilenir.
5. `locked_canon` durumunda edit, restore ve proposal promote kontrolleri
   devre dışı kalır; kullanıcıya neden gösterilir.
6. Conflict durumunda kullanıcının yerel form girdisi korunur. UI mevcut
   revision’ı yükleme, değişiklikleri yeniden uygulama veya vazgeçme seçenekleri
   sunar.

İlk slice proposal oluşturma için ayrı bir yönetim ekranı gerektirmez; typed
proposal command ve backend lifecycle testleri hazır olur. AI öneri ekranı
provider fazında eklenir.

## Test kabul ölçütleri

### Rust

- `0003` migration’ı idempotent çalışır ve Phase 0/1 verisini korur.
- Character create/update/archive işlemleri revision `1, 2, ...` üretir.
- CharacterState partial update yalnızca verilen alanları değiştirir ve yeni
  revision üretir.
- `expected_revision` uyuşmazlığında hiçbir snapshot veya revision değişmez.
- Locked canonical update, restore ve promote işlemleri reddedilir.
- Restore seçilen eski snapshot’tan yeni revision üretir; geçmiş satırı değişmez.
- Proposal create/list/reject/promote transition kuralları ve promote conflict’i
  test edilir.
- Revision JSON snapshot’ları typed row değerleriyle round-trip eşleşir.

### Frontend

- History paneli revision metadata’sını gösterir.
- Diff yalnızca değişen alanları gösterir.
- Restore komutu seçilen revision ve current expected revision ile çağrılır.
- Locked durumunda mutation kontrolleri kapalıdır.
- Conflict mesajı güvenlidir ve kullanıcının form girdisini silmez.
- Existing project/character tests ile birlikte tam frontend test komutu çalışır.

## Faz sınırı ve sonraki geçiş

Bu spec tamamlandığında uygulama canonical Character memory değişikliklerini
izleyebilir, inceleyebilir ve geri yükleyebilir; henüz AI önerisi üretmez.
Spec’in implementation planı ayrıca yazılıp onaylanmadan kod değişikliği
başlatılmaz. Sonraki provider fazı, proposal payload’larının AI tool’larından
nasıl üretileceğini ve kullanıcı onay ekranını ayrıca tanımlar.
