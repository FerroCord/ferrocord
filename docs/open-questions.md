# Açık kararlar, belirsizlikler ve kaynak sınırları

Kaynak: [özgün ADR](reference/architecture-decisions-2026-10-31.md), özellikle ADR-001 ve ADR-006. Bu belge, ferrocord uygulamasından önce netleştirilecek sözleşmeleri toplar. Maddeler yeni mimari karar veya tamamlanmış uygulama anlamına gelmez. Faz 1 görevleri ve kabul ölçütleri [yol haritasında](roadmap.md), hedef tasarım [mimari belgesinde](architecture.md) yer alır.

## Uygulama öncesi sözleşmeler

| Konu | Netleştirilecek ayrıntı | Aşama |
| --- | --- | --- |
| Context depolaması | Senkron RwLock/HashMap veya kaynakta alternatif olan DashMap seçimi; kilit zehirlenmesi ve hata politikası | Faz 1, Context öncesi |
| Servis API'si | Kayıt/erişim imzaları ve tip sınırları; eksik servis, yinelenen kayıt, değiştirme/kaldırma ve tip dönüşümü davranışı | Faz 1, Context öncesi |
| EventBus API'si | Yayın/abonelik imzaları; kanal kapasitesi, oluşturulması ve ömrü | Faz 1, EventBus öncesi |
| Olay teslimi | Yavaş alıcı ve gecikme (`lag`), geri basınç (backpressure), alıcısız yayın, hata aktarımı ve kapanış politikası | Faz 1, EventBus öncesi |
| Plugin sözleşmesi | Kesin trait imzaları, async ayrıntıları, Context aktarımı, nesne güvenliği ve trait sınırları | Faz 1, Plugin öncesi |
| Yaşam döngüsü | Sıralama ve eklenti bağımlılıkları; başlangıç hatasında geri alma, tekrar çağrı, iptal ve kapanış temizliği | Faz 1, Plugin öncesi |
| Araç zinciri | Rust sürümü ve asgari desteklenen Rust sürümü (MSRV), bağımlılık sürümleri ve özellik bayrakları | Faz 1, crate iskeleti öncesi |
| Dağıtım | Sürümleme, lisans ve paket metaverisi; CI kapsamı, izinleri ve crates.io yayın gereksinimleri | Faz 1, CI ve sürüm hazırlığı |

Kaynakta `#[async_trait] pub trait Plugin` yönelimi bulunur; kesin metot imzaları tanımlanmamıştır. Lisans ve paket metaverisi gibi yayın ayrıntıları da kaynakta kesinleşmiş kabul edilmemelidir. API ve test politikaları ilgili kararlar netleştikten sonra oluşturulmalıdır.

## Korunacak mimari sınırlar

- Servis erişimi kısa süreli senkron kilit ve `Arc` paylaşımı kullanır. `.await` boyunca senkron kilit veya DashMap guard'ı tutulmaz.
- Olay yolu her `E: 'static + Send + Sync + Clone` tipi için ayrı Tokio broadcast kanalı kullanır.
- Çekirdek genel amaçlı Context, EventBus ve Plugin sorumluluklarını taşır. Ajan, LLM, prompt, PTY, alt ajan ve araç adaptörleri ferrocore kapsamındadır.
- Bağımlılık yönü `ferrocore -> ferrocord` olarak korunur.

## Kaynak iddialarının doğrulama durumu

Özgün ADR'deki DeepSeek Harness ve Cordis karşılaştırmaları, Rust portu durumu ve nanosaniye düzeyinde erişim beklentisi bu depoda bağımsız olarak doğrulanmadı. Performans garantisi için ölçüm gerekir. Proje isimlerinin marka ve crates.io uygunluğu da doğrulanmadı.

Kaynakta sentinel ve bazı ajan akışları için eksik görsel yer tutucuları bulunur. Bu eksikliklerin çözümü ferrocore kapsamındadır; çekirdek için varsayımsal API gereksinimi oluşturmaz.

[İlerleme kaydı](reference/progress-2026-10-31.md), depo oluşturma ve o tarihteki görünürlük doğrulamasını içerir. Güncel uygulama durumu yerel dosyalardan ayrıca kontrol edilmelidir.

## Tasarım riskleri ve olası darboğazlar

Aşağıdaki başlıklar tasarım riskleridir; performans ölçümü yapılmadı.

- Senkron kilit altında uzun işlem veya yoğun yazma, yürütücü iş parçacıklarını bloke edebilir.
- `Arc` paylaşımı servis iç mantığının eşzamanlılık güvenliğini tek başına sağlamaz.
- Kanal kapasitesi, abone hızı ve olayların klonlama maliyeti bellek ve teslim davranışını etkiler. Tokio broadcast davranışı değerlendirilmeden kayıpsız teslim veya sınırsız kuyruk garantisi verilemez.
- Eklenti başlangıç/kapanış hataları ve belirsiz temizleme sahipliği kaynakların açık kalmasına yol açabilir.

Kararlar, ilgili test hedefleriyle birlikte belgelenmelidir. CI kurulumu ve yayın kapsamı kullanıcıyla netleştirilir; yayınlama ve paylaşılan sistemleri etkileyen işlemler ayrıca onay gerektirir.
