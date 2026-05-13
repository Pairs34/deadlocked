use std::{
    collections::HashMap,
    fs::read_to_string,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use egui::Color32;
use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    cs2::{bones::Bones, entity::weapon::Weapon, key_codes::KeyCode},
    ui::color::Colors,
};

pub const SLEEP_DURATION: Duration = Duration::from_secs(5);
pub const DEFAULT_CONFIG_NAME: &str = "deadlocked.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum Language {
    English,
    Turkish,
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
    pub first_launch: bool,
    pub send_stacktraces: bool,
    pub language: Language,
    /// File name (e.g. "myconfig.toml") of the config to load on startup.
    /// None → fall back to DEFAULT_CONFIG_NAME.
    pub default_config: Option<String>,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            first_launch: true,
            send_stacktraces: true,
            language: Language::default(),
            default_config: None,
        }
    }
}

pub fn read_app_config() -> ApplicationConfig {
    if !APP_CONFIG_PATH.exists() {
        return ApplicationConfig::default();
    }

    let Ok(config_string) = read_to_string(APP_CONFIG_PATH.as_path()) else {
        return ApplicationConfig::default();
    };

    let config = toml::from_str(&config_string);
    config.unwrap_or_default()
}

pub fn write_app_config(config: &ApplicationConfig) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(APP_CONFIG_PATH.as_path(), out);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub aim: AimConfig,
    pub player: PlayerConfig,
    pub hud: HudConfig,
    pub misc: UnsafeConfig,
    pub accent_color: Color32,
    pub fps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            player: PlayerConfig::default(),
            hud: HudConfig::default(),
            misc: UnsafeConfig::default(),
            accent_color: Colors::BLUE,
            fps: 120,
        }
    }
}

impl Config {
    /// VAC güvenli, "legit" oynanabilecek varsayılan ayar.
    /// - Player ESP aktif (sadece okuma, tespit edilmez)
    /// - Aimbot, Triggerbot, RCS tamamen kapalı
    /// - No-flash, FOV değiştirici, No-smoke kapalı (belleğe yazıyor)
    pub fn legit() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aim: AimConfig {
                aimbot_hotkey: KeyCode::Mouse5,
                triggerbot_hotkey: KeyCode::Mouse4,
                global: WeaponConfig {
                    aimbot: AimbotConfig {
                        enable_override: false,
                        enabled: false,
                        mode: KeyMode::Hold,
                        fov: 3.0,
                        smooth: 10.0,
                        visibility_check: true,
                        flash_check: true,
                        start_bullet: 1,
                        distance_adjusted_fov: true,
                        target_friendlies: false,
                        bones: vec![Bones::Head, Bones::Neck],
                        targeting_mode: TargetingMode::Fov,
                    },
                    rcs: RcsConfig {
                        enable_override: false,
                        enabled: false,
                        mode: RcsMode::Standalone,
                        strength: glam::Vec2::splat(0.5),
                        smoothing: 0.0,
                    },
                    triggerbot: TriggerbotConfig {
                        enable_override: false,
                        enabled: false,
                        ..TriggerbotConfig::default()
                    },
                },
                weapons,
            },
            player: PlayerConfig {
                enabled: true,
                esp_hotkey: KeyCode::X,
                show_friendlies: false,
                draw_box: DrawMode::Health,
                box_mode: BoxMode::Gap,
                box_visible_color: egui::Color32::WHITE,
                box_invisible_color: egui::Color32::RED,
                draw_skeleton: DrawMode::None,
                skeleton_color: egui::Color32::WHITE,
                head_circle: false,
                health_bar: true,
                armor_bar: true,
                player_name: true,
                name_color: egui::Color32::WHITE,
                weapon_icon: true,
                weapon_icon_color: egui::Color32::WHITE,
                tags: false,
                tag_color: egui::Color32::WHITE,
                visible_only: false,
                sound: SoundConfig::default(),
            },
            hud: HudConfig {
                bomb_timer: true,
                spectator_list: true,
                dropped_weapons: true,
                keybind_list: false,
                fov_circle: false,
                grenade_trails: false,
                ..HudConfig::default()
            },
            misc: UnsafeConfig::default(), // hepsi kapalı (yazım gerektiriyor)
            accent_color: Colors::BLUE,
            fps: 120,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }

    /// Sıfırlama için: tüm override'lar kapalı, tüm aktif bayraklar false.
    /// Global ayarlar geçerli olur; UI'da hiçbir şey "aktif" görünmez.
    pub fn reset() -> Self {
        Self {
            aimbot: AimbotConfig {
                enable_override: false,
                enabled: false,
                ..AimbotConfig::default()
            },
            rcs: RcsConfig::default(),         // default: enabled: false, enable_override: false
            triggerbot: TriggerbotConfig::default(), // default: enabled: false, enable_override: false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: KeyMode,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub flash_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub bones: Vec<Bones>,
    pub targeting_mode: TargetingMode,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            mode: KeyMode::Hold,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 0,
            visibility_check: true,
            flash_check: true,
            fov: 2.5,
            smooth: 5.0,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            targeting_mode: TargetingMode::Fov,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: RcsMode,
    pub strength: Vec2,
    /// Smoothing factor (0.0 = instant, 0.9 = heavy). Applied as EMA over frames.
    pub smoothing: f32,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            mode: RcsMode::Standalone,
            strength: Vec2::splat(0.5),
            smoothing: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum RcsMode {
    /// RCS çalışır, aimbot aktif olmasa bile geri tepmeyi telafi eder
    Standalone,
    /// Sadece aimbot bir hedefe kilitliyken RCS aktif olur
    Aiming,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum KeyMode {
    Hold,
    Toggle,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TargetingMode {
    Fov,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub mode: KeyMode,
    pub flash_check: bool,
    pub scope_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub head_only: bool,
    /// When true, fire even if the crosshair is not directly on an enemy
    /// (e.g. pointing at a wall) as long as an enemy lies along the aim ray.
    /// Per-weapon (penetration capability) lets the player decide which guns
    /// can wallbang.
    pub wallbang: bool,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            delay: 100..=200,
            shot_duration: 200,
            mode: KeyMode::Hold,
            flash_check: true,
            scope_check: true,
            velocity_check: true,
            velocity_threshold: 100.0,
            head_only: false,
            wallbang: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    pub aimbot_hotkey: KeyCode,
    pub triggerbot_hotkey: KeyCode,
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aimbot_hotkey: KeyCode::Mouse5,
            triggerbot_hotkey: KeyCode::Mouse4,
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum DrawMode {
    None,
    Health,
    Color,
}

#[derive(Debug, Clone, PartialEq, EnumIter, Serialize, Deserialize)]
pub enum BoxMode {
    Gap,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub enabled: bool,
    pub esp_hotkey: KeyCode,
    pub show_friendlies: bool,
    pub draw_box: DrawMode,
    pub box_mode: BoxMode,
    pub box_visible_color: Color32,
    pub box_invisible_color: Color32,
    pub draw_skeleton: DrawMode,
    pub skeleton_color: Color32,
    pub head_circle: bool,
    pub health_bar: bool,
    pub armor_bar: bool,
    pub player_name: bool,
    pub name_color: Color32,
    pub weapon_icon: bool,
    pub weapon_icon_color: Color32,
    pub tags: bool,
    pub tag_color: Color32,
    pub visible_only: bool,
    pub sound: SoundConfig,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            esp_hotkey: KeyCode::X,
            show_friendlies: false,
            draw_box: DrawMode::Color,
            box_mode: BoxMode::Gap,
            box_visible_color: Color32::WHITE,
            box_invisible_color: Color32::RED,
            draw_skeleton: DrawMode::Health,
            skeleton_color: Color32::WHITE,
            head_circle: true,
            health_bar: true,
            armor_bar: true,
            player_name: true,
            name_color: Color32::WHITE,
            weapon_icon: true,
            weapon_icon_color: Color32::WHITE,
            tags: true,
            tag_color: Color32::WHITE,
            visible_only: false,
            sound: SoundConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    pub footstep_diameter: f32,
    pub gunshot_diameter: f32,
    pub weapon_diameter: f32,
    pub fadeout_start: f32,
    pub fadeout_duration: f32,
    pub show_visible: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            footstep_diameter: crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT,
            gunshot_diameter: crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT,
            weapon_diameter: crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT,
            fadeout_start: 1.0,
            fadeout_duration: 1.0,
            show_visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RadarConfig {
    pub enabled: bool,
    pub size: f32,
    pub zoom: f32,
    pub rotate_with_player: bool,
    pub show_names: bool,
    pub background_alpha: u8,
    pub pos_x: f32,
    pub pos_y: f32,
}

impl Default for RadarConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size: 200.0,
            zoom: 0.1,
            rotate_with_player: true,
            show_names: false,
            background_alpha: 160,
            pos_x: 10.0,
            pos_y: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub fov_circle: bool,
    pub sniper_crosshair: CrosshairConfig,
    pub dropped_weapons: bool,
    pub keybind_list: bool,
    pub spectator_list: bool,
    pub grenade_trails: bool,
    pub smoke_trail_color: Color32,
    pub molotov_trail_color: Color32,
    pub incendiary_trail_color: Color32,
    pub flash_trail_color: Color32,
    pub he_trail_color: Color32,
    pub decoy_trail_color: Color32,
    pub text_outline: bool,
    pub text_color: Color32,
    pub line_width: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub debug: bool,
    pub radar: RadarConfig,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            fov_circle: false,
            sniper_crosshair: CrosshairConfig::default(),
            dropped_weapons: true,
            keybind_list: false,
            spectator_list: false,
            grenade_trails: true,
            smoke_trail_color: Color32::LIGHT_GRAY,
            molotov_trail_color: Color32::RED,
            incendiary_trail_color: Color32::ORANGE,
            flash_trail_color: Color32::WHITE,
            he_trail_color: Color32::DARK_GRAY,
            decoy_trail_color: Color32::PURPLE,
            text_outline: true,
            text_color: Colors::TEXT,
            line_width: 2.0,
            font_size: 16.0,
            icon_size: 20.0,
            debug: false,
            radar: RadarConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosshairConfig {
    pub enabled: bool,
    pub color: Color32,
    pub line_length: f32,
    pub line_width: f32,
    pub gap: f32,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Color32::WHITE,
            line_length: 50.0,
            line_width: 2.0,
            gap: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnsafeConfig {
    pub no_flash: bool,
    pub max_flash_alpha: f32,
    pub fov_changer: bool,
    pub desired_fov: u32,
    pub no_smoke: bool,
    pub change_smoke_color: bool,
    pub smoke_color: Color32,
    pub radar_hack: bool,
}

impl Default for UnsafeConfig {
    fn default() -> Self {
        Self {
            no_flash: false,
            max_flash_alpha: 127.0,
            fov_changer: false,
            desired_fov: 90,
            no_smoke: false,
            change_smoke_color: false,
            smoke_color: Color32::RED,
            radar_hack: false,
        }
    }
}

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("deadlocked"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("configs");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static APP_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| BASE_PATH.join("deadlocked.toml"));

pub fn parse_config(path: &Path) -> Config {
    if !path.exists() || path.is_dir() {
        return Config::default();
    }

    let Ok(config_string) = read_to_string(path) else {
        return Config::default();
    };

    let config = toml::from_str(&config_string);
    if config.is_err() {
        utils::warn!("config file invalid");
    } else if let Some(file_name) = path.file_name() {
        utils::info!("loaded config {:?}", file_name);
    }
    config.unwrap_or_default()
}

pub fn write_config(config: &Config, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(path, out);
}

pub fn delete_config(path: &Path) {
    if !path.exists() {
        return;
    }

    if std::fs::remove_file(path).is_ok()
        && let Some(file_name) = path.file_name()
    {
        utils::info!("deleted config {:?}", file_name);
    }
}

pub fn available_configs() -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(8);
    let Ok(dir) = std::fs::read_dir::<&Path>(CONFIG_PATH.as_ref()) else {
        return files;
    };

    for path in dir {
        let Ok(file) = path else {
            continue;
        };
        let Ok(file_type) = file.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.ends_with(".toml") {
            continue;
        }
        files.push(file.path())
    }
    if files.is_empty() {
        let path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        write_config(&Config::default(), &path);
        files.push(path);
    }
    files
}
