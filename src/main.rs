use clap::Parser;
use std::path::PathBuf;

mod dota;
use dota::*;

#[derive(Parser, Debug)]
#[command(
    name = "aura-cli",
    version,
    about = "✦ Aura Community CLI — Open-source basic Dota 2 configuration optimizer"
)]
struct Cli {
    /// Apply the basic VAC-safe configuration preset (basic)
    #[arg(short, long, value_name = "PRESET", num_args = 0..=1, default_missing_value = "basic")]
    apply: Option<String>,

    /// Restore all modified configurations from backup
    #[arg(short, long)]
    restore: bool,

    /// Print current detected Dota 2 environment and settings status
    #[arg(short, long)]
    status: bool,

    /// Override Dota 2 directory path (path to "dota 2 beta" or "game/dota")
    #[arg(long, value_name = "PATH")]
    dota_dir: Option<PathBuf>,

    /// Override Steam installation directory path
    #[arg(long, value_name = "PATH")]
    steam_dir: Option<PathBuf>,
}

fn aura_pro_url() -> &'static str {
    option_env!("AURA_SITE_URL").unwrap_or("https://github.com/farconiorig/aura-cli")
}

fn get_backup_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local_app_data).join("aura").join("backups");
        }
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            return PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("aura")
                .join("backups");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(xdg_data) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(xdg_data).join("aura/backups");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".local/share/aura/backups");
        }
    }

    std::env::temp_dir().join("aura_backups")
}

fn account_backup_path(backup_dir: &std::path::Path, account_id: &str) -> PathBuf {
    backup_dir
        .join("userdata")
        .join(account_id)
        .join("video.txt.bak")
}

fn main() {
    let cli = Cli::parse();
    let backup_dir = get_backup_dir();

    println!("✦ AURA COMMUNITY CLI (Open-Source MIT Edition)");
    println!("  100% VAC-Safe Basic Configuration (video.txt & autoexec.cfg)\n");

    let env = match DotaEnvironment::detect_with_overrides(
        cli.dota_dir.as_deref(),
        cli.steam_dir.as_deref(),
    ) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("❌ Ошибка обнаружения Dota 2: {err}");
            eprintln!("   Убедитесь, что Steam и Dota 2 установлены, или передайте --dota-dir.");
            std::process::exit(1);
        }
    };

    if cli.status || (cli.apply.is_none() && !cli.restore) {
        println!("  Директория игры: {}", env.dota_game_dir.display());
        match &env.steam_dir {
            Some(s) => println!("  Steam:           {}", s.display()),
            None => println!("  Steam:           Не обнаружен (--dota-dir режим)"),
        }
        println!(
            "  Аккаунты:        {} обнаружено",
            env.userdata_configs.len()
        );
        println!(
            "  autoexec.cfg:    {}",
            if is_autoexec_applied(&env.cfg_dir()) {
                "Активен (Basic)"
            } else {
                "Не настроен"
            }
        );
        for profile in &env.userdata_configs {
            match VideoConfig::load_from_file(&profile.video_cfg_path) {
                Ok(cfg) => {
                    println!(
                        "  video.txt [{}] : {}",
                        profile.account_id,
                        if cfg.is_basic_optimized() {
                            "Оптимизирован (Basic)"
                        } else {
                            "Стандартный"
                        }
                    );
                }
                Err(e) => eprintln!("  video.txt [{}] : Ошибка чтения ({e})", profile.account_id),
            }
        }
        println!(
            "\n  ℹ️ Расширенные пресеты (Competitive/Potato/HighRefresh), WebM-нейтрализатор,"
        );
        println!(
            "     No-Bling, очистка кэша, нативный VPK и GUI доступны в Aura Pro: {}",
            aura_pro_url()
        );
    }

    if let Some(ref preset) = cli.apply {
        if !preset.eq_ignore_ascii_case("basic") {
            eprintln!(
                "❌ Пресет '{preset}' недоступен в Aura Community CLI (поддерживается только 'basic')."
            );
            eprintln!(
                "   Расширенные профили (Competitive, Balanced, Potato, HighRefresh) доступны в Aura Pro: {}",
                aura_pro_url()
            );
            std::process::exit(1);
        }

        println!("  Применение базового пресета 'basic'...");
        for profile in &env.userdata_configs {
            let mut cfg = match VideoConfig::load_from_file(&profile.video_cfg_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("❌ Ошибка чтения {}: {e}", profile.video_cfg_path.display());
                    std::process::exit(1);
                }
            };
            cfg.apply_basic_preset();
            let bak = account_backup_path(&backup_dir, &profile.account_id);
            if let Err(e) = cfg.save_to_file(&profile.video_cfg_path, Some(&bak)) {
                eprintln!("❌ {e}");
                std::process::exit(1);
            }
            println!("  ✓ video.txt обновлен для аккаунта {}", profile.account_id);
        }

        if let Err(e) = apply_autoexec(&env.cfg_dir(), AutoexecPreset::Basic, Some(&backup_dir)) {
            eprintln!("❌ {e}");
            std::process::exit(1);
        }
        println!("  ✓ autoexec.cfg успешно записан (базовый пресет).");
        println!("\n✓ Базовая оптимизация завершена!");
    }

    if cli.restore {
        println!("  Восстановление файлов из бэкапа...");
        if let Err(e) = restore_autoexec(&env.cfg_dir(), &backup_dir) {
            eprintln!("❌ {e}");
            std::process::exit(1);
        }

        let single_account = env.userdata_configs.len() == 1;
        let legacy_bak = backup_dir.join("video.txt.bak");

        for profile in &env.userdata_configs {
            let bak = account_backup_path(&backup_dir, &profile.account_id);
            let source_bak = if bak.exists() {
                Some(bak)
            } else if single_account && legacy_bak.exists() {
                Some(legacy_bak.clone())
            } else {
                None
            };

            if let Some(src) = source_bak {
                if let Err(e) = std::fs::copy(&src, &profile.video_cfg_path) {
                    eprintln!(
                        "❌ Не удалось восстановить {} из {}: {e}",
                        profile.video_cfg_path.display(),
                        src.display()
                    );
                    std::process::exit(1);
                }
                println!(
                    "  ✓ video.txt восстановлен для аккаунта {}",
                    profile.account_id
                );
            }
        }
        println!("  ✓ Исходные конфигурации восстановлены.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_backup_path_is_isolated_per_account() {
        let root = PathBuf::from("/tmp/aura_test");
        let p1 = account_backup_path(&root, "10001");
        let p2 = account_backup_path(&root, "20002");
        assert_ne!(p1, p2);
        assert!(p1.ends_with("userdata/10001/video.txt.bak"));
        assert!(p2.ends_with("userdata/20002/video.txt.bak"));
    }
}
