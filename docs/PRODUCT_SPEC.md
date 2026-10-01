Sen kıdemli bir yazılım mimarı ve full-stack geliştiricisin. Birlikte uzun vadeli olarak geliştireceğimiz bir uygulamanın temelini kuracağız.

Projenin çalışma adı:

WEBNOVEL AI STUDIO

Bu proje basit bir AI chat uygulaması değildir.

Amaç; Windows PC ve Android mobil cihazlarda çalışan, uzun soluklu webnovel ve kurgu projelerinin geliştirilmesi, planlanması ve yazılması için tasarlanmış, modelden bağımsız, kalıcı hafızalı ve çok sağlayıcılı tam teşekküllü bir AI çalışma ortamı oluşturmaktır.

==================================================
1. TEMEL ÜRÜN VİZYONU
==================================================

Uygulama kullanıcının yapay zekâ ile konuşarak bir webnovel projesi oluşturmasına izin verecek.

Bir proje içerisinde:

- Ana hikâye fikri
- Tür
- Tema
- Ton
- Dünya
- Karakterler
- Karakter ilişkileri
- Olay örgüsü
- Timeline
- Arc yapısı
- Bölüm planları
- Sahne planları
- Canon kuralları
- Yazım kuralları
- Sırlar
- Foreshadowing
- Aktif plot thread'ler
- Karakterlerin güncel durumları

kalıcı proje hafızasında tutulacaktır.

Kullanıcı aynı projeye hem Windows PC üzerinden hem Android cihaz üzerinden erişebilecek ve bir cihazda başladığı işe diğer cihazda devam edebilecektir.

Uygulama tek bir AI sağlayıcısına bağımlı olmayacaktır.

İleride desteklenecek sağlayıcılar:

- OpenAI
- Anthropic
- Google Gemini
- OpenRouter
- diğer OpenAI-compatible API'ler
- Ollama
- llama.cpp
- yerel GGUF modelleri

Ancak uygulamanın ana hafızası hiçbir zaman AI sağlayıcısına ait conversation memory olmayacaktır.

Canonical state tamamen bizim uygulamamızın veritabanında tutulacaktır.

Model değiştirildiğinde proje hafızası kaybolmamalıdır.

==================================================
2. UYGULAMANIN TEMEL MİMARİ PRENSİBİ
==================================================

Sistemin merkezinde üç ana motor bulunacaktır:

1. Novel Memory Engine
2. Narrative Orchestrator
3. Context Compiler

Bunlara ek olarak:

4. Continuity Engine
5. Manuscript Workspace
6. Provider Abstraction Layer
7. Sync Engine

bulunacaktır.

AI modeli uygulamanın kendisi değildir.

AI modelleri değiştirilebilir motorlardır.

Uygulamanın gerçek ürünü:

Novel Memory
+
Narrative Orchestration
+
Context Compilation
+
Continuity Management
+
Writing Workspace

kombinasyonudur.

==================================================
3. HEDEF PLATFORMLAR
==================================================

Öncelikli hedefler:

- Windows
- Android

Aynı ana kod tabanının mümkün olduğunca paylaşılması tercih edilir.

Tercih edilen teknoloji:

Frontend:
- React
- TypeScript

Desktop + Mobile shell:
- Tauri 2

Native/backend core:
- Rust

Local database:
- SQLite

Cloud database:
- PostgreSQL

Backend API:
- Rust tabanlı olabilir
veya mimari gerekçeyle farklı bir çözüm kullanılabilir,
ancak backend teknoloji seçimi modüler tutulmalıdır.

Editor:
- TipTap veya ProseMirror tabanlı rich-text/structured editor

State management:
- Zustand veya benzer hafif ve sürdürülebilir çözüm

==================================================
4. HAFIZA MİMARİSİ
==================================================

Üç ana hafıza seviyesi bulunmalıdır.

A. GLOBAL MEMORY

Kullanıcıya ait genel tercihler.

Örnek:

- tercih edilen yazım dili
- anlatıcı tipi
- POV tercihi
- bölüm uzunluğu
- sahne uzunluğu
- diyalog yoğunluğu
- edebi dil seviyesi
- pacing tercihleri
- tekrar istememe
- karakter gelişimi tercihleri
- romantizm pacing tercihleri
- sık kullanılan yazım kuralları

Global Memory bütün projeler için varsayılan değer sağlar.

Ancak Project Memory her zaman Global Memory üzerine override uygulayabilir.

B. PROJECT MEMORY

Belirli bir webnovel projesinin canonical hafızasıdır.

Project Memory kesinlikle tek büyük text/json blob olarak tutulmamalıdır.

Atomik, yapılandırılmış ve sorgulanabilir nesneler kullanılmalıdır.

Temel entity türleri:

- Project
- Character
- CharacterState
- Relationship
- WorldRule
- Location
- Faction
- TimelineEvent
- PlotThread
- Arc
- Chapter
- Scene
- Secret
- Foreshadow
- CanonRule
- StyleRule
- StoryFact
- Revision

C. WORKING MEMORY

O anda açık olan:

- Developer Chat
- Arc Chat
- Chapter Chat
- mevcut bölüm
- mevcut sahne

için geçici çalışma bağlamıdır.

Working Memory canonical veri değildir.

==================================================
5. CANON SİSTEMİ
==================================================

Hafıza kayıtlarının en az üç durumu bulunmalıdır:

DRAFT

Henüz tartışılan veya önerilen fikir.

CANON

Kullanıcı tarafından kabul edilmiş bilgi.

LOCKED_CANON

Kullanıcı tarafından kesinleştirilmiş ve alt AI süreçleri tarafından değiştirilemeyecek bilgi.

Örneğin Chapter Writer locked canon değiştiremez.

Developer Chat ise ancak kullanıcı açıkça istediğinde locked canon üzerinde değişiklik önerebilir veya yapabilir.

Her canonical değişiklik versioned olmalıdır.

Memory history tutulmalıdır.

Kullanıcı:

- Undo
- Compare
- Restore

yapabilmelidir.

==================================================
6. DEVELOPER CHAT
==================================================

Her projenin bir ana Developer Chat'i olacaktır.

Bu chat normal bir sohbet ekranından daha güçlüdür.

Burada kullanıcı AI ile konuşarak:

- hikâye fikri geliştirir
- karakter yaratır
- dünya kurar
- olay örgüsü geliştirir
- timeline oluşturur
- arc planlar
- yazım kuralları belirler
- canon değiştirir
- karakter veya dünya bilgilerini revize eder

Developer Chat AI'si özel memory tool'larına sahip olacaktır.

Gelecekte örneğin:

memory.search
memory.create
memory.update
memory.delete
memory.lock
memory.history

character.create
character.update

timeline.create
timeline.update

plot.create
plot.update
plot.resolve

arc.create
arc.update

gibi internal tool'lar bulunabilir.

Örnek kullanıcı komutu:

"Ji-an artık 24 değil 25 yaşında olsun."

AI yalnızca chat cevabı vermemelidir.

Gerekli memory kayıtlarını güncellemeli,
bağlantılı timeline veya diğer entity'lerde etki varsa bunları da analiz etmelidir.

Her değişiklik revision log'a kaydedilmelidir.

==================================================
7. PROJECT BIBLE
==================================================

Project Bible gerçek hafıza değildir.

Project Bible, Project Memory'nin kullanıcı tarafından okunabilen görünümüdür.

Kullanıcı Project Bible açtığında sistem yapılandırılmış hafızadan özet oluşturmalıdır.

Örnek bölümler:

- Project Overview
- Premise
- Genre
- Themes
- Tone
- Narrative Style
- Characters
- Relationships
- World
- Locations
- Factions
- Timeline
- Arcs
- Plot Threads
- Secrets
- Foreshadowing
- Canon Rules
- Writing Rules
- Current Story State

==================================================
8. PROJE / ARC / CHAPTER YAPISI
==================================================

Bir proje yaklaşık olarak şu yapıya sahip olacaktır:

Project

- Developer Chat

- Arc 1
  - Arc Planning Chat
  - Chapter 1
  - Chapter 2
  - Chapter 3

- Arc 2
  - Arc Planning Chat
  - Chapter 4
  - Chapter 5

vb.

Arc Chat Project Memory'ye bağlıdır.

Chapter Chat da Project Memory'ye bağlıdır.

Ancak hiçbir alt chat bütün Project Memory'yi doğrudan context olarak almamalıdır.

Context Compiler yalnızca gerekli bilgileri seçmelidir.

==================================================
9. MANUSCRIPT VE CHAT AYRIMI
==================================================

Roman bölümleri chat mesajı olarak tutulmamalıdır.

Her chapter ayrı bir manuscript document olmalıdır.

Chat ve manuscript birbirinden ayrı entity olmalıdır.

Kullanıcı örneğin:

"İkinci sahnedeki diyaloğu daha gergin yap."

dediğinde AI mümkünse yalnızca ilgili manuscript bölümüne patch uygulamalıdır.

Chapter revision sistemi bulunmalıdır.

Örnek:

Draft 1
Draft 2
AI Revision
User Revision
Final

Diff gösterilebilmelidir.

==================================================
10. NARRATIVE ORCHESTRATOR
==================================================

Sistem tek bir modele bütün işi yaptırmamalıdır.

Mantıksal görevler ayrılmalıdır.

Örnek logical agents:

- Story Architect
- Arc Planner
- Chapter Planner
- Scene Planner
- Writer
- Continuity Critic
- Style Critic
- Memory Curator

Ancak ÇOK ÖNEMLİ:

Logical agent != ayrı API çağrısı.

Sekiz logical agent varsa sekiz ayrı model çağrısı yapmak zorunlu değildir.

Maliyet kontrolü için bir model çağrısı birden fazla görevi birlikte yerine getirebilir.

Örneğin tek çağrı:

- Chapter Plan
- Scene Plan
- Continuity Risks
- Required Memory Queries

üretebilir.

==================================================
11. MODEL ROUTING
==================================================

Uygulama farklı büyüklükte modeller kullanabilmelidir.

Kullanım senaryosunda çoğunlukla çok büyük modeller kullanılabilir.

Ancak bazı yardımcı görevlerde 14B ve üzeri yerel modeller kullanılacaktır.

Büyük modeller:

- Story Architecture
- Arc Planning
- Complex Chapter Planning
- Main Writing
- Deep Character Psychology
- Major Revision
- Important Reveal Planning

14B–32B veya benzer küçük/orta modeller:

- Memory Extraction
- Summary
- Entity Extraction
- Timeline Extraction
- Metadata
- Tagging
- Retrieval Query Generation
- Basic Continuity Checking
- Classification

için kullanılabilir.

Narrative Orchestrator modelin yeteneklerine göre görevleri farklı biçimde parçalara ayırabilmelidir.

==================================================
12. MODEL PROFILE
==================================================

Her model için ileride ModelProfile sistemi kurulacaktır.

Örnek:

model_id

strengths:
- prose
- dialogue

weaknesses:
- long term planning
- continuity

strategy:
- use detailed chapter plan
- smaller context chunks
- enable stronger post-check

Böylece bütün modellere aynı workflow uygulanmayacaktır.

==================================================
13. QUALITY MODES
==================================================

En az üç kalite modu planlanmalıdır.

FAST

Yaklaşık:

Planner
Writer
Light Memory Extraction

BALANCED

Planner
Writer
Continuity Check
Memory Extraction

DEEP

Planner
Plot Analysis
Writer
Continuity Critic
Style Critic
Revision
Memory Extraction

Normal kullanıcı için varsayılan Balanced olabilir.

Ancak sistem mimarisi quality mode'a göre çağrıları dinamik oluşturmalıdır.

==================================================
14. CONTEXT COMPILER
==================================================

Context Compiler uygulamanın en kritik bileşenlerinden biridir.

Hiçbir model çağrısında otomatik olarak bütün chat geçmişi veya bütün proje hafızası gönderilmemelidir.

Context Compiler göreve göre context üretmelidir.

Örneğin bir Chapter Writer için:

- system instructions
- global writing preferences
- project core canon
- current arc
- chapter objective
- relevant characters
- current CharacterState records
- relevant relationships
- relevant timeline events
- active plot threads
- recent chapter summaries
- retrieved relevant memories
- recent chat messages

kullanılabilir.

Context bütçesi modele ve göreve göre değişebilmelidir.

Model context limitleri hardcoded olmamalıdır.

==================================================
15. STRUCTURED MEMORY + SEMANTIC MEMORY
==================================================

Yalnızca vector/RAG sistemi kullanmak yasaktır.

Kesin bilgiler structured database üzerinden alınmalıdır.

Örnek:

character.age
relationship.status
timeline.date
canon_rule
secret.status

Eski sahne benzerliği gibi fuzzy ihtiyaçlarda semantic retrieval kullanılmalıdır.

İdeal sistem:

Structured Retrieval
+
Semantic Retrieval
+
Relationship Graph

kombinasyonudur.

==================================================
16. MEMORY EXTRACTION
==================================================

Bir chapter finalize edildiğinde Memory Extractor çalışmalıdır.

Chapter metninden en az şu değişiklikler çıkarılmalıdır:

- chapter summary
- new facts
- character state changes
- relationship changes
- timeline events
- knowledge changes
- secrets revealed
- new secrets
- foreshadowing
- new plot threads
- progressed plot threads
- resolved plot threads
- physical condition changes
- emotional state changes
- location changes
- promises
- unresolved conflicts

Memory Extractor doğrudan canonical hafızayı değiştirmeden önce validation aşamasından geçebilmelidir.

Locked canon ile çelişen değişiklikler otomatik kabul edilmemelidir.

==================================================
17. CHARACTER STATE
==================================================

Karakterlerin yalnızca sabit biyografileri tutulmayacaktır.

CharacterState ayrıca bulunmalıdır.

Örnek alanlar:

- current location
- physical condition
- injuries
- emotional state
- goals
- beliefs
- knowledge
- secrets known
- current conflicts
- possessions
- relationship states
- promises
- last appearance
- current arc role

Bu state zaman içinde değişecektir.

==================================================
18. PLOT GRAPH
==================================================

Plot thread'leri düz text olarak tutulmamalıdır.

Bir plot graph veya ilişkilendirilebilir yapı bulunmalıdır.

Örnek:

Attack Mystery

- Foreshadow A
- Evidence B
- False Conclusion
- Evidence C
- Reveal
- Consequence

Her event ilgili chapter/arc ile bağlanabilmelidir.

AI mevcut chapter sırasında hangi plot thread'in hangi aşamasında olduğunu anlayabilmelidir.

==================================================
19. CONTINUITY ENGINE
==================================================

Continuity Engine şu sorunları kontrol edebilmelidir:

- timeline contradiction
- age contradiction
- location contradiction
- knowledge contradiction
- relationship contradiction
- character state contradiction
- world rule contradiction
- locked canon contradiction
- unexplained injury
- forgotten plot thread
- repeated scene
- repeated emotional beat
- POV violation
- character drift
- reveal timing problems

Semantic similarity ile tekrar eden sahnelerin tespiti ileride desteklenmelidir.

==================================================
20. PROVIDER ABSTRACTION
==================================================

UI doğrudan OpenAI veya başka provider kodu bilmemelidir.

Ortak interface tasarlanmalıdır.

Örnek kavramsal interface:

AIProvider

- generate()
- stream()
- listModels()
- embed()
- supportsTools()
- supportsVision()
- supportsStructuredOutput()
- supportsPromptCaching()
- getContextLimit()

Provider adapter'ları bu interface'i uygular.

Canonical project state hiçbir provider'a bağlı olmamalıdır.

==================================================
21. SYNC
==================================================

Windows ve Android aynı proje hafızasını kullanacaktır.

Local-first yaklaşım tercih edilir.

Her cihaz:

SQLite
+
local cache

kullanabilir.

Sunucu:

PostgreSQL

kullanabilir.

Genel akış:

Windows SQLite
↕
Sync Service
↕
PostgreSQL
↕
Sync Service
↕
Android SQLite

İnternet bağlantısı yokken temel proje işlemleri yapılabilmelidir.

Bağlantı geldiğinde sync yapılmalıdır.

==================================================
22. SYNC CONFLICT
==================================================

Roman metni gibi kritik verilerde basit last-write-wins kullanılmamalıdır.

Eğer PC ve telefon aynı chapter üzerinde farklı değişiklik yaptıysa conflict oluşturulmalıdır.

Kullanıcıya:

- Merge
- Use PC
- Use Mobile
- Keep Both

gibi seçenekler verilebilir.

Metadata için bazı durumlarda otomatik merge mümkün olabilir.

==================================================
23. REVISION / EVENT LOG
==================================================

Önemli entity değişiklikleri revisioned olmalıdır.

En azından şu alanlar düşünülmelidir:

id
entity_type
entity_id
revision
previous_value
new_value
created_at
device_id
actor_type
actor_id

actor_type:

- user
- ai
- system

olabilir.

==================================================
24. API KEY GÜVENLİĞİ
==================================================

Provider API key'leri plain text database'e yazılmamalıdır.

Windows ve Android'in güvenli credential/key storage mekanizmaları kullanılmalıdır.

API key'leri varsayılan olarak cloud sync edilmemelidir.

Kullanıcının açık tercihi olmadan cihazlar arasında anahtar taşınmamalıdır.

==================================================
25. UI YAPISI
==================================================

Desktop ekranı yaklaşık olarak:

LEFT SIDEBAR

Project navigation
Developer Chat
Arcs
Chapters

CENTER

Chat veya Manuscript Editor

RIGHT SIDEBAR

Context
Characters
Timeline
Plot Threads
Memory
Continuity
Project Bible

BOTTOM STATUS

Selected model
Quality mode
Context usage
Sync status

Android tarafında aynı yapı:

- drawer
- tabs
- bottom navigation
- modal panels

kullanılarak responsive hale getirilmelidir.

==================================================
26. TASARIM PRENSİPLERİ
==================================================

Aşağıdaki prensipler bağlayıcıdır:

1. Provider bağımsızlık
2. Local-first
3. Structured memory
4. Revision safety
5. Explicit canon
6. Context minimization
7. Modular AI orchestration
8. Cost-aware model routing
9. Cross-device continuity
10. User control

==================================================
27. KOD KALİTESİ
==================================================

Kod:

- modüler
- test edilebilir
- strongly typed
- kolay genişletilebilir
- provider bağımsız
- platform bağımsız
- dokümante edilmiş

olmalıdır.

Devasa god class/component oluşturma.

Business logic UI component'lerinin içine gömülmemelidir.

AI prompt'ları component'lerin içinde hardcode edilmemelidir.

Database access UI'dan doğrudan yapılmamalıdır.

Provider-specific logic core domain içine sızmamalıdır.

==================================================
28. BU AŞAMADA YAPILMAYACAKLAR
==================================================

ÇOK ÖNEMLİ:

İlk görevde bütün uygulamayı geliştirme.

Henüz şunları implement etme:

- full AI orchestration
- OpenAI integration
- Anthropic integration
- Gemini integration
- local LLM
- semantic search
- full sync engine
- continuity AI
- memory extraction AI
- EPUB export
- PDF export
- DOCX export

Önce sağlam temel oluşturulacaktır.

==================================================
29. PHASE 0 — ŞİMDİ YAPMAN GEREKEN
==================================================

İlk geliştirme görevin yalnızca Phase 0'dır.

Phase 0 hedefleri:

1. Monorepo veya uygun workspace yapısını kur.

2. Tauri 2 + React + TypeScript temel uygulamasını oluştur.

3. Windows build hedefini hazırla.

4. Android build hedefinin proje yapısını hazırla.

5. Rust/Tauri backend core kur.

6. SQLite bağlantısı ve migration sistemini kur.

7. Temel Project entity'sini oluştur.

Project için minimum:

- id
- name
- description
- created_at
- updated_at
- status

8. Project CRUD oluştur:

- create
- list
- open
- rename
- delete/archive

9. Ana UI shell oluştur:

- sidebar
- main content
- right context panel için placeholder
- status bar

10. Projects ekranı oluştur.

11. Yeni proje oluşturma modal/dialog oluştur.

12. Temel routing/navigation kur.

13. Error handling sistemi oluştur.

14. Logging altyapısı oluştur.

15. Unit test altyapısını kur.

16. Integration test için temel yapı bırak.

17. Lint + formatting yapılandır.

18. README oluştur.

19. Mimariyi açıklayan docs/ARCHITECTURE.md oluştur.

20. Database migration'larını ayrı dosyalarda tut.

==================================================
30. ÖNERİLEN REPOSITORY YAPISI
==================================================

Bunu birebir kullanmak zorunda değilsin ancak buna yakın modüler yapı tercih et:

webnovel-ai-studio/

apps/
  client/

src-tauri/

packages/
  domain/
  ai-core/
  memory/
  provider-types/
  prompts/
  shared/

migrations/

docs/
  ARCHITECTURE.md
  PRODUCT_SPEC.md
  MEMORY_SYSTEM.md
  AI_ORCHESTRATION.md
  SYNC_PROTOCOL.md

AGENTS.md

README.md

Gereksiz paket oluşturma.

Phase 0 için ihtiyaç olmayan boş abstraction'ları yalnızca gelecekte lazım olacak diye aşırı üretme.

Ancak ileride AI/memory/sync katmanlarının eklenmesini zorlaştıracak sıkı bağımlılıklar da oluşturma.

==================================================
31. DOMAIN TASARIMI
==================================================

Şimdilik minimum domain:

Project

Ancak kod organizasyonu ileride şu entity'leri destekleyebilecek biçimde olmalıdır:

Character
CharacterState
Relationship
WorldRule
TimelineEvent
PlotThread
Arc
Chapter
Scene
MemoryItem
Revision
Conversation
Message
Manuscript

Bunları Phase 0'da tam implement etme.

==================================================
32. TEST KRİTERLERİ
==================================================

Phase 0 sonunda en az şunlar çalışmalıdır:

- uygulama açılır
- project listesi görüntülenir
- yeni project oluşturulur
- SQLite'a kaydedilir
- uygulama yeniden açıldığında project kaybolmaz
- project yeniden adlandırılabilir
- project archive/delete edilebilir
- temel navigation çalışır
- unit testler geçer
- build başarılıdır

==================================================
33. ÇALIŞMA ŞEKLİN
==================================================

İlk olarak mevcut repository'yi incele.

Eğer repository boşsa uygun başlangıç yapısını kur.

Kod yazmadan önce kısa bir implementation plan oluştur.

Ardından Phase 0'ı uygula.

Karar verirken:

- sürdürülebilirliği
- Windows/Android uyumluluğunu
- ileride eklenecek AI katmanlarını
- test edilebilirliği

önceliklendir.

Gereksiz karmaşıklık oluşturma.

Phase 0 dışındaki özellikleri implement etme.

Ancak ilerideki mimariye zarar verecek kısa vadeli hack kullanma.

Her önemli değişiklik sonrası uygun testleri çalıştır.

TypeScript veya Rust compile error bırakma.

Lint hatası bırakma.

Kullanılmayan dead code bırakmamaya çalış.

==================================================
34. PHASE 0 SONUNDA BANA RAPOR VER
==================================================

İş bittiğinde aşağıdaki formatta rapor hazırla:

IMPLEMENTED

- yapılan işler

ARCHITECTURE DECISIONS

- verdiğin önemli kararlar
- nedenleri

FILES CREATED / CHANGED

- önemli dosyalar

DATABASE

- oluşturulan schema
- migrations

TESTS

- hangi testler çalıştı
- sonuçları

WINDOWS STATUS

- build durumu

ANDROID STATUS

- build/proje hazırlık durumu
- varsa environment kaynaklı eksikler

KNOWN LIMITATIONS

- mevcut eksikler

NEXT RECOMMENDED PHASE

- Phase 1 için önerin

Ancak Phase 1'i benim onayım olmadan implement etmeye başlama.

Şimdi Phase 0 ile başla.
