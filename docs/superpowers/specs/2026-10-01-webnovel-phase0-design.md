# WEBNOVEL AI STUDIO Phase 0 Tasarım Spesifikasyonu

## Amaç

Phase 0, WEBNOVEL AI STUDIO için Windows ve Android hedeflerini taşıyabilecek, yerel SQLite kalıcılığı olan ve temel proje yaşam döngüsünü destekleyen çalışır bir iskelet kurar. Kullanıcı uygulamayı açabilir, projeleri listeleyebilir, yeni proje oluşturabilir, bir projeyi açabilir, yeniden adlandırabilir ve arşivleyebilir. AI sağlayıcıları, anlatı orkestrasyonu, gelişmiş hafıza, semantic retrieval ve senkronizasyon bu fazın dışındadır.

## Kapsam ve başarı ölçütleri

- React + TypeScript istemcisi Tauri 2 kabuğu içinde çalışır.
- Rust çekirdeği SQLite bağlantısını ve migration çalıştırmayı yönetir.
- Proje kayıtları uygulama yeniden başlatıldığında korunur.
- Aktif ve arşivlenmiş projeler açıkça ayrılır; arşivleme geri döndürülebilir bir durum değişikliğidir.
- Proje listesi, yeni proje modalı, proje çalışma alanı ve temel navigation çalışır.
- UI veritabanına doğrudan erişmez; typed Tauri command katmanını kullanır.
- Rust ve TypeScript için test, format ve lint komutları tanımlıdır.
- Mimari ve yerel geliştirme adımları README ve `docs/ARCHITECTURE.md` içinde açıklanır.

## Repository ve bulut çalışma modeli

GitHub repository (`Ghost-Aze/AI-webnovel-creater`) kaynak kodun canonical kaynağıdır. Local checkout ve Codex Cloud environment aynı repository’den başlar; geliştirme görevleri feature branch veya izole worktree üzerinde yürütülür. `main` dalı yalnızca gözden geçirilmiş ve doğrulanmış değişiklikleri alır.

Codex Cloud environment içinde repository, Rust/Node/Tauri bağımlılıkları, test komutları ve Android build gereksinimleri tanımlanır. Environment bir kez yayınlandıktan sonra yeni görevler web, masaüstü veya mobil Codex istemcisinden başlatılabilir. Aynı task yeniden açılarak farklı cihazdan devam edilir; mobil cihaz repository’nin kendisini çalıştırmaz, Cloud environment üzerinde çalışan görevi yönlendirir ve sonucu inceler.

Yerel SQLite verisi kaynak repository’ye commit edilmez. Uygulama verisi cihazın uygulama veri dizininde tutulur. Phase 0 senkronizasyonu proje veritabanı için değil, kaynak kod geliştirme akışı içindir; cihazlar arası novel verisi senkronizasyonu sonraki fazdır.

## Mimari

```text
apps/client       React + TypeScript + Vite UI
src-tauri         Tauri commands, Rust application services, SQLite adapter
packages/domain   UI'nin ihtiyaç duyduğu küçük, provider-bağımsız domain tipleri
packages/shared   Ortak sonuç/hata ve tarih yardımcıları (yalnızca kullanılanlar)
migrations        Sıralı SQLite migration dosyaları
docs              Mimari, ürün ve cloud setup belgeleri
```

İstemci yalnızca `src-tauri` tarafından sunulan komutları çağırır. Rust tarafında command handler, application service ve repository sınırları bulunur. SQLite ayrıntısı repository arkasında kalır; ileride cloud/sync katmanı eklenirken domain ve UI değişmeden yeni adapter eklenebilir.

## Domain ve veritabanı

Phase 0'ın tek canonical domain nesnesi `Project`'tir:

```text
id          TEXT PRIMARY KEY (UUID)
name        TEXT NOT NULL
description TEXT NOT NULL DEFAULT ''
status      TEXT NOT NULL CHECK (status IN ('active', 'archived'))
created_at  TEXT NOT NULL (UTC ISO-8601)
updated_at  TEXT NOT NULL (UTC ISO-8601)
```

İlk migration `projects` tablosunu ve migration metadata tablosunu oluşturur. `updated_at`, isim veya açıklama değiştiğinde güncellenir. Listeleme varsayılan olarak aktif projeleri güncellenme tarihine göre, yeniden eskiye sıralar; arşivli kayıtlar ayrı filtreyle görülebilir.

## Uygulama arayüzü

Tauri command sözleşmeleri:

- `project_create(input: CreateProjectInput) -> Project`
- `project_list(filter: ProjectListFilter) -> Vec<Project>`
- `project_get(id: ProjectId) -> Project`
- `project_update(id: ProjectId, input: UpdateProjectInput) -> Project`
- `project_archive(id: ProjectId) -> Project`

Geçersiz kimlik, boş isim ve SQLite hataları typed application error olarak dönüştürülür. UI bunları kullanıcı mesajına çevirir; ham stack trace göstermez. Arşivlenmiş kayıt üzerinde varsayılan update işlemi reddedilir ve açık bir hata döner.

UI route'ları `/projects` ve `/projects/:projectId` olur. Ana shell sol navigation, merkez içerik, sağ context placeholder ve alt status bar içerir. Sağ paneldeki karakter/timeline/memory alanları bu fazda yalnızca placeholder'dır. Mobil görünüm aynı bilgi mimarisini responsive drawer ve alt navigation ile taşır.

## Hata yönetimi ve loglama

Rust tarafında domain/application/storage ayrımı olan serializable error türleri kullanılır. Beklenen kullanıcı hataları `warning`, beklenmeyen teknik hatalar `error` seviyesinde loglanır. Loglar credential, API key veya proje içeriğini yazmaz. Frontend'de command çağrıları tek bir typed client yardımcı modülünden geçer; loading, empty ve error durumları her ekran tarafından ele alınır.

## Test ve doğrulama

- Rust unit testleri: project validation, status filtreleme, archive davranışı ve timestamp güncellemesi.
- Rust integration test temeli: geçici SQLite veritabanında migration + create/list/update/archive akışı.
- Frontend testleri: proje listesi boş/dolu durumu, create modal doğrulaması ve command error gösterimi.
- CI'ye uygun komutlar: Rust format/check/test, TypeScript typecheck, lint ve frontend test.
- Windows build komutu belgelenir. Android için Tauri project scaffold'ı hazırlanır; Android SDK/NDK yoksa build sonucu açıkça `environment-limited` olarak raporlanır.
- Codex Cloud setup adımları ve beklenen doğrulama komutları `README.md` ve `docs/ARCHITECTURE.md` içinde bulunur.

## Güvenlik ve kapsam sınırları

Phase 0 provider API anahtarı saklamaz, ağ çağrısı yapmaz ve novel verisi için cloud sync başlatmaz. SQLite dosyası uygulama veri dizininde tutulur. GitHub’a SQLite runtime verisi, API anahtarı veya yerel credential commit edilmez. AI prompt'ları, memory entity'leri ve provider adapter'ları bu fazda oluşturulmaz; yalnızca sonraki fazların eklenmesini engellemeyecek modül sınırları bırakılır.

