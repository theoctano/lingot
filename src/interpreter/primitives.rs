use std::fs;
use std::path::Path;
use std::process::Command;

use crate::interpreter::value::*;

pub fn io_print(val: &LingotValue) {
    println!("{}", val);
}

pub fn fs_read(path: &str) -> Result<LingotValue, String> {
    fs::read_to_string(path)
        .map(LingotValue::Text)
        .map_err(|e| format!("cannot read '{}': {}", path, e))
}

pub fn fs_write(path: &str, content: &str) -> Result<LingotValue, String> {
    // Create parent directories if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directory for '{}': {}", path, e))?;
    }
    fs::write(path, content)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot write '{}': {}", path, e))
}

pub fn fs_rename(src: &str, dest: &str) -> Result<LingotValue, String> {
    // Create parent directories if needed
    if let Some(parent) = Path::new(dest).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directory for '{}': {}", dest, e))?;
    }
    fs::rename(src, dest)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot move '{}' to '{}': {}", src, dest, e))
}

pub fn fs_delete(path: &str) -> Result<LingotValue, String> {
    let p = Path::new(path);
    if p.is_dir() {
        fs::remove_dir_all(path)
            .map(|_| LingotValue::Void)
            .map_err(|e| format!("cannot delete '{}': {}", path, e))
    } else {
        fs::remove_file(path)
            .map(|_| LingotValue::Void)
            .map_err(|e| format!("cannot delete '{}': {}", path, e))
    }
}

pub fn fs_list(path: &str) -> Result<LingotValue, String> {
    let entries = fs::read_dir(path)
        .map_err(|e| format!("cannot list '{}': {}", path, e))?;

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("error reading entry: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();
        items.push(LingotValue::Text(name));
    }

    Ok(LingotValue::List(items))
}

pub fn process_exec(cmd: &str) -> Result<LingotValue, String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                Ok(LingotValue::Text(stdout))
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                Err(format!("command failed: {}", if stderr.is_empty() {
                    format!("exit code {}", out.status.code().unwrap_or(-1))
                } else {
                    stderr
                }))
            }
        }
        Err(e) => Err(format!("cannot execute '{}': {}", cmd, e)),
    }
}

pub fn fs_copy(src: &str, dest: &str) -> Result<LingotValue, String> {
    if let Some(parent) = Path::new(dest).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directory for '{}': {}", dest, e))?;
    }
    fs::copy(src, dest)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot copy '{}' to '{}': {}", src, dest, e))
}

pub fn text_prefix(path: &str, prefix: &str) -> Result<LingotValue, String> {
    let p = Path::new(path);
    let filename = p.file_name()
        .ok_or_else(|| format!("invalid path: {}", path))?
        .to_string_lossy();
    let new_name = format!("{}{}", prefix, filename);
    let new_path = p.with_file_name(&new_name);
    fs::rename(path, &new_path)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot prefix '{}': {}", path, e))
}

pub fn text_suffix(path: &str, suffix: &str) -> Result<LingotValue, String> {
    let p = Path::new(path);
    let stem = p.file_stem()
        .ok_or_else(|| format!("invalid path: {}", path))?
        .to_string_lossy();
    let ext = p.extension().map(|e| e.to_string_lossy().to_string());
    let new_name = match ext {
        Some(ext) => format!("{}{}.{}", stem, suffix, ext),
        None => format!("{}{}", stem, suffix),
    };
    let new_path = p.with_file_name(&new_name);
    fs::rename(path, &new_path)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot suffix '{}': {}", path, e))
}

pub fn fs_rename_file(path: &str, new_name: &str) -> Result<LingotValue, String> {
    let p = Path::new(path);
    let new_path = p.with_file_name(new_name);
    fs::rename(path, &new_path)
        .map(|_| LingotValue::Void)
        .map_err(|e| format!("cannot rename '{}' to '{}': {}", path, new_name, e))
}
