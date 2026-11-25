use std::fs;
use std::path::Path;

use crate::dota::paths::extract_vdf_quoted_tokens;

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub root_name: String,
    pub settings: Vec<(String, String)>,
}

impl VideoConfig {
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut settings = Vec::new();
        let mut root_name: Option<String> = None;

        for raw_line in content.lines() {
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed == "{" || trimmed == "}" || trimmed.starts_with("//") {
                continue;
            }

            let tokens = extract_vdf_quoted_tokens(trimmed);
            if tokens.len() == 1 && root_name.is_none() {
                root_name = Some(tokens[0].clone());
            } else if tokens.len() >= 2 {
                let key = tokens[0].clone();
                let val = tokens[1].clone();
                if let Some((_, existing)) = settings.iter_mut().find(|(k, _)| k == &key) {
                    *existing = val;
                } else {
                    settings.push((key, val));
                }
            }
        }

        Ok(Self {
            root_name: root_name.unwrap_or_else(|| "video.cfg".to_string()),
            settings,
        })
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    pub fn save_to_file(&self, path: &Path, backup_path: Option<&Path>) -> Result<(), String> {
        if let Some(bak) = backup_path {
            if let Some(parent) = bak.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Не удалось создать папку бэкапа {}: {e}", parent.display())
                })?;
            }
            if path.exists() && !bak.exists() {
                fs::copy(path, bak)
                    .map_err(|e| format!("Не удалось создать бэкап {}: {e}", bak.display()))?;
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Не удалось создать директорию {}: {e}", parent.display()))?;
        }

        let formatted = self.format_vdf();
        fs::write(path, formatted)
            .map_err(|e| format!("Не удалось сохранить {}: {e}", path.display()))?;

        Ok(())
    }

    pub fn format_vdf(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\"{}\"\n{{\n", escape_vdf(&self.root_name)));
        for (key, val) in &self.settings {
            out.push_str(&format!(
                "\t\"{}\"\t\t\"{}\"\n",
                escape_vdf(key),
                escape_vdf(val)
            ));
        }
        out.push_str("}\n");
        out
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        if let Some((_, existing)) = self.settings.iter_mut().find(|(k, _)| k == key) {
            *existing = value.to_string();
        } else {
            self.settings.push((key.to_string(), value.to_string()));
        }
    }

    /// Basic, safe configuration preset available in Community CLI.
    pub fn apply_basic_preset(&mut self) {
        self.set("setting.cl_globallight_shadow_mode", "0");
        self.set("setting.dota_cheap_water", "true");
        self.set("setting.r_ssao", "false");
        self.set("setting.dota_portrait_animate", "false");
        self.set("setting.dota_ambient_creatures", "false");
        self.set("setting.dota_ambient_cloth", "false");
        self.set("setting.r_dota_allow_wind_on_trees", "false");
        self.set("setting.r_dota_fxaa", "false");
        self.set("setting.r_deferred_specular_bloom", "false");
        self.set("setting.r_deferred_specular", "false");
        self.set("setting.r_deferred_height_fog", "false");
        self.set("setting.r_grass_quality", "0");
        self.set("setting.mat_vsync", "0");
        self.set("setting.useadvanced", "1");
    }

    pub fn is_basic_optimized(&self) -> bool {
        self.get("setting.cl_globallight_shadow_mode") == Some("0")
            && self.get("setting.dota_cheap_water") == Some("true")
            && self.get("setting.dota_portrait_animate") == Some("false")
            && self.get("setting.r_dota_allow_wind_on_trees") == Some("false")
            && self.get("setting.mat_vsync") == Some("0")
    }
}

fn escape_vdf(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_preserves_empty_values_spaces_and_order() {
        let input = "\"video.cfg\"\r\n{\r\n\t\"setting.gpu_family\"\t\t\"\"\r\n\t\"setting.cl_globallight_shadow_mode\"\t\t\"2\"\r\n\t\"setting.custom_note\"\t\t\"hello world\"\r\n}\r\n";
        let cfg = VideoConfig::parse(input).unwrap();
        assert_eq!(cfg.root_name, "video.cfg");
        assert_eq!(cfg.get("setting.gpu_family"), Some(""));
        assert_eq!(cfg.get("setting.cl_globallight_shadow_mode"), Some("2"));
        assert_eq!(cfg.get("setting.custom_note"), Some("hello world"));

        let out = cfg.format_vdf();
        let reparsed = VideoConfig::parse(&out).unwrap();
        assert_eq!(reparsed.settings, cfg.settings);
    }

    #[test]
    fn test_per_account_backup_does_not_clobber() {
        let tmp = tempdir().unwrap();
        let acc1 = tmp.path().join("userdata/111/570/local/cfg/video.txt");
        let acc2 = tmp.path().join("userdata/222/570/local/cfg/video.txt");
        let bak1 = tmp.path().join("backups/userdata/111/video.txt.bak");
        let bak2 = tmp.path().join("backups/userdata/222/video.txt.bak");

        fs::create_dir_all(acc1.parent().unwrap()).unwrap();
        fs::create_dir_all(acc2.parent().unwrap()).unwrap();
        fs::write(
            &acc1,
            "\"video.cfg\"\n{\n\t\"setting.defaultres\"\t\t\"1920\"\n}\n",
        )
        .unwrap();
        fs::write(
            &acc2,
            "\"video.cfg\"\n{\n\t\"setting.defaultres\"\t\t\"2560\"\n}\n",
        )
        .unwrap();

        let mut c1 = VideoConfig::load_from_file(&acc1).unwrap();
        c1.apply_basic_preset();
        c1.save_to_file(&acc1, Some(&bak1)).unwrap();

        let mut c2 = VideoConfig::load_from_file(&acc2).unwrap();
        c2.apply_basic_preset();
        c2.save_to_file(&acc2, Some(&bak2)).unwrap();

        assert!(c1.is_basic_optimized());
        assert!(c2.is_basic_optimized());

        let restored1 = fs::read_to_string(&bak1).unwrap();
        let restored2 = fs::read_to_string(&bak2).unwrap();
        assert!(restored1.contains("1920"));
        assert!(!restored1.contains("2560"));
        assert!(restored2.contains("2560"));
        assert!(!restored2.contains("1920"));
    }
}
