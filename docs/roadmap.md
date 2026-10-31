# ferrocord yol haritası

## Kaynak ve güncel durum

[ADR Faz 1](reference/architecture-decisions-2026-10-31.md), `ferrocord v0.1.0` için bağımsız depo, senkron Arc tabanlı Context, jenerik EventBus ve Plugin trait'lerinin birim testleriyle dondurulmasını hedefler. Bu sürüm bir hedeftir; henüz yayımlanmış ürün değildir.

[31 Ekim 2026 ilerleme kaydı](reference/progress-2026-10-31.md), GitHub depolarının private olarak açıldığını kaydeder. Kaydın sonraki adımlarında yerel hazırlık henüz beklemededir; mevcut çalışma alanında ise ferrocord klonlanmış ve `origin` adresi `https://github.com/FerroCord/ferrocord.git` olarak tanımlanmıştır.

- **Tamamlandı:** GitHub deposunun açılması ve yerel klonun hazırlanması.
- **Mevcut çalışma:** kapsam, mimari ve yapılacakların Türkçe dokümantasyonu.
- **Başlamadı:** Rust uygulaması, manifest, birim testleri ve CI/yayımlama hattı.

## Faz 1 görevleri ve kabul ölçütleri

Aşağıdaki ölçütler uygulamanın doğrulanması için hedeflerdir; tamamlanmış test sonucu veya kaynakta bulunmayan kesin API kararı değildir.

| Görev | Kabul ölçütü | Doğrulama/test hedefi |
| --- | --- | --- |
| Açık sözleşmeleri netleştir | Context depolama seçimi, servis davranışları, olay kapasitesi/hataları ve Plugin yaşam döngüsü açıkça kararlaştırılmış; kesin imzalar belgelenmiş olmalı. | Kararlarla API ve test senaryolarının karşılaştırılması; belirsizliklerin varsayımla kapatılmaması. |
| Bağımsız crate iskeletini oluştur | `Cargo.toml`, `src/lib.rs`, `context.rs`, `event.rs`, `plugin.rs` hedef yapıya uymalı; ferrocore veya AI/LLM/PTY bağımlılığı bulunmamalı. | Manifest ve bağımlılık incelemesi; seçilen araç zinciriyle derleme. |
| Context'i uygula | Tip tabanlı kayıt/erişim ve Arc paylaşımı çalışmalı; senkron kilit yalnızca kısa kritik bölümde tutulmalı. | Birden fazla servis tipi, aynı servise paylaşılan erişim, eşzamanlı okuma/yazma; karara bağlanan eksik ve yinelenen kayıt/hata senaryoları. |
| EventBus'ı uygula | Her `E: 'static + Send + Sync + Clone` tipi ayrı Tokio broadcast kanalına sahip olmalı; sabit ajan olay enum'ı olmamalı. | Olay tipi izolasyonu, birden fazla aboneye yayın; kararlaştırılan lag, kapasite, alıcısız yayın ve kapanış senaryoları. |
| Plugin sözleşmesini uygula | Onaylanmış yaşam döngüsü imzaları ve hata/kapanış davranışları uygulanmalı; ajan alanı çekirdeğe sızmamalı. | Genel amaçlı örnek test eklentisiyle kararlaştırılan sıra, başlangıç hatası, tekrar çağrı, iptal ve temizleme davranışları. |
| API'yi testlerle dondur | Context, EventBus ve Plugin testleri kararlaştırılan sözleşmeleri kapsamalı; belgeler gerçek uygulamayla eşleşmeli. | Birim testleri, biçim/Clippy kontrolleri ve bağımsız diff incelemesi; başarısız veya doğrulanmamış kontrollerin raporlanması. |
| CI ve sürüm hazırlığını onayla | CI kapsamı, araç zinciri, yetkiler ve yayımlama süreci kullanıcıyla onaylanmalı; yalnızca onaylanan yapı kurulmalı. | Onay sonrası gerçek CI çalışmasının ve yayın önkoşullarının kontrolü; yayın işlemi ayrıca açık onaya bağlı. |

## Uygulama öncesi karar listesi

Context, EventBus, Plugin, araç zinciri ve dağıtım sözleşmeleri [açık kararlar belgesinde](open-questions.md) izlenir. Ayrıntılar netleşmeden testlere varsayımsal politika gömülmemelidir. Kaynak iddialarının doğrulama durumu ve olası darboğazlar da aynı belgede yer alır.

## Planlanan doğrulama

Manifest ve araç zinciri oluşturulduktan sonra gerçek yapıya göre şu kontroller değerlendirilir:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

Bu komutlar mevcut depoda henüz çalıştırılabilir bir uygulamayı doğrulamış değildir. Eşzamanlı erişim testleriyle birlikte `.await` boyunca kilit/guard tutulmadığı kod incelemesinde kontrol edilmelidir. Yaşam döngüsü testleri kesin sözleşme belirlendikten sonra yazılmalıdır. Son değişiklik ayrı bir alt ajan tarafından incelenmeli; mümkün değilse bu sınır açıkça raporlanmalıdır.

## Faz 1 sonrası sınır

Kaynağın Faz 2–5 işleri ferrocore'a aittir: ortak driver, PTY/RLM MVP, recursive spawning ve derinlik kotaları, ReAct/Reflexion, MCP/WASM, plan ve MCTS döngüleri. FSM ve ambient de kaynakta ileri döngü adaylarıdır. Bu görevler ferrocord içine taşınmaz; çekirdek yalnızca onaylanan genel amaçlı ihtiyaçlar doğrultusunda geliştirilir. Entegrasyon sırasında ortaya çıkabilecek API değişiklikleri ayrıca değerlendirilir.

## Onay sınırları

Bu dokümantasyon çalışması CI kodu eklemez ve sürüm yayımlamaz. CI kurulumu, crates.io veya GitHub yayını, public görünürlüğe geçiş, push, birleştirme, silme ve paylaşılan sistem değişiklikleri için kapsamı netleştirip kullanıcı onayı alınmalıdır. Depo açılmış olması, bu işlemler için otomatik yetki sayılmaz.
