use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DotaEnvironment {
    pub steam_dir: Option<PathBuf>,
    pub dota_game_dir: PathBuf,
    pub userdata_configs: Vec<UserDataProfile>,
}

#[derive(Debug, Clone)]
pub struct UserDataProfile {
    pub account_id: String,
    pub video_cfg_path: PathBuf,
}

impl DotaEnvironment {
    pub fn detect_with_overrides(
        dota_dir_override: Option<&Path>,
        steam_dir_override: Option<&Path>,
    ) -> Result<Self, String> {
        let steam_dir = match steam_dir_override {
            Some(p) if p.exists() => Some(p.to_path_buf()),
            Some(p) => {
                return Err(format!(
                    "Указанная директория Steam не существует: {}",
                    p.display()
                ));
            }
            None => detect_steam_path(),
        };

        let dota_game_dir = match dota_dir_override {
            Some(p) => resolve_dota_game_dir_override(p)?,
            None => {
                let s_dir = steam_dir.as_deref().ok_or_else(|| {
                    "Не удалось обнаружить директорию установки Steam (укажите --steam-dir или --dota-dir)".to_string()
                })?;
                find_dota_game_dir(s_dir)
                    .ok_or_else(|| "Dota 2 не найдена в библиотеках Steam".to_string())?
            }
        };

        let userdata_configs = steam_dir
            .as_deref()
            .map(find_userdata_profiles)
            .unwrap_or_default();

        Ok(Self {
            steam_dir,
            dota_game_dir,
            userdata_configs,
        })
    }

    pub fn cfg_dir(&self) -> PathBuf {
        self.dota_game_dir.join("cfg")
    }
}

fn resolve_dota_game_dir_override(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err(format!(
            "Указанная директория Dota 2 не существует: {}",
            path.display()
        ));
    }

    let nested = path.join("game").join("dota");
    if nested.exists() {
        return Ok(nested);
    }

    let direct_dota = path.join("dota");
    if path.file_name().and_then(|n| n.to_str()) == Some("game") && direct_dota.exists() {
        return Ok(direct_dota);
    }

    if path.join("cfg").exists() || path.join("gameinfo.gi").exists() {
        return Ok(path.to_path_buf());
    }

    Ok(path.to_path_buf())
}

pub fn detect_steam_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let candidates = [
            home.join(".local/share/Steam"),
            home.join(".steam/steam"),
            home.join(".steam/root"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
            home.join("snap/steam/common/.local/share/Steam"),
        ];

        for path in candidates {
            if path.exists() && path.join("steamapps").exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut candidates = Vec::new();

        if let Ok(out) = std::process::Command::new("reg")
            .args(["query", "HKCU\\Software\\Valve\\Steam", "/v", "SteamPath"])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    if line.contains("SteamPath") {
                        if let Some(val) = line.split("REG_SZ").nth(1) {
                            let normalized = val.trim().replace('/', "\\");
                            let p = PathBuf::from(normalized);
                            if p.exists() {
                                candidates.push(p);
                            }
                        }
                    }
                }
            }
        }

        if let Some(pf86) = std::env::var_os("PROGRAMFILES(X86)") {
            candidates.push(PathBuf::from(pf86).join("Steam"));
        }
        if let Some(pf) = std::env::var_os("PROGRAMFILES") {
            candidates.push(PathBuf::from(pf).join("Steam"));
        }

        candidates.extend([
            PathBuf::from("C:\\Program Files (x86)\\Steam"),
            PathBuf::from("C:\\Program Files\\Steam"),
            PathBuf::from("D:\\Steam"),
            PathBuf::from("E:\\Steam"),
            PathBuf::from("D:\\Games\\Steam"),
            PathBuf::from("E:\\Games\\Steam"),
        ]);

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
                if video_cfg.exists() {
                    profiles.push(UserDataProfile {
                        account_id,
                        video_cfg_path: video_cfg,
                    });
                }
            }
        }
    }

    profiles.sort_by(|a, b| a.account_id.cmp(&b.account_id));
    profiles
}

/// Extract quoted strings on a single VDF line, honoring `\\` and `\"` escapes.
pub(crate) fn extract_vdf_quoted_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if c == '"' {
            let mut buf = String::new();
            while let Some(inner) = chars.next() {
                if inner == '\\' {
                    if let Some(escaped) = chars.next() {
                        match escaped {
                            '\\' => buf.push('\\'),
                            '"' => buf.push('"'),
                            'n' => buf.push('\n'),
                            't' => buf.push('\t'),
                            other => {
                                buf.push('\\');
                                buf.push(other);
                            }
                        }
                    }
                } else if inner == '"' {
                    break;
                } else {
                    buf.push(inner);
                }
            }
            tokens.push(buf);
        }
    }

    tokens
}

pub fn parse_library_folders(vdf_content: &str) -> Vec<String> {
    let mut dota_paths = Vec::new();
    let mut all_paths = Vec::new();
    let mut depth: usize = 0;
    let mut current_path: Option<String> = None;
    let mut has_dota = false;

    for raw_line in vdf_content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed == "{" {
            depth += 1;
            continue;
        }

        if trimmed == "}" {
            if depth == 2 {
                if has_dota {
                    if let Some(p) = current_path.take() {
                        dota_paths.push(p);
                    }
                } else {
                    current_path = None;
                }
                has_dota = false;
            }
            depth = depth.saturating_sub(1);
            continue;
        }

        let tokens = extract_vdf_quoted_tokens(trimmed);
        if depth == 2 && tokens.len() >= 2 && tokens[0].eq_ignore_ascii_case("path") {
            let path_val = tokens[1].clone();
            all_paths.push(path_val.clone());
            current_path = Some(path_val);
        } else if depth >= 3 && !tokens.is_empty() && tokens[0] == "570" {
            has_dota = true;
        }
    }

    if !dota_paths.is_empty() {
        dota_paths
    } else {
        all_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_library_folders_handles_nested_apps_and_escapes() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"apps"
		{
			"730"		"12345"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"apps"
		{
			"570"		"56789"
		}
	}
}
"#;
        let parsed = parse_library_folders(vdf);
        assert_eq!(parsed, vec!["D:\\SteamLibrary"]);
    }

    #[test]
    fn test_parse_library_folders_falls_back_when_570_absent() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/home/user/.local/share/Steam"
		"apps"
		{
			"730"		"123"
		}
	}
}
"#;
        let parsed = parse_library_folders(vdf);
        assert_eq!(parsed, vec!["/home/user/.local/share/Steam"]);
    }

    #[test]
    fn test_detect_with_overrides_finds_profiles_and_dota() {
        let tmp = tempdir().unwrap();
        let steam = tmp.path().join("Steam");
        let dota_game = steam.join("steamapps/common/dota 2 beta/game/dota");
        fs::create_dir_all(dota_game.join("cfg")).unwrap();
        let vcfg = steam.join("userdata/123456/570/local/cfg/video.txt");
        fs::create_dir_all(vcfg.parent().unwrap()).unwrap();
        fs::write(&vcfg, "\"video.cfg\"\n{\n}\n").unwrap();

        let env = DotaEnvironment::detect_with_overrides(None, Some(&steam)).unwrap();
        assert_eq!(env.dota_game_dir, dota_game);
        assert_eq!(env.userdata_configs.len(), 1);
        assert_eq!(env.userdata_configs[0].account_id, "123456");
    }
}
