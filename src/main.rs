#![allow(dead_code)]
use clap::Parser;
use std::path::PathBuf;

mod dota;
use dota::*;

#[derive(Parser, Debug)]
#[command(name = "aura-cli", about = "✦ Aura Community CLI — Open-source Dota 2 Configuration Optimizer")]
struct Cli {
    /// Apply an optimization preset: competitive, balanced, or potato
    #[arg(short, long)]
    apply: Option<String>,

    /// Restore all files and configurations from backup
    #[arg(short, long)]
    restore: bool,

    /// Clean shader and temporary cache files
    #[arg(short, long)]
    clean_cache: bool,

    /// Print current detected Dota 2 environment and settings status
    #[arg(short, long)]
    status: bool,
}

fn get_backup_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/aura/backups")
    } else {
        std::env::temp_dir().join("aura_backups")
    }
}

fn main() {
    let cli = Cli::parse();
    let backup_dir = get_backup_dir();

    println!("✦ AURA COMMUNITY CLI (Open-Source Edition)");
    println!("  100% VAC-Safe Configuration Tuning (video.txt & autoexec.cfg)\n");

    let env = match DotaEnvironment::detect() {
        Ok(e) => e,
        Err(err) => {
            eprintln!("❌ Ошибка обнаружения Dota 2: {err}");
            eprintln!("   Убедитесь, что Steam и Dota 2 установлены в стандартную директорию.");
            std::process::exit(1);
        }
    };

    if cli.status || (cli.apply.is_none() && !cli.restore && !cli.clean_cache) {
        println!("  Директория игры: {}", env.dota_game_dir.display());
        println!("  Steam:           {}", env.steam_dir.display());
        println!("  Аккаунты:        {} обнаружено", env.userdata_configs.len());
        println!("  autoexec.cfg:    {}", if is_autoexec_applied(&env.dota_game_dir.join("cfg")) { "Активен" } else { "Не настроен" });
        for profile in &env.userdata_configs {
            if let Ok(cfg) = VideoConfig::load_from_file(&profile.video_cfg_path) {
                println!("  video.txt [{}] : {}", profile.account_id, if cfg.is_competitive_optimized() { "Оптимизирован" } else { "Стандартный" });
            }
        }
        println!("\n  ℹ️ Для расширенных функций (Panorama WebM нейтрализатор, No-Bling и нативный GUI)");
        println!("     используйте Aura Pro: http://localhost:4242");
    }

    if let Some(ref preset) = cli.apply {
        println!("  Применение пресета '{}'...", preset);
        for profile in &env.userdata_configs {
            if let Ok(mut cfg) = VideoConfig::load_from_file(&profile.video_cfg_path) {
                match preset.to_lowercase().as_str() {
                    "competitive" | "comp" => cfg.apply_competitive_preset(),
                    "balanced" => cfg.apply_balanced_preset(),
                    "potato" => cfg.apply_potato_preset(),
                    other => {
                        eprintln!("  Неизвестный пресет: {other}. Доступные: competitive, balanced, potato.");
                        return;
                    }
                }
                let _ = cfg.save_to_file(&profile.video_cfg_path, Some(&backup_dir));
                println!("  ✓ video.txt обновлен для аккаунта {}", profile.account_id);
            }
        }
        let _ = apply_autoexec(&env.dota_game_dir.join("cfg"), AutoexecPreset::Competitive, Some(&backup_dir));
        println!("  ✓ autoexec.cfg успешно записан (сетевые твики и интерполяция).");
        println!("\n✓ Оптимизация успешно завершена!");
    }

    if cli.clean_cache {
        println!("  Очистка кэша Dota 2...");
        let report = DotaCleaner::scan(&env.dota_game_dir, &env.steam_dir);
        let mb = report.total_mb();
        let _ = DotaCleaner::purge_dumps(&report);
        let _ = DotaCleaner::purge_logs(&report);
        let _ = DotaCleaner::purge_shader_cache(&report);
        println!("  ✓ Освобождено {:.1} МБ дискового пространства.", mb);
    }

    if cli.restore {
        println!("  Восстановление всех файлов из бэкапа...");
        let _ = restore_autoexec(&env.dota_game_dir.join("cfg"), &backup_dir);
        for profile in &env.userdata_configs {
            let bak = backup_dir.join("video.txt.bak");
            if bak.exists() {
                let _ = std::fs::copy(&bak, &profile.video_cfg_path);
            }
        }
        println!("  ✓ Исходные конфигурации восстановлены.");
    }
}
