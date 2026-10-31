# ferrocord mimarisi

## Durum ve karar kaynağı

Bu belge hedef tasarımı anlatır; uygulanmış veya test edilmiş API belgesi değildir. Depo açılmış ve klonlanmıştır; Rust uygulaması başlamamıştır.

Kaynaklar: [tam mimari karar kaydı](reference/architecture-decisions-2026-10-31.md), özellikle ADR-001 ve ADR-006; [31 Ekim 2026 ilerleme kaydı](reference/progress-2026-10-31.md). Kaynaklardaki harici projelerle ilgili iddialar, nanosaniye erişim beklentisi dahil performans ifadeleri ve karşılaştırmalar bu depoda doğrulanmamıştır. Eksik görsel yer tutucuları kesin teknik sözleşme oluşturmaz.

## Sorumluluk ve bağımlılık sınırı

ferrocord ayrı bir depo ve bağımsız Rust kütüphane crate'i olarak planlanır. Üç temel sorumluluğu vardır: servis konteyneri, tip güvenli olay yolu ve eklenti yaşam döngüsü.

Bağımlılık yönü **`ferrocore → ferrocord`** şeklindedir. ferrocord içinde AI, LLM, prompt veya terminal kavramları bulunmaz. ferrocore; AgentLoop, Tool ve Session modellerini, ortak TurnDriver'ı, LLM akışı/retry/bütçe/iptal yönetimini, PTY/sentinel yürütmesini, alt ajanları, JSONL geçmişini ve MCP/WASM adaptörlerini kendi katmanında tanımlar.

RLM ve ReAct kardeş döngüleri ile ileri ajan döngüleri ferrocore'a aittir; bu çekirdekte uygulanmaz. Birinci parti ajan modülleri ferrocore workspace'inde, harici eklentiler ayrı depolarda planlanır. ferrocord Plugin sözleşmesi, tek başına harici süreç veya WASM yükleme motoru anlamına gelmez.

## Context: tip tabanlı servis konteyneri

ADR-001'in temel tasarımı `std::sync::RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>` yapısıdır. **DashMap kaynakta alternatif olarak belirtilir; nihai seçim burada yapılmaz.**

- Servis anahtarı `TypeId` ile temsil edilir.
- Servisler `Arc` üzerinden paylaşılır; `Any + Send + Sync` sınırları tip silme ve eşzamanlı erişim temelini oluşturur.
- Okuma sırasında kısa süreli senkron kilit alınır, servis için `Arc::clone` yapılır ve kilit bırakılır. Servisle çalışma kilit dışında gerçekleşir.
- `.await` boyunca senkron kilit tutulmaz. DashMap seçilirse aynı sınır guard'lar için de geçerlidir.
- Kaynak bu kullanım için `tokio::sync::RwLock` yerine kısa süreli senkron erişimi seçer; bu, uzun süren işi kilit içinde yapma izni değildir.

Kesin kayıt/erişim metot adları ve imzaları tanımlı değildir. Eksik servis, yanlış tip dönüşümü, yinelenen kayıt, değiştirme/kaldırma ve kilit zehirlenmesi davranışları açık kararlardır.

## EventBus: tip güvenli olay yolu

Her olay tipi için ayrı bir **`tokio::sync::broadcast::Sender<E>`** kanalı planlanır. Olay sınırı **`E: 'static + Send + Sync + Clone`** şeklindedir. Kanallar tip tabanlı olarak ayrılır; ajan alanını bilen sabit bir `Event` enum'ı çekirdeğe konmaz.

Kaynakta `event.rs`, TypeId tabanlı jenerik Tokio broadcast olay yolu olarak tanımlanır. Yayınlama ve abonelik API'sinin kesin metot imzaları henüz belirtilmemiştir.

Kanal kapasitesi, yavaş alıcı/gecikme (`lag`) politikası, geri basınç (backpressure), alıcısız yayın, kapanış ve hata aktarımı açık kararlardır. Tokio broadcast davranışı değerlendirilmeden kayıpsız teslim, üreticiyi bekletme veya sınırsız kuyruk garantisi verilemez.

## Plugin: genel amaçlı yaşam döngüsü

Kaynak, `plugin.rs` içinde `#[async_trait] pub trait Plugin` yönelimini ve eklenti yaşam döngüsü sorumluluğunu belirtir. **Kesin trait imzaları bilinmiyor.** Başlatma/durdurma metotlarının adları, parametreleri, dönüş/hata tipleri, Context aktarımı, nesne güvenliği ve trait sınırları bu belgede icat edilmez.

Yaşam döngüsü sırası, eklentiler arası bağımlılık, başarısız başlangıçta geri alma, tekrar çağrı, iptal ve kapanış temizliği uygulama öncesinde netleştirilmelidir. Aşağıdaki yol haritası, testleri bu kararların sonucuna bağlar.

## Hedef dosya ağacı

Aşağıdaki yollar ADR'deki hedeflerdir; henüz oluşturulmuş uygulama dosyaları değildir:

- `Cargo.toml`: bağımsız `[package]`, paket adı `ferrocord`.
- `src/lib.rs`: kütüphane giriş noktası.
- `src/context.rs`: senkron servis konteyneri ve Arc paylaşımı.
- `src/event.rs`: tip tabanlı Tokio broadcast olay yolu.
- `src/plugin.rs`: Plugin yaşam döngüsü sözleşmesi.
- `.github/workflows/ci.yml`: bağımsız hızlı CI ve crates.io yayımlama hattı hedefi; kurulum ve yayımlama kapsamı kullanıcı onayıyla netleştirilir.

Rust sürümü/MSRV, bağımlılık sürümleri, özellik bayrakları ve yayımlama ayarları kaynakta kesinleştirilmemiştir. Bu belge manifest veya CI kodu eklemez.

## Açık kararlar ve kaynak sınırları

Uygulama öncesi sözleşmeler, kaynak iddialarının doğrulama durumu ve eksik görsellerin kapsamı [açık kararlar belgesinde](open-questions.md) izlenir. Context, EventBus ve Plugin API ayrıntıları ilgili kararlar netleştikten sonra kesinleştirilecektir.

## Kaynak ve eşzamanlılık sınırları

Senkron kilit altında uzun işlem veya yoğun yazma, yürütücü iş parçacıklarını bloke edebilir. `Arc` paylaşımı servis iç mantığının doğruluğunu kendiliğinden sağlamaz. Olay kanalı kapasitesi ve abonelerin hızı bellek ve teslim davranışını etkiler; `Clone` maliyeti olay yüküne bağlıdır. Eklenti başlangıç/kapanış hataları kaynakların açık kalmasına yol açabilir. Bunlar ölçülmüş sorunlar değil, karar ve test aşamasında ele alınacak risklerdir.

Uygulama adımları, kabul ölçütleri ve doğrulama hedefleri için [yol haritasına](roadmap.md) bakın.
