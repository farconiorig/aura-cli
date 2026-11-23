use std::fs;
use std::path::Path;

pub const AURA_AUTOEXEC_HEADER: &str = "// === AURA DOTA 2 PERFORMANCE OPTIMIZATION ===";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AutoexecPreset {
    Competitive,
    Potato,
    HighRefresh,
}

impl AutoexecPreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Competitive => "Киберспортивный (Competitive)",
            Self::Potato => "Ultra FPS Potato (Макс. 1% Low)",
            Self::HighRefresh => "High Refresh Rate (120Hz/144Hz+)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Competitive => "Rate 800000, interp 0.025, r_threaded_particles 1, snd_mixahead 0.05, panorama_fast_layout 1.",
            Self::Potato => "Форсированное глушение партиклов (fallback 4), дешевая вода, 0 трупов, 2D портреты, 0 живности.",
            Self::HighRefresh => "Синхронизация тикрейта под 120/144Hz (interp 0.0166), rate 800000, fps_max 0, минимальный джиттер.",
        }
    }

    pub fn generate_content(&self) -> String {
        let mut s = String::new();
        s.push_str(AURA_AUTOEXEC_HEADER);
        s.push('\n');
        match self {
            Self::Competitive => {
                s.push_str(r#"// [1. Network & Interpolation]
rate "800000"
cl_interp "0.025"
cl_interp_ratio "1"
cl_updaterate "60"
cl_cmdrate "60"
cl_smooth "0"
cl_smoothtime "0.01"
net_maxroutable "1200"

// [2. Source 2 Engine & Threading]
r_threaded_particles "1"
mat_queue_mode "2"
engine_no_focus_sleep "0"

// [3. Panorama UI & Dashboard]
panorama_fast_layout "1"
cl_dashboard_precache_option "2"
dota_dashboard_game_priority "1"

// [4. Low Latency Audio Stack]
snd_mixahead "0.05"
snd_async_fullyasync "1"

// [5. Framerate]
fps_max "0"
"#);
            }
            Self::Potato => {
                s.push_str(r#"// [1. Network & Interpolation]
rate "800000"
cl_interp "0.025"
cl_interp_ratio "1"
cl_updaterate "60"
cl_cmdrate "60"
cl_smooth "0"
net_maxroutable "1200"

// [2. Aggressive GPU & CPU Cuts]
r_threaded_particles "1"
cl_particle_fallback_base "4"
cl_particle_fallback_multiplier "0"
dota_cheap_water "1"
r_drawflecks "0"
cl_ragdoll_limit "0"
r_drawmodeldecals "0"
dota_portrait_animate "0"
dota_ambient_creatures "0"
dota_ambient_creatures_pop "0"
engine_no_focus_sleep "0"

// [3. Panorama UI Performance]
panorama_fast_layout "1"
cl_dashboard_precache_option "2"
dota_dashboard_game_priority "1"

// [4. Audio Buffer]
snd_mixahead "0.05"
snd_async_fullyasync "1"

// [5. Framerate]
fps_max "0"
"#);
            }
            Self::HighRefresh => {
                s.push_str(r#"// [1. High Refresh Rate Tickrate & Interpolation]
rate "800000"
cl_interp "0.0166"
cl_interp_ratio "1"
cl_updaterate "60"
cl_cmdrate "60"
cl_smooth "0"
cl_smoothtime "0.005"
net_maxroutable "1200"

// [2. Source 2 Threading & Latency]
r_threaded_particles "1"
mat_queue_mode "2"
engine_no_focus_sleep "0"

// [3. Panorama UI]
panorama_fast_layout "1"
cl_dashboard_precache_option "2"

// [4. Audio Buffer]
snd_mixahead "0.04"
snd_async_fullyasync "1"

// [5. Framerate]
fps_max "0"
"#);
            }
        }
        s.push_str("// =============================================\n");
        s
    }
}

pub fn is_autoexec_applied(cfg_dir: &Path) -> bool {
    let autoexec = cfg_dir.join("autoexec.cfg");
    if !autoexec.exists() {
        return false;
    }
    if let Ok(content) = fs::read_to_string(&autoexec) {
        return content.contains(AURA_AUTOEXEC_HEADER) || content.contains("panorama_fast_layout");
    }
    false
}

pub fn apply_autoexec(
    cfg_dir: &Path,
    preset: AutoexecPreset,
    backup_dir: Option<&Path>,
) -> Result<(), String> {
    fs::create_dir_all(cfg_dir)
        .map_err(|e| format!("Не удалось создать директорию cfg: {e}"))?;

    let autoexec = cfg_dir.join("autoexec.cfg");

    if let Some(b_dir) = backup_dir {
        let _ = fs::create_dir_all(b_dir);
        let backup_file = b_dir.join("autoexec.cfg.bak");
        if autoexec.exists() && !backup_file.exists() {
            let _ = fs::copy(&autoexec, backup_file);
        }
    }

    let current_content = if autoexec.exists() {
        fs::read_to_string(&autoexec).unwrap_or_default()
    } else {
        String::new()
    };

    let new_block = preset.generate_content();

    let final_content = if let Some(start_idx) = current_content.find(AURA_AUTOEXEC_HEADER) {
        // Strip out existing aura block
        let mut clean = current_content[..start_idx].to_string();
        if let Some(end_marker) = current_content[start_idx..].find("// =============================================\n") {
            let after = &current_content[start_idx + end_marker + 49..];
            clean.push_str(after.trim_start());
        }
        if !clean.is_empty() && !clean.ends_with('\n') {
            clean.push('\n');
        }
        clean.push_str(&new_block);
        clean
    } else {
        let mut clean = current_content;
        if !clean.is_empty() && !clean.ends_with('\n') {
            clean.push('\n');
        }
        clean.push_str(&new_block);
        clean
    };

    fs::write(&autoexec, final_content)
        .map_err(|e| format!("Не удалось сохранить autoexec.cfg: {e}"))?;

    Ok(())
}

pub fn restore_autoexec(cfg_dir: &Path, backup_dir: &Path) -> Result<(), String> {
    let autoexec = cfg_dir.join("autoexec.cfg");
    let backup_file = backup_dir.join("autoexec.cfg.bak");

    if backup_file.exists() {
        fs::copy(&backup_file, &autoexec)
            .map_err(|e| format!("Не удалось восстановить autoexec.cfg: {e}"))?;
    } else if autoexec.exists() {
        if let Ok(content) = fs::read_to_string(&autoexec) {
            if content.starts_with(AURA_AUTOEXEC_HEADER) {
                let _ = fs::remove_file(&autoexec);
            }
        }
    }
    Ok(())
}
