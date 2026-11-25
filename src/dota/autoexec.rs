use std::fs;
use std::path::Path;

pub const AURA_AUTOEXEC_HEADER: &str = "// === AURA DOTA 2 PERFORMANCE OPTIMIZATION ===";
pub const AURA_AUTOEXEC_FOOTER: &str = "// === END AURA DOTA 2 OPTIMIZATION ===";
const LEGACY_AUTOEXEC_FOOTER: &str = "// =============================================";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoexecPreset {
    Basic,
}

impl AutoexecPreset {
    pub fn generate_content(&self) -> String {
        match self {
            Self::Basic => format!(
                "{AURA_AUTOEXEC_HEADER}\n\
// [1. Basic Network & Threading]\n\
rate \"800000\"\n\
cl_interp_ratio \"1\"\n\
cl_smooth \"0\"\n\
r_threaded_particles \"1\"\n\
engine_no_focus_sleep \"0\"\n\
\n\
// [2. Framerate]\n\
fps_max \"0\"\n\
{AURA_AUTOEXEC_FOOTER}\n"
            ),
        }
    }
}

pub fn is_autoexec_applied(cfg_dir: &Path) -> bool {
    let autoexec = cfg_dir.join("autoexec.cfg");
    if !autoexec.exists() {
        return false;
    }
    if let Ok(content) = fs::read_to_string(&autoexec) {
        return content.contains(AURA_AUTOEXEC_HEADER);
    }
    false
}

fn strip_aura_block(raw_content: &str) -> String {
    let normalized = raw_content.replace("\r\n", "\n");
    let Some(start_idx) = normalized.find(AURA_AUTOEXEC_HEADER) else {
        return normalized;
    };

    let prefix = normalized[..start_idx].trim_end_matches('\n');
    let rest = &normalized[start_idx + AURA_AUTOEXEC_HEADER.len()..];

    let after = if let Some(idx) = rest.find(AURA_AUTOEXEC_FOOTER) {
        &rest[idx + AURA_AUTOEXEC_FOOTER.len()..]
    } else if let Some(idx) = rest.find(LEGACY_AUTOEXEC_FOOTER) {
        &rest[idx + LEGACY_AUTOEXEC_FOOTER.len()..]
    } else {
        ""
    };
    let suffix = after.trim_start_matches('\n');

    match (prefix.is_empty(), suffix.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("{prefix}\n"),
        (true, false) => suffix.to_string(),
        (false, false) => format!("{prefix}\n\n{suffix}"),
    }
}

pub fn apply_autoexec(
    cfg_dir: &Path,
    preset: AutoexecPreset,
    backup_dir: Option<&Path>,
) -> Result<(), String> {
    fs::create_dir_all(cfg_dir).map_err(|e| format!("Не удалось создать директорию cfg: {e}"))?;

    let autoexec = cfg_dir.join("autoexec.cfg");

    if let Some(b_dir) = backup_dir {
        fs::create_dir_all(b_dir)
            .map_err(|e| format!("Не удалось создать директорию бэкапа: {e}"))?;
        let backup_file = b_dir.join("autoexec.cfg.bak");
        if autoexec.exists() && !backup_file.exists() {
            fs::copy(&autoexec, backup_file)
                .map_err(|e| format!("Не удалось создать бэкап autoexec.cfg: {e}"))?;
        }
    }

    let current_content = if autoexec.exists() {
        fs::read_to_string(&autoexec)
            .map_err(|e| format!("Не удалось прочитать autoexec.cfg: {e}"))?
    } else {
        String::new()
    };

    let mut clean = strip_aura_block(&current_content);
    let new_block = preset.generate_content();

    if !clean.is_empty() && !clean.ends_with('\n') {
        clean.push('\n');
    }
    clean.push_str(&new_block);

    fs::write(&autoexec, clean).map_err(|e| format!("Не удалось сохранить autoexec.cfg: {e}"))?;

    Ok(())
}

pub fn restore_autoexec(cfg_dir: &Path, backup_dir: &Path) -> Result<(), String> {
    let autoexec = cfg_dir.join("autoexec.cfg");
    let backup_file = backup_dir.join("autoexec.cfg.bak");

    if backup_file.exists() {
        fs::copy(&backup_file, &autoexec)
            .map_err(|e| format!("Не удалось восстановить autoexec.cfg: {e}"))?;
    } else if autoexec.exists() {
        let content = fs::read_to_string(&autoexec)
            .map_err(|e| format!("Не удалось прочитать autoexec.cfg: {e}"))?;
        if content.contains(AURA_AUTOEXEC_HEADER) {
            let stripped = strip_aura_block(&content);
            if stripped.trim().is_empty() {
                fs::remove_file(&autoexec)
                    .map_err(|e| format!("Не удалось удалить autoexec.cfg: {e}"))?;
            } else {
                fs::write(&autoexec, stripped)
                    .map_err(|e| format!("Не удалось обновить autoexec.cfg: {e}"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_apply_reapply_crlf_and_restore_preserves_user_lines() {
        let tmp = tempdir().unwrap();
        let cfg = tmp.path().join("cfg");
        let bak = tmp.path().join("backups");
        fs::create_dir_all(&cfg).unwrap();

        let autoexec = cfg.join("autoexec.cfg");
        fs::write(&autoexec, "// user top\r\nbind \"F5\" \"say_team hi\"\r\n").unwrap();

        apply_autoexec(&cfg, AutoexecPreset::Basic, Some(&bak)).unwrap();
        assert!(is_autoexec_applied(&cfg));

        // Append user line below and normalize CRLF re-apply
        let with_bottom = format!(
            "{}\r\n// user bottom\r\n",
            fs::read_to_string(&autoexec).unwrap()
        );
        fs::write(&autoexec, with_bottom).unwrap();

        apply_autoexec(&cfg, AutoexecPreset::Basic, Some(&bak)).unwrap();
        let final_txt = fs::read_to_string(&autoexec).unwrap();
        assert!(final_txt.contains("// user top"));
        assert!(final_txt.contains("// user bottom"));
        assert_eq!(final_txt.matches(AURA_AUTOEXEC_HEADER).count(), 1);

        restore_autoexec(&cfg, &bak).unwrap();
        let restored = fs::read_to_string(&autoexec).unwrap();
        assert!(!restored.contains(AURA_AUTOEXEC_HEADER));
        assert!(restored.contains("bind \"F5\" \"say_team hi\""));
    }

    #[test]
    fn test_restore_without_backup_removes_empty_or_strips_block() {
        let tmp = tempdir().unwrap();
        let cfg = tmp.path().join("cfg");
        let bak = tmp.path().join("empty_bak");
        fs::create_dir_all(&cfg).unwrap();

        apply_autoexec(&cfg, AutoexecPreset::Basic, None).unwrap();
        assert!(cfg.join("autoexec.cfg").exists());
        restore_autoexec(&cfg, &bak).unwrap();
        assert!(!cfg.join("autoexec.cfg").exists());

        // With existing custom lines and legacy footer
        let legacy = format!(
            "con_enable \"1\"\n{AURA_AUTOEXEC_HEADER}\nrate \"800000\"\n{LEGACY_AUTOEXEC_FOOTER}\n"
        );
        fs::write(cfg.join("autoexec.cfg"), legacy).unwrap();
        restore_autoexec(&cfg, &bak).unwrap();
        let remaining = fs::read_to_string(cfg.join("autoexec.cfg")).unwrap();
        assert_eq!(remaining, "con_enable \"1\"\n");
    }
}
