use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DotaEnvironment {
    pub steam_dir: PathBuf,
    pub dota_game_dir: PathBuf,
    pub userdata_configs: Vec<UserDataProfile>,
}

#[derive(Debug, Clone)]
pub struct UserDataProfile {
    pub account_id: String,
    pub video_cfg_path: PathBuf,
}

impl DotaEnvironment {
    pub fn detect() -> Result<Self, String> {
        let steam_dir = detect_steam_path()
            .ok_or_else(|| "Не удалось обнаружить директорию установки Steam".to_string())?;

        let dota_game_dir = find_dota_game_dir(&steam_dir)
            .ok_or_else(|| "Dota 2 не найдена в библиотеках Steam".to_string())?;

        let userdata_configs = find_userdata_profiles(&steam_dir);

        Ok(Self {
            steam_dir,
            dota_game_dir,
            userdata_configs,
        })
    }

    pub fn panorama_videos_dir(&self) -> PathBuf {
        self.dota_game_dir.join("panorama").join("videos")
    }

    pub fn gameinfo_path(&self) -> PathBuf {
        self.dota_game_dir.join("gameinfo.gi")
    }

    pub fn autoexec_path(&self) -> PathBuf {
        self.dota_game_dir.join("cfg").join("autoexec.cfg")
    }

    pub fn tempcontent_dir(&self) -> PathBuf {
        // dota_tempcontent is placed in game/
        self.dota_game_dir
            .parent()
            .unwrap_or(&self.dota_game_dir)
            .join("dota_tempcontent")
    }
}

pub fn detect_steam_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        let candidates = [
            PathBuf::from(&home).join(".local/share/Steam"),
            PathBuf::from(&home).join(".steam/steam"),
            PathBuf::from(&home).join(".steam/root"),
        ];

        for path in candidates {
            if path.exists() && path.join("steamapps").exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let candidates = [
            PathBuf::from("C:\\Program Files (x86)\\Steam"),
            PathBuf::from("C:\\Program Files\\Steam"),
            PathBuf::from("D:\\Steam"),
        ];
        for path in candidates {
            if path.exists() && path.join("steamapps").exists() {
                return Some(path);
            }
        }
    }

    None
}

pub fn find_dota_game_dir(steam_dir: &Path) -> Option<PathBuf> {
    let library_vdf = steam_dir.join("steamapps").join("libraryfolders.vdf");
    if !library_vdf.exists() {
        // Fallback to steam_dir/steamapps/common/dota 2 beta/game/dota
        let direct = steam_dir
            .join("steamapps")
            .join("common")
            .join("dota 2 beta")
            .join("game")
            .join("dota");
        if direct.exists() {
            return Some(direct);
        }
        return None;
    }

    let content = fs::read_to_string(&library_vdf).ok()?;
    let libraries = parse_library_folders(&content);

    for lib in libraries {
        let dota_path = PathBuf::from(lib)
            .join("steamapps")
            .join("common")
            .join("dota 2 beta")
            .join("game")
            .join("dota");
        if dota_path.exists() {
            return Some(dota_path);
        }
    }

    None
}

pub fn find_userdata_profiles(steam_dir: &Path) -> Vec<UserDataProfile> {
    let mut profiles = Vec::new();
    let userdata = steam_dir.join("userdata");
    if !userdata.exists() {
        return profiles;
    }

    if let Ok(entries) = fs::read_dir(userdata) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let account_id = entry.file_name().to_string_lossy().to_string();
                if account_id == "0" || account_id == "anonymous" {
                    continue;
                }

                let video_cfg = path.join("570").join("local").join("cfg").join("video.txt");
                let exists = video_cfg.exists();
                if exists {
                    profiles.push(UserDataProfile {
                        account_id,
                        video_cfg_path: video_cfg,
                    });
                }
            }
        }
    }

    profiles
}

fn parse_library_folders(vdf_content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut current_path: Option<String> = None;
    let mut has_dota = false;

    for line in vdf_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 4 {
                current_path = Some(parts[3].to_string());
                has_dota = false;
            }
        } else if trimmed.contains("\"570\"") {
            has_dota = true;
        } else if trimmed == "}" {
            if has_dota {
                if let Some(p) = current_path.take() {
                    paths.push(p);
                }
            }
        }
    }

    if paths.is_empty() {
        // Fallback: extract all "path" values
        for line in vdf_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"path\"") {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() >= 4 {
                    paths.push(parts[3].to_string());
                }
            }
        }
    }

    paths
}
