use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub root_name: String,
    pub settings: BTreeMap<String, String>,
}

impl VideoConfig {
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut settings = BTreeMap::new();
        let mut root_name = "video.cfg".to_string();

        let mut lines = content.lines();
        for line in lines.by_ref() {
            let trimmed = line.trim();
            if trimmed.starts_with('"') && !trimmed.contains('\t') && !trimmed.contains("  ") {
                root_name = trimmed.trim_matches('"').to_string();
                break;
            }
        }

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed == "{" || trimmed == "}" || trimmed == format!("\"{}\"", root_name) {
                continue;
            }

            // Lines are like: "setting.cl_globallight_shadow_mode"		"0"
            let parts: Vec<&str> = trimmed
                .split('"')
                .filter(|s| !s.trim().is_empty())
                .collect();

            if parts.len() >= 2 {
                let key = parts[0].to_string();
                let val = parts[1].to_string();
                settings.insert(key, val);
            }
        }

        Ok(Self {
            root_name,
            settings,
        })
    }

    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Не удалось прочитать {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    pub fn save_to_file(&self, path: &Path, backup_dir: Option<&Path>) -> Result<(), String> {
        if let Some(b_dir) = backup_dir {
            let _ = fs::create_dir_all(b_dir);
            let backup_file = b_dir.join("video.txt.bak");
            if path.exists() && !backup_file.exists() {
                let _ = fs::copy(path, backup_file);
            }
        }

        let formatted = self.format_vdf();
        fs::write(path, formatted)
            .map_err(|e| format!("Не удалось сохранить {}: {e}", path.display()))?;

        Ok(())
    }

    pub fn format_vdf(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("\"{}\"\n{{\n", self.root_name));
        for (key, val) in &self.settings {
            out.push_str(&format!("\t\"{}\"\t\t\"{}\"\n", key, val));
        }
        out.push_str("}\n");
        out
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }

    pub fn apply_competitive_preset(&mut self) {
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
        self.set("setting.cl_particle_fallback_base", "4");
        self.set("setting.cl_particle_fallback_multiplier", "2");
        self.set("setting.r_dashboard_render_quality", "false");
        self.set("setting.r_dota_fsr_upsample", "1");
        self.set("setting.mat_vsync", "0");
        self.set("setting.useadvanced", "1");
    }

    pub fn apply_balanced_preset(&mut self) {
        self.apply_competitive_preset();
        // Keep textures higher
        self.set("setting.r_texture_stream_mip_bias", "0");
        self.set("setting.shaderquality", "1");
    }

    pub fn apply_potato_preset(&mut self) {
        self.apply_competitive_preset();
        self.set("setting.mat_viewportscale", "0.7");
        self.set("setting.shaderquality", "0");
        self.set("setting.gpu_level", "0");
        self.set("setting.cpu_level", "0");
        self.set("setting.r_texture_stream_mip_bias", "2");
    }

    pub fn is_competitive_optimized(&self) -> bool {
        self.get("setting.cl_globallight_shadow_mode").map(|v| v == "0").unwrap_or(false)
            && self.get("setting.dota_cheap_water").map(|v| v == "true").unwrap_or(false)
            && self.get("setting.dota_portrait_animate").map(|v| v == "false").unwrap_or(false)
            && self.get("setting.r_dota_allow_wind_on_trees").map(|v| v == "false").unwrap_or(false)
    }
}
