# Açık kararlar, belirsizlikler ve kaynak sınırları

Kaynak: [özgün ADR](reference/architecture-decisions-2026-10-31.md), özellikle ADR-001 ve ADR-006. Onaylanan Faz 1 sözleşmeleri [mimari belgesinde](architecture.md), kabul ölçütleri [yol haritasında](roadmap.md) açıklanır. Aşağıdakiler kalan belirsizliklerdir; eşzamanlı uygulamanın veya doğrulamanın tamamlandığı anlamına gelmez.

## Kalan kararlar

| Konu | Durum |
| --- | --- |
| MSRV | İlan edilmedi; yerelde gözlenen Rust 1.99.0 bir asgari sürüm garantisi değildir. |
| Yayın ve lisans | Manifest için `MIT OR Apache-2.0` seçildi; lisans dosyaları ve yayımlama ayrıca yetki gerektirir. Paket ve marka uygunluğu doğrulanmadı. |
| Gerçek CI çalışması | Onaylı CI kapsamı biçim, Clippy ve test kontrolleridir; GitHub Actions çalışmasının sonucu ayrıca doğrulanmalıdır. |
| Sonraki entegrasyonlar | ferrocore entegrasyonunda doğabilecek API gereksinimleri ve olası değişiklikler ayrıca değerlendirilir. |

Context kayıt ve kilit politikası, EventBus kapasite ve teslim davranışı, Plugin imzaları ve yaşam döngüsü ile CI kapsamı Faz 1 için seçilmiştir; artık açık karar olarak izlenmez.

## Korunacak mimari sınırlar

- Servis erişimi kısa süreli senkron `RwLock` ve `Arc` paylaşımı kullanır. `.await` boyunca senkron kilit tutulmaz.
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
- Kanal kapasitesi, abone hızı ve olayların klonlama maliyeti bellek ve teslim davranışını etkiler. Broadcast kayıpsız teslim veya sınırsız kuyruk garantisi vermez.
- Eklenti başlangıç/kapanış hataları veya iptal kaynakların açık kalmasına yol açabilir; çağıran yaşam döngüsü future'ını tamamlamalıdır.

Yayınlama ve paylaşılan sistemleri etkileyen işlemler ayrıca onay gerektirir.
