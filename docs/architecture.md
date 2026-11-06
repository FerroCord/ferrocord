# ferrocord mimarisi

## Durum ve karar kaynağı

Bu belge, eşzamanlı geliştirilen Faz 1 uygulamasının onaylı sözleşmesini açıklar; test veya CI başarısı iddiası değildir.

Kaynaklar: [tam mimari karar kaydı](reference/architecture-decisions-2026-10-31.md), özellikle ADR-001 ve ADR-006; [31 Ekim 2026 ilerleme kaydı](reference/progress-2026-10-31.md). Kaynaklardaki harici projelerle ilgili iddialar, nanosaniye erişim beklentisi dahil performans ifadeleri ve karşılaştırmalar bu depoda doğrulanmamıştır. Eksik görsel yer tutucuları kesin teknik sözleşme oluşturmaz.

## Sorumluluk ve bağımlılık sınırı

ferrocord ayrı bir depo ve bağımsız Rust kütüphane crate'idir. Üç temel sorumluluğu vardır: servis konteyneri, tip güvenli olay yolu ve eklenti yaşam döngüsü.

Bağımlılık yönü **`ferrocore → ferrocord`** şeklindedir. ferrocord içinde AI, LLM, prompt veya terminal kavramları bulunmaz. ferrocore; AgentLoop, Tool ve Session modellerini, ortak TurnDriver'ı, LLM akışı/retry/bütçe/iptal yönetimini, PTY/sentinel yürütmesini, alt ajanları, JSONL geçmişini ve MCP/WASM adaptörlerini kendi katmanında tanımlar.

RLM ve ReAct kardeş döngüleri ile ileri ajan döngüleri ferrocore'a aittir; bu çekirdekte uygulanmaz. Birinci parti ajan modülleri ferrocore workspace'inde, harici eklentiler ayrı depolarda planlanır. ferrocord Plugin sözleşmesi, tek başına harici süreç veya WASM yükleme motoru anlamına gelmez.

## Context: tip tabanlı servis konteyneri

Depolama `std::sync::RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>` kullanır. `Context::new()` ve `Default` boş konteyner oluşturur. `T: Any + Send + Sync` için:

- `insert(&self, service: Arc<T>) -> Option<Arc<T>>` eski servisi atomik olarak değiştirir ve döndürür.
- `try_insert(&self, service: Arc<T>) -> Result<(), DuplicateService>` mevcut servisi değiştirmeden yinelenen kayıt hatası verir.
- `get<T>(&self) -> Option<Arc<T>>` paylaşılan servisi döndürür; eksik tip için `None` verir.

Kilit zehirlenmesinde `PoisonError::into_inner()` ile devam edilir; TypeId ile saklanan tipin uyuşmaması iç invariant ihlalidir ve panic oluşturur. Kilit altında yalnızca tablo erişimi ve Arc işlemleri yapılır; eski değerin yok edilmesi kilit dışında gerçekleşir. `.await` boyunca senkron kilit tutulmaz.

## EventBus: tip güvenli olay yolu

Her `E: 'static + Send + Sync + Clone` tipi için ayrı bir `tokio::sync::broadcast` kanalı kullanılır. `EventBus::new()` ve `Default` varsayılan 256 kapasitelidir; `with_capacity(NonZeroUsize) -> Result<Self, InvalidCapacity>` tüm tipler için kapasiteyi seçer ve `usize::MAX / 2` üzerindeki değerleri reddeder. Tokio tamponu sonraki ikinin kuvvetine yuvarlar: 3 için kapasite 4 olaydır. Bellek ayırma başarısızlığı süreç hatasına yol açabilir.

`subscribe<E>(&self) -> broadcast::Receiver<E>` abone oluşturur; `publish<E>(&self, event: E) -> usize` gönderim anındaki alıcı sayısını döndürür. Abone yoksa sonuç 0'dır ve olay saklanmaz. Tip başına kanal ilk kullanımda tek kez oluşturulur; başlangıç alıcısı bırakılır. Sender'lar bus yaşadığı sürece tutulur; bus ve geçici sender klonları bırakılınca alıcılar kapanır.

Alıcı `recv` için Tokio'nun `Lagged(n)` ve `Closed` sonuçlarını doğrudan alır. Otomatik atlama, üreticiye geri basınç veya kayıpsız teslim garantisi yoktur. Tablo kilidi yayın dışında bırakılır; zehirlenme ve tip invariantı Context ile aynı politikayı izler.

## Plugin: genel amaçlı yaşam döngüsü

`Plugin: Send + Sync` nesne güvenli async trait'idir: `name(&self) -> &str`, `init(&self, context: &Context) -> Result<(), PluginError>` ve `shutdown(&self, context: &Context) -> Result<(), PluginError>`. `PluginError`, `Box<dyn std::error::Error + Send + Sync + 'static>` tipidir.

`PluginManager::new()` ve `Default` yeni yönetici oluşturur. `add(&mut self, Box<dyn Plugin>)` yalnızca başlangıç durumunda çalışır; ad kayıt sırasında bir kez saklanır ve tam string eşitliğiyle benzersiz olmalıdır. `start` ve `shutdown` async metotları `&mut self` ve `&Context` alır. Başlangıç FIFO, kapanış LIFO sırasındadır. Başarısız init sonrasında yalnızca başarıyla başlatılmış eklentiler ters sırada kapatılır; özgün hata ve tüm rollback hataları korunur. Başarısız init kendi kısmi kaynak temizliğinden sorumludur. Normal kapanış bütün eklentileri dener ve tüm hataları çağrı sırasıyla toplar.

`LifecycleError`, `InvalidState`, `DuplicateName(String)`, `InitFailed { plugin, source, rollback_errors }` ve `ShutdownFailed { errors }` varyantlarını sunar. Başarısız başlangıç veya tamamlanan kapanış sonrası yönetici sonlanır; yeniden başlatma yoktur. Başlatma veya kapanış future'ı tamamlanmadan bırakılırsa yönetici ara durumda kalır ve sonraki işlemler `InvalidState` verir. İptalde async temizleme garanti edilmez; çağıran future'ı tamamlanana kadar poll etmeli ve başlangıç ile kapanışta aynı Context örneğini kullanmalıdır.

## Paket ve CI

Paket Rust 2021 edition ve `MIT OR Apache-2.0` lisans ifadesini kullanır; MSRV ilan edilmez. Çalışma zamanı bağımlılıkları `async-trait` ve `sync` özellikli Tokio'dur. Testlerde Tokio'nun `macros`, `rt-multi-thread`, `sync` ve `time` özellikleri kullanılır. Lisans ifadesi lisans dosyaları veya yayımlama yetkisi anlamına gelmez.

Onaylı CI, push ve pull request için Ubuntu üzerinde stable Rust, rustfmt ve clippy ile biçim, Clippy ve test kontrollerini çalıştırır; yalnızca `contents: read` izni kullanır. Yayın veya dağıtım adımı yoktur. Yapılandırma, gerçek GitHub Actions çalışmasının başarılı olduğuna kanıt değildir.

## Açık kararlar ve kaynak sınırları

Kalan kararlar, kaynak iddialarının doğrulama durumu ve eksik görsellerin kapsamı [açık kararlar belgesinde](open-questions.md) izlenir.

## Kaynak ve eşzamanlılık sınırları

Senkron kilit altında uzun işlem veya yoğun yazma, yürütücü iş parçacıklarını bloke edebilir. `Arc` paylaşımı servis iç mantığının doğruluğunu kendiliğinden sağlamaz. Olay kanalı kapasitesi ve abonelerin hızı bellek ve teslim davranışını etkiler; `Clone` maliyeti olay yüküne bağlıdır. Eklenti hataları veya iptal kaynakların açık kalmasına yol açabilir. Bunlar ölçülmüş sorunlar değil, tasarım riskleridir.

Uygulama adımları, kabul ölçütleri ve doğrulama hedefleri için [yol haritasına](roadmap.md) bakın.
