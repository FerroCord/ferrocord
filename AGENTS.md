# ferrocord ajan rehberi

## Yerel kapsam

Bu rehber yalnızca bu `ferrocord/` deposunu ve alt dizinlerini kapsar. Kardeş ferrocore deposunu veya üst çalışma alanını değiştirme. Daha derindeki talimat dosyalarını çalışmadan önce kontrol et.

## Durum ve kaynaklar

Depo açıldı ve klonlandı; proje tasarım aşamasındadır. Rust kaynakları, Cargo manifesti, testler ve CI henüz yoktur. Önce güncel dosyaları incele.

- [Mimari](docs/architecture.md) ve [yol haritası](docs/roadmap.md) ferrocord kapsamını açıklar.
- [Açık kararlar](docs/open-questions.md), uygulama öncesi sözleşmeleri ve kaynak sınırlarını toplar.
- [Tam ADR kopyası](docs/reference/architecture-decisions-2026-10-31.md) iki projenin karar kaynağıdır.
- [Tam ilerleme kopyası](docs/reference/progress-2026-10-31.md) tarihsel durum kaydıdır.

Tam kaynak kopyalarındaki harici proje iddiaları ve performans beklentileri doğrulanmış sonuç değildir. Görsel yer tutucuları eksiktir; özellikle sentinel protokolünün çerçeve biçimi metinden tamamlanamaz ve ferrocore kapsamındadır. Kaynakta belirtilmeyen API ayrıntılarını kesin karar gibi yazma.

## Mimari sınırlar

- Yalnızca genel amaçlı Context, typed EventBus ve Plugin yaşam döngüsünü geliştir. AI, LLM, prompt, PTY ve ajan modellerini ekleme; bağımlılık yönü `ferrocore → ferrocord` olmalıdır.
- Servis erişimi kısa süreli senkron kilit ve `Arc` kullanır. `.await` boyunca senkron kilit veya DashMap guard'ı tutma.
- Olay yolunu olay tipi üzerinden jenerik tut; sabit ajan olay enum'ı ekleme.
- Plugin imzaları, yaşam döngüsü sırası, yinelenen servis kaydı ve olay geri basıncı gibi açık kararları uygulamadan önce netleştir; varsayımla API dondurma.

## Çalışma ve doğrulama

Türkçe, açık ve kanıta dayalı iletişim kur. Önce ilgili dosyaları oku ve kısa plan paylaş; ardından istenen değişikliği yap ve ilgili kontrolleri çalıştır. SOLID, KISS, DRY ve YAGNI ilkelerine uy; ilgisiz kullanıcı çalışmalarını koru.

Manifest ve araç zinciri oluşturulduğunda biçim, Clippy ve test komutlarını gerçek yapıya göre seç. Context eşzamanlılığı, olay tipi izolasyonu ve kararlaştırılan eklenti yaşam döngüsü için test ekle. Kaynak sınırlarını ve doğrulanmamış davranışları raporla.

Son aşamada ayrı bir alt ajana diff incelemesi yaptır; mümkün değilse bağımsız inceleme yapılmadığını belirt. Sonuçta değişiklikleri, kontrolleri ve doğrulanmamış noktaları listele.

CI kurulumu ve yayımlama kapsamını kullanıcıyla onayla. Silme, birleştirme, push, sürüm/crates.io yayını, görünürlük değişikliği ve paylaşılan sistemleri etkileyen işlemleri açık onay olmadan yapma.
