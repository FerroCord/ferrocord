# ferrocord yol haritası

## Kaynak ve güncel durum

[ADR Faz 1](reference/architecture-decisions-2026-10-31.md), `ferrocord v0.1.0` için bağımsız depo, senkron Arc tabanlı Context, jenerik EventBus ve Plugin trait'lerinin birim testleriyle dondurulmasını hedefler. Bu sürüm bir hedeftir; henüz yayımlanmış ürün değildir.

[31 Ekim 2026 ilerleme kaydı](reference/progress-2026-10-31.md), GitHub depolarının private olarak açıldığını kaydeder. Kaydın sonraki adımlarında yerel hazırlık henüz beklemededir; mevcut çalışma alanında ise ferrocord klonlanmış ve `origin` adresi `https://github.com/FerroCord/ferrocord.git` olarak tanımlanmıştır.

- **Tamamlandı:** GitHub deposunun açılması, yerel klon ve Faz 1 sözleşmelerinin seçimi.
- **Mevcut çalışma:** Rust crate'i, davranış testleri ve onaylı CI yapılandırması eşzamanlı geliştiriliyor.
- **Doğrulama sınırı:** Yerel kontrol sonuçları ayrıca raporlanmalıdır; GitHub Actions çalışması ve yayın yapılmış olduğu varsayılmamalıdır.

## Faz 1 görevleri ve kabul ölçütleri

Aşağıdaki ölçütler onaylı sözleşmeye göre doğrulama hedefleridir; tek başlarına test veya CI sonucu değildir.

| Görev | Kabul ölçütü | Doğrulama/test hedefi |
| --- | --- | --- |
| Sözleşmeleri belgele | Context depolaması, servis davranışları, olay kapasitesi/hataları ve Plugin yaşam döngüsü seçilmiş ve belgelenmiş olmalı. | API ve test senaryolarını [mimari sözleşmeyle](architecture.md) karşılaştır. |
| Bağımsız crate iskeletini oluştur | `Cargo.toml`, `src/lib.rs`, `context.rs`, `event.rs`, `plugin.rs` hedef yapıya uymalı; ferrocore veya AI/LLM/PTY bağımlılığı bulunmamalı. | Manifest ve bağımlılık incelemesi; seçilen araç zinciriyle derleme. |
| Context'i uygula | Tip tabanlı kayıt/erişim ve Arc paylaşımı çalışmalı; senkron kilit yalnızca kısa kritik bölümde tutulmalı. | Birden fazla servis tipi, aynı servise paylaşılan erişim, eşzamanlı okuma/yazma; karara bağlanan eksik ve yinelenen kayıt/hata senaryoları. |
| EventBus'ı uygula | Her `E: 'static + Send + Sync + Clone` tipi ayrı Tokio broadcast kanalına sahip olmalı; sabit ajan olay enum'ı olmamalı. | Olay tipi izolasyonu, birden fazla aboneye yayın; kararlaştırılan lag, kapasite, alıcısız yayın ve kapanış senaryoları. |
| Plugin sözleşmesini uygula | Onaylanmış yaşam döngüsü imzaları ve hata/kapanış davranışları uygulanmalı; ajan alanı çekirdeğe sızmamalı. | Genel amaçlı örnek test eklentisiyle kararlaştırılan sıra, başlangıç hatası, tekrar çağrı, iptal ve temizleme davranışları. |
| API'yi testlerle dondur | Context, EventBus ve Plugin testleri kararlaştırılan sözleşmeleri kapsamalı; belgeler gerçek uygulamayla eşleşmeli. | Birim testleri, biçim/Clippy kontrolleri ve bağımsız diff incelemesi; başarısız veya doğrulanmamış kontrollerin raporlanması. |
| CI | Push ve pull request üzerinde Ubuntu, stable Rust, rustfmt ve clippy ile biçim, Clippy ve test kontrolleri; yalnızca `contents: read` izni. | Gerçek GitHub Actions sonucunu ayrıca doğrula; yayın adımı yoktur. |

## Kalan kararlar

MSRV, yayın hazırlığı ve kaynak iddialarının doğrulama durumu [açık kararlar belgesinde](open-questions.md) izlenir. Seçilen Context, EventBus ve Plugin sözleşmeleri artık açık karar değildir.

## Doğrulama

Yerel kontroller:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

Context için kayıt, değiştirme, eksik servis ve eşzamanlı erişim; EventBus için tip izolasyonu, alıcı sayısı, lag, kapasite ve kapanış; Plugin için sıralama, rollback, hata toplama, geçersiz durum ve iptal senaryoları doğrulanmalıdır. Gerçek tüketici örneği servis, olay ve yaşam döngüsü kullanımını sınamalıdır. Son diff bağımsız incelenmeli; çalıştırılmayan veya engellenen kontroller açıkça raporlanmalıdır. Yerel kontroller GitHub Actions çalışmasının yerine geçmez.

## Faz 1 sonrası sınır

Kaynağın Faz 2–5 işleri ferrocore'a aittir: ortak driver, PTY/RLM MVP, recursive spawning ve derinlik kotaları, ReAct/Reflexion, MCP/WASM, plan ve MCTS döngüleri. FSM ve ambient de kaynakta ileri döngü adaylarıdır. Bu görevler ferrocord içine taşınmaz; çekirdek yalnızca onaylanan genel amaçlı ihtiyaçlar doğrultusunda geliştirilir. Entegrasyon sırasında ortaya çıkabilecek API değişiklikleri ayrıca değerlendirilir.

## Onay sınırları

Onaylı kapsam CI yapılandırmasını içerir; yayın veya dağıtım içermez. crates.io veya GitHub yayını, public görünürlüğe geçiş, push, birleştirme, silme ve paylaşılan sistem değişiklikleri ayrıca açık kullanıcı onayı gerektirir. Depo açılmış olması bu işlemler için otomatik yetki sayılmaz.
