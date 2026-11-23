use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct CleanReport {
    pub dump_files: Vec<PathBuf>,
    pub dump_size_bytes: u64,
    pub log_files: Vec<PathBuf>,
    pub log_size_bytes: u64,
    pub shader_cache_dirs: Vec<PathBuf>,
    pub shader_size_bytes: u64,
}

impl CleanReport {
    pub fn total_bytes(&self) -> u64 {
        self.dump_size_bytes + self.log_size_bytes + self.shader_size_bytes
    }

    pub fn total_mb(&self) -> f64 {
        self.total_bytes() as f64 / (1024.0 * 1024.0)
    }
}

pub struct DotaCleaner;

impl DotaCleaner {
    pub fn scan(dota_game_dir: &Path, steam_dir: &Path) -> CleanReport {
        let mut report = CleanReport::default();

        // 1. Dumps (*.dmp)
        let dumps_dir = dota_game_dir.join("dumps");
        if dumps_dir.exists() {
            Self::collect_files_by_ext(&dumps_dir, "dmp", &mut report.dump_files, &mut report.dump_size_bytes);
        }
        // Also check dota root for rogue .dmp files
        Self::collect_files_by_ext(dota_game_dir, "dmp", &mut report.dump_files, &mut report.dump_size_bytes);

        // 2. Logs (*.log, *.txt logs)
        let logs_dir = dota_game_dir.join("logs");
        if logs_dir.exists() {
            Self::collect_files(&logs_dir, &mut report.log_files, &mut report.log_size_bytes);
        }
        for log_candidate in ["console.log", "qconsole.log", "connection.log", "asserts.log"] {
            let p = dota_game_dir.join(log_candidate);
            if p.exists() {
                if let Ok(m) = fs::metadata(&p) {
                    report.log_size_bytes += m.len();
                    report.log_files.push(p);
                }
            }
        }

        // 3. Shader Caches (Steam AppID 570)
        let steam_shader = steam_dir.join("steamapps/shadercache/570");
        if steam_shader.exists() {
            let sz = Self::dir_size(&steam_shader);
            report.shader_size_bytes += sz;
            report.shader_cache_dirs.push(steam_shader);
        }

        if let Ok(home) = std::env::var("HOME") {
            let mesa_cache = PathBuf::from(home).join(".cache/mesa_shader_cache");
            if mesa_cache.exists() {
                let sz = Self::dir_size(&mesa_cache);
                report.shader_size_bytes += sz;
                report.shader_cache_dirs.push(mesa_cache);
            }
        }

        report
    }

    fn collect_files_by_ext(dir: &Path, ext: &str, list: &mut Vec<PathBuf>, total_size: &mut u64) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().map(|e| e == ext).unwrap_or(false) {
                    if let Ok(m) = fs::metadata(&p) {
                        *total_size += m.len();
                        list.push(p);
                    }
                }
            }
        }
    }

    fn collect_files(dir: &Path, list: &mut Vec<PathBuf>, total_size: &mut u64) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(m) = fs::metadata(&p) {
                        *total_size += m.len();
                        list.push(p);
                    }
                }
            }
        }
    }

    fn dir_size(dir: &Path) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    total += Self::dir_size(&p);
                } else if let Ok(m) = fs::metadata(&p) {
                    total += m.len();
                }
            }
        }
        total
    }

    pub fn purge_dumps(report: &CleanReport) -> usize {
        let mut count = 0;
        for file in &report.dump_files {
            if fs::remove_file(file).is_ok() {
                count += 1;
            }
        }
        count
    }

    pub fn purge_logs(report: &CleanReport) -> usize {
        let mut count = 0;
        for file in &report.log_files {
            if fs::remove_file(file).is_ok() {
                count += 1;
            }
        }
        count
    }

    pub fn purge_shader_cache(report: &CleanReport) -> usize {
        let mut count = 0;
        for dir in &report.shader_cache_dirs {
            if fs::remove_dir_all(dir).is_ok() {
                count += 1;
                let _ = fs::create_dir_all(dir);
            }
        }
        count
    }

    pub fn purge_all(report: &CleanReport) -> (usize, usize, usize) {
        let d = Self::purge_dumps(report);
        let l = Self::purge_logs(report);
        let s = Self::purge_shader_cache(report);
        (d, l, s)
    }
}
