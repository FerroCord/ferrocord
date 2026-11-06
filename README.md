# ferrocord

Genel amaçlı, tip güvenli Rust mikro-çekirdeğidir. Servis konteyneri, jenerik olay yolu ve eklenti yaşam döngüsünü kapsar.

## Proje durumu

**Faz 1 uygulaması geliştiriliyor.** Bağımsız `ferrocord` crate'i için Rust kaynakları, Cargo manifesti, testler ve CI yapılandırması hazırlanıyor. `0.1.0` paket sürümüdür; yayımlanmış sürüm olduğu anlamına gelmez.

31 Ekim 2026 ilerleme kaydında iki deponun da private olarak oluşturulduğu belirtilir. Public duruma geçiş ve yayımlama kullanıcı onayı gerektirir. Kaydın yerel hazırlıkla ilgili sonraki adımları tarihsel durumu anlatır; bu çalışma alanında ferrocord artık klonlanmıştır.

## Depolar ve bağımlılık yönü

- Organization: <https://github.com/FerroCord>
- Bağımsız çekirdek ve crate: <https://github.com/FerroCord/ferrocord>
- Otonom yazılım geliştirme ajanı: <https://github.com/FerroCord/ferrocore>

`ferrocore → ferrocord`: Ajan çekirdeği kullanır; çekirdek ajana bağımlı değildir. Birinci parti ajan modülleri ferrocore Cargo workspace'inde, harici `ferrocore-plugin-*` eklentileri ihtiyaç halinde ayrı depolarda yer alır.

## Kapsam

- **Context:** Tip üzerinden servis kaydı ve `Arc` ile paylaşılan servis erişimi.
- **EventBus:** Her olay tipine ait ayrı Tokio broadcast kanalı.
- **Plugin:** Genel amaçlı eklenti yaşam döngüsü sözleşmesi.

AI, LLM, prompt, terminal/PTY, ajan döngüleri, araç adaptörleri ve oturum kayıtları bu çekirdeğin kapsamı dışındadır. Bunlar ferrocore sorumluluklarıdır.

## Belgeler

- [Mimari ve Faz 1 API sözleşmeleri](docs/architecture.md)
- [Faz 1 yol haritası ve kabul ölçütleri](docs/roadmap.md)
- [Açık kararlar, belirsizlikler ve kaynak sınırları](docs/open-questions.md)
- [Yerel ajan çalışma rehberi](AGENTS.md)
- [Mimari kararların tam kaynak kopyası — 31 Ekim 2026](docs/reference/architecture-decisions-2026-10-31.md)
- [İlerleme kaydının tam kaynak kopyası — 31 Ekim 2026](docs/reference/progress-2026-10-31.md)

Kaynak kopyalar iki projeyi birlikte anlatır; her madde ferrocord görevi değildir. Harici projelerle ilgili iddialar ve performans beklentileri bu depoda doğrulanmamıştır. Kaynakta eksik görsel yer tutucuları vardır; bunlar tamamlanmış protokol veya API tanımı sayılmaz.

## Doğrulama durumu

Yerel doğrulama sonuçları ayrıca raporlanmalıdır; CI yapılandırmasının varlığı GitHub Actions üzerinde başarıyla çalıştığını göstermez. Biçim, Clippy ve test komutları ile kabul ölçütleri [yol haritasındadır](docs/roadmap.md).
