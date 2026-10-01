# WEBNOVEL AI STUDIO Phase 1 Tasarım Spesifikasyonu

## Amaç

Phase 1, Project Memory’nin ilk yapılandırılmış parçalarını ekler: proje içindeki karakterler ve karakterlerin zamanla değişen durumları. Bu faz, AI sağlayıcısı veya sohbet orkestrasyonu eklemeden canonical belleğin sorgulanabilir veri modelini genişletir.

## Kapsam

- `Character` CRUD ve aktif/arşiv filtreleme
- Her karakter için tekil `CharacterState` kaydı
- SQLite `0002` migration’ı ve foreign-key ilişkileri
- Typed Tauri commands ve TypeScript client
- Proje workspace’inde karakter listesi ve karakter state düzenleme paneli
- Rust integration testleri ve frontend component testleri

Bu fazda provider adapter’ı, memory extraction, semantic search, relationship graph, sync ve AI çağrıları eklenmez.

## Veri modeli

`characters` tablosu `id`, `project_id`, `name`, `summary`, `role`, `status`, `created_at` ve `updated_at` alanlarını taşır. İsim proje içinde benzersizdir. `status` yalnızca `active` veya `archived` olabilir.

`character_states` tablosu bir karaktere bire bir bağlıdır. `current_location`, `physical_condition`, `injuries`, `emotional_state`, `goals`, `beliefs`, `knowledge`, `secrets_known`, `current_conflicts`, `possessions`, `promises`, `last_appearance` ve `current_arc_role` alanları ayrı sütunlardır; tek JSON blob kullanılmaz.

Karakter arşivlendiğinde state korunur ancak varsayılan aktif listeden çıkar. Arşivli karakter güncellenemez. Proje arşivliyse karakter mutation’ları reddedilir.

## Command sözleşmeleri

- `character_create(project_id, input) -> Character`
- `character_list(project_id, filter) -> Vec<Character>`
- `character_get(id) -> Character`
- `character_update(id, input) -> Character`
- `character_archive(id) -> Character`
- `character_state_get(character_id) -> CharacterState`
- `character_state_update(character_id, input) -> CharacterState`

Frontend bu çağrıları tek typed client modülünden yapar. Hatalar Phase 0’daki güvenli `CommandError` modelini kullanır.

## Kabul ölçütleri

- Boş karakter adı reddedilir ve SQLite’a yazılmaz.
- Aynı projede aynı isimle ikinci karakter oluşturulamaz.
- Karakter listesi varsayılan olarak arşivlileri dışlar.
- State kaydı yoksa `character_state_get` boş değerlerle default state döndürür.
- State update upsert davranışıyla tekrar çalıştırılabilir.
- Proje veya karakter arşivliyse mutation güvenli typed hata döndürür.
- Migration tekrar çalıştırıldığında mevcut proje ve karakter verisi korunur.
