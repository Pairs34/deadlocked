# Deadlocked — Mimari ve Yapı Analizi

## Genel Bakış

**Deadlocked**, Counter-Strike 2 (CS2) için yalnızca Linux'ta çalışan, Rust ile yazılmış bir harici oyun aracıdır. Oyun sürecinin belleğini doğrudan okuyarak çeşitli özellikler sunar. Uygulama iki ana iş parçacığı üzerinde çalışır: **oyun döngüsü** ve **UI (arayüz) döngüsü**.

---

## Teknoloji Yığını

| Katman          | Teknoloji                        |
|-----------------|----------------------------------|
| Dil             | Rust (2024 edition)              |
| GUI             | egui + egui_glow (immediate mode)|
| Pencere         | winit (X11 zorlamalı)            |
| OpenGL bağlamı  | glutin + EGL                     |
| Matematik       | glam (SIMD vektör/matris)        |
| Seri hale getirme | serde + toml (config) + serde_json (grenadlar) |
| Fare girişi     | Linux uinput çekirdek modülü     |
| Bellek okuma    | `/proc/<pid>/mem` + `process_vm_readv` syscall |

---

## Üst Düzey Mimari

```
┌──────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│  - Logger başlatma                                           │
│  - uinput kontrolü                                           │
│  - Crash handler kurulumu                                    │
│  - WAYLAND_DISPLAY kaldırma (X11 zorlaması)                  │
│  - Channel<UiMessage, GameMessage> oluşturma                 │
│  - Arc<Mutex<Data>> paylaşılan durum                         │
└───────────────┬──────────────────────────┬───────────────────┘
                │                          │
     ┌──────────▼──────────┐   ┌──────────▼──────────────────┐
     │   Oyun İş Parçacığı │   │   UI İş Parçacığı (Ana)     │
     │   game::GameManager │   │   ui::app::App              │
     │                     │   │                             │
     │  CS2'yi bul → setup │   │  winit EventLoop            │
     │  Her karedeki döngü:│   │  ┌──────────┐ ┌──────────┐ │
     │  - bellek oku       │   │  │ GUI      │ │ Overlay  │ │
     │  - özellikleri çalıştır│ │  │ (egui)   │ │ (egui)   │ │
     │  - Data güncelle    │   │  └──────────┘ └──────────┘ │
     └─────────────────────┘   └─────────────────────────────┘
                │   Channel (çift yönlü)   │
                └──────────────────────────┘
                     Arc<Mutex<Data>>
                  (paylaşılan oyun durumu)
```

### İş Parçacığı İletişimi

- **`Channel<UiMessage, GameMessage>`**: İki yönlü kanal.
  - `UiMessage(GameStatus)` → Oyun thread'i, CS2'nin bağlı/bağlı değil durumunu UI'ye bildirir.
  - `GameMessage(Box<Config>)` → UI thread'i, güncellenmiş konfigürasyonu oyun thread'ine gönderir.
- **`Arc<Mutex<Data>>`**: Her iki thread tarafından erişilen paylaşılan oyun durumu. Oyun thread'i yazar, UI thread'i okur (ESP çizimi için).

---

## Modüller

### `src/main.rs` — Giriş Noktası

- Logger'ı başlatır (`deadlocked.log` dosyasına yazar).
- `/dev/uinput` erişimini kontrol eder.
- Crash handler'ı kurar (panik anında stack trace gönderir).
- `WAYLAND_DISPLAY` ortam değişkenini siler → winit X11 üzerinden çalışır.
- Oyun thread'ini spawn eder, ardından winit event loop'unu başlatır.

---

### `src/game.rs` — Oyun Döngüsü

```
GameManager
  ├── channel: Channel<UiMessage, GameMessage>
  ├── data: Arc<Mutex<Data>>
  ├── config: Config
  ├── mouse: Mouse
  └── cs2: CS2
```

**Döngü akışı (`run()`):**
1. Kanaldaki bekleyen `GameMessage`'ları tüketir → config güncellenir.
2. CS2 geçerli değilse `setup()` çağrılır (süreci tekrar bulmaya çalışır).
3. CS2 geçerliyse:
   - `cs2.run(config, mouse)` → tüm aktif özellikleri çalıştırır.
   - `cs2.data(config, data)` → paylaşılan `Data`'yı günceller (UI için).
4. Hedef FPS'e göre uyku süresi hesaplanır (`1 / config.fps` saniye).

---

### `src/cs2/` — CS2 Çekirdeği

#### `CS2` Struct (mod.rs)

```
CS2
  ├── is_valid: bool
  ├── process: Process          ← bellek okuma
  ├── offsets: Offsets          ← dinamik bellek ofseti
  ├── input: Input              ← CS2'nin tuş durumu
  ├── bvh: Option<Bvh>          ← görünürlük için dünya geometrisi
  ├── target: Target            ← aimbot hedef seçimi
  ├── players: Vec<Player>      ← önbelleğe alınmış oyuncu listesi
  ├── entities: Vec<Entity>     ← önbelleğe alınmış varlık listesi
  ├── recoil: Recoil            ← RCS durumu
  ├── aim: Aimbot               ├── özelliklerin aktif/pasif durumu
  ├── trigger: Triggerbot       │
  ├── esp: EspToggle            │
  ├── weapon: Weapon            ← yerel oyuncunun silahı
  └── planted_c4: Option<PlantedC4>
```

**`run()` çalışma sırası:**
```
input.update()
  → cache_entities()          (200ms'de bir)
  → check_bvh()               (harita değişiminde)
  → smoke.disable/color()     (smoke modifikasyonu)
  → no_flash()
  → fov_changer()
  → esp_toggle()
  → rcs()
  → triggerbot()
  → triggerbot_shoot()
  → find_target()
  → aimbot()
```

#### `find_offsets.rs` — Dinamik Ofset Keşfi

Hardcoded ofset yerine CS2 her güncellendiğinde çalışacak şekilde:
- Modül taban adreslerini `/proc/<pid>/maps` üzerinden bulur (`client.so`, `engine2.so`, vb.)
- `GameResourceServiceClientV0`, `VEngineCvar0`, `InputSystemVersion0` gibi arayüzleri arar.
- `process_vm_readv` ile bellek örüntü taraması (pattern scanning) yapar.
- Schema sistemi aracılığıyla nesne özellik ofsetlerini dinamik olarak okur.

#### `offsets.rs` — Ofset Yapıları

```
Offsets
  ├── library: LibraryOffsets       ← .so taban adresleri
  ├── interface: InterfaceOffsets   ← arayüz pointer'ları
  ├── direct: DirectOffsets         ← doğrudan pointer'lar (local_player, view_matrix...)
  ├── convar: ConvarOffsets         ← FFA modu, sensitivite
  ├── controller: PlayerControllerOffsets
  ├── pawn: PawnOffsets             ← sağlık, zırh, açılar, silah, vb.
  ├── game_scene_node: GameSceneNodeOffsets
  └── entity_identity: EntityIdentityOffsets
```

#### `entity/` — Oyun Varlıkları

| Dosya | Açıklama |
|-------|----------|
| `player.rs` | `controller` + `pawn` pointer çifti; sağlık, konum, kemik pozisyonu, silah, görünürlük |
| `weapon.rs` | Silah türü + mermi bilgisi |
| `weapon_class.rs` | Silah sınıfı (Rifle, Pistol, Sniper, Knife, Grenade vb.) |
| `planted_c4.rs` | Bomba durumu: süre, defuse, konum |
| `smoke.rs` | Duman granatlı — renk/devre dışı bırakma |
| `inferno.rs` | Molotov ateşi alanı |
| `molotov.rs` | Molotov grenası |

**Entity enum:**
```rust
enum Entity {
    Weapon { weapon, entity },
    Inferno(Inferno),
    Smoke(Smoke),
    Molotov(Molotov),
    Flashbang(u64),
    HeGrenade(u64),
    Decoy(u64),
}
```

#### `features/` — Oyun Özellikleri

| Özellik | Dosya | Açıklama |
|---------|-------|----------|
| **Aimbot** | `aimbot.rs` | Hold/Toggle modda çalışır; en küçük FOV'lu kemiği hedefler; mesafeye göre FOV ayarı; görünürlük kontrolü |
| **Triggerbot** | `triggerbot.rs` | Crosshair düşmanda olduğunda ateşler; normal dağılımlı rastgele gecikme; kafa-sadece modu |
| **RCS** | `rcs.rs` | Aim punch ofsetini mouse hareketi ile dengeler; ayarlanabilir güç |
| **No Flash** | `no_flash.rs` | Flash alpha değerini sıfırlar |
| **FOV Changer** | `fov_changer.rs` | Oyuncu FOV değerini belleğe yazar |
| **ESP Toggle** | `esp_toggle.rs` | ESP'yi tuş ile açıp kapatır |

#### `target.rs` — Hedef Seçimi

- Tüm düşman oyuncuları tarar.
- FOV tabanlı sıralama (merkeze en yakın).
- Ölü, bağışıklı veya takım arkadaşlarını (FFA hariç) filtreler.
- Seçilen hedefi `Target` struct'ında saklar.

#### `bones.rs` — İskelet Sistemi

CS2'nin 22 kemikli iskelet modelini tanımlar: kalça, omurga (4 bölüm), boyun, kafa, kollar, bacaklar. Kemik bağlantıları ESP iskelet çizimi için kullanılır.

#### `bvh.rs` — BVH Görünürlük Kontrolü

CS2'nin fizik motorundan (`vphys_world`) dünya geometrisini okur. BVH (Bounding Volume Hierarchy) ağacını ayrıştırarak ray casting ile hedefin görünür olup olmadığını kontrol eder.

#### `input.rs` — Tuş Durumu

CS2'nin dahili `ButtonState` bellek alanını okur. 512 tuş için bit düzeyinde durum saklar. `is_key_pressed` ve `key_just_pressed` metodlarını sağlar.

---

### `src/os/` — İşletim Sistemi Katmanı

#### `process.rs` — Bellek Okuma

```
Process
  ├── pid: i32
  ├── file: File         ← /proc/<pid>/mem dosya tanıtıcısı
  ├── path: PathBuf      ← /proc/<pid>/
  ├── min/max: u64       ← geçerli adres aralığı (güvenli okuma için)
  └── string_cache       ← pointer → string önbelleği
```

- `/proc/<pid>/mem` üzerinden `FileExt::read_at` ile okuma yapar.
- Büyük veri için `process_vm_readv` syscall'ı kullanır.
- `/proc/<pid>/maps` üzerinden modül taban adreslerini ve boyutlarını bulur.
- ELF export tablosunu ayrıştırarak fonksiyon adreslerini çözer.

#### `mouse.rs` — Sanal Fare

Linux `uinput` çekirdek modülü ile sanal fare aygıtı oluşturur.

**Gizleme:** Aygıt kendini `TI-84 Plus Silver Calculator` (Texas Instruments, USB 0x0451:0xe008) olarak tanıtır.

- `mouse.move_rel(Vec2)` → rölatif mouse hareketi gönderir (aimbot/RCS için).
- `mouse.click()` → sol tık gönderir (triggerbot için).

#### `crash.rs` — Çökme Yönetimi

- Rust panik hook'u kurar.
- Panik anında stack trace'i `avitrano.ddns.net:1440` adresine TCP üzerinden gönderir (kullanıcı onayıyla).
- `STACKTRACE_SENT` atomic flag ile çift gönderimi önler.

---

### `src/ui/` — Kullanıcı Arayüzü

#### İki Pencere

```
App (winit ApplicationHandler)
  ├── gui: Option<WindowContext>      ← ayarlar penceresi
  └── overlay: Option<WindowContext>  ← şeffaf overlay (oyun üstü)
```

Her `WindowContext` bir OpenGL bağlamı (glutin/EGL) + egui renderer (egui_glow) içerir.

#### `gui/` — Ayarlar Penceresi

Sol sidebar + içerik paneli düzeni. Sekmeler:

| Sekme | İçerik |
|-------|--------|
| **Aimbot** | Aimbot, triggerbot, RCS ayarları; silah bazlı geçersiz kılma |
| **Player** | ESP rengi, iskelet, sağlık çubuğu, isim, mesafe |
| **HUD** | Bomba zamanlayıcı, düşürülmüş silahlar, grenad izleri, FOV dairesi |
| **Grenades** | Özel grenad pozisyonları (harita başına) |
| **Unsafe** | No-flash, FOV changer, duman rengi/devre dışı |
| **Config** | Çoklu config profili yönetimi (TOML) |
| **Application** | İlk başlatma, stack trace ayarı |

#### `overlay/` — ESP Overlay

- `player.rs`: Kutu ESP, iskelet, sağlık çubuğu, isim, silah, mesafe, hareket izi
- `entity.rs`: Düşürülmüş silahlar, grenadlar (inferno alanı, duman)
- `hud.rs`: Bomba zamanlayıcısı, FOV dairesi, keskin nişancı crosshair, keybind listesi, aktif özellik göstergesi

#### `grenades.rs` — Grenad Yardımcısı

Haritaya göre organize edilmiş özel grenad pozisyon veritabanı. Her grenad: UUID, isim, konum, bakış açısı, silah türü ve modifikatörler (zıplama, çömelme, koşu) içerir. JSON formatında `~/.config/deadlocked/grenades.json`'a kaydedilir.

#### `trail.rs` — Hareket İzi

Düşman oyuncularının son pozisyonlarını kaydeder ve overlay üzerinde çizgi olarak gösterir.

---

### `src/config.rs` — Konfigürasyon Sistemi

```
Config (TOML serileştirme)
  ├── aim: AimConfig
  │     ├── global: WeaponConfig
  │     │     ├── aimbot: AimbotConfig
  │     │     ├── rcs: RcsConfig
  │     │     └── triggerbot: TriggerbotConfig
  │     ├── weapons: HashMap<Weapon, WeaponConfig>   ← silah bazlı geçersiz kılma
  │     ├── aimbot_hotkey: KeyCode
  │     └── triggerbot_hotkey: KeyCode
  ├── player: PlayerConfig      ← ESP görsel ayarları
  ├── hud: HudConfig            ← HUD elemanları
  ├── misc: UnsafeConfig        ← no-flash, FOV, duman
  ├── accent_color: Color32
  └── fps: u32                  ← oyun döngüsü hedef FPS
```

Config dosyaları `~/.config/deadlocked/` altında TOML formatında saklanır. Birden fazla profil desteklenir.

---

### `src/data.rs` — Paylaşılan Durum

UI thread'i tarafından okunur, oyun thread'i tarafından yazılır:

```
Data
  ├── in_game: bool
  ├── is_ffa: bool
  ├── weapon: Weapon              ← yerel oyuncunun silahı
  ├── players: Vec<PlayerData>    ← düşman oyuncular
  ├── friendlies: Vec<PlayerData> ← takım arkadaşları (FFA modunda)
  ├── local_player: PlayerData
  ├── entities: Vec<EntityInfo>   ← düşürülmüş silahlar, grenadlar
  ├── bomb: BombData              ← bomba durumu
  ├── view_matrix: Mat4           ← 3D → 2D projeksiyon
  ├── view_angles: Vec2
  ├── window_position/size: Vec2  ← CS2 pencere boyutu (overlay konumlandırma)
  ├── aimbot_active: bool
  ├── triggerbot_active: bool
  └── esp_active: bool
```

---

### `src/parser/` — BVH Parser

CS2 fizik motorundan okunan ham üçgen verilerini (BVH ağacı) işler. Dünya geometrisi görünürlük ray casting işlemleri için kullanılır.

---

### `src/math.rs` — Matematik Yardımcıları

| Fonksiyon | Açıklama |
|-----------|----------|
| `angles_from_vector(Vec3) → Vec2` | İleri vektörden pitch/yaw açısı |
| `angles_to_fov(view, aim) → f32` | İki açı arasındaki FOV farkı |
| `world_to_screen(pos, matrix, size) → Option<Vec2>` | 3D dünya koordinatını 2D ekran koordinatına çevirir |
| `vec2_clamp(Vec2)` | Açıları [-89, 89] / [-180, 180] aralığına kısar |

---

## Derleme

```toml
[features]
read-only = []   # bellek yazma işlemlerini devre dışı bırakır
```

```toml
[profile.release]
lto = true       # link-time optimization (boyut ve hız)
debug = true     # debug sembollerini korur
```

Sadece Linux desteklenir — `#[cfg(not(target_os = "linux"))] compile_error!` ile zorlanır.

---

## Veri Akış Diyagramı

```
CS2 Süreci (bellek)
       │
       │  /proc/<pid>/mem  (process_vm_readv)
       ▼
  Process::read<T>(address) ──────────────────────────────┐
       │                                                   │
       ▼                                                   │
  find_offsets()                                           │
  (pattern scan + interface lookup + schema)               │
       │                                                   │
       ▼                                                   │
  CS2::run()                                               │
  ├── input.update() ← button_state belleği               │
  ├── cache_entities() ← entity list pointer'ları          │
  ├── no_flash / fov_changer ← belleğe yazar              │
  ├── rcs() ──────────────────────────────► uinput (fare)  │
  ├── triggerbot_shoot() ─────────────────► uinput (tık)   │
  └── aimbot() ───────────────────────────► uinput (fare)  │
       │                                                   │
  CS2::data() ──► Arc<Mutex<Data>> ────────────────────────┘
                         │
                         ▼
                  App::overlay()
                  ├── draw_player() → ESP kutu, iskelet
                  ├── draw_entity() → silahlar, grenadlar
                  └── draw_bomb_timer(), draw_fov_circle()
                         │
                         ▼
                  egui_glow → OpenGL → Ekran
```

---

## Güvenlik ve Gizleme Teknikleri

| Teknik | Açıklama |
|--------|----------|
| Sanal fare kimliği | Fare aygıtı `TI-84 Plus Silver Calculator` olarak tanıtılır |
| X11 zorlaması | `WAYLAND_DISPLAY` kaldırılarak overlay tespiti zorlaştırılır |
| Dinamik ofsetler | Hardcoded ofset yok; her çalıştırmada tarama yapılır |
| Harici süreç | CS2'nin kendi süreciyle hiçbir DLL enjeksiyonu yapılmaz |
| `read-only` feature | Yazma özelliklerini derleme zamanında devre dışı bırakır |
