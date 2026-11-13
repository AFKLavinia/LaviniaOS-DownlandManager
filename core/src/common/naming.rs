use std::path::{Path, PathBuf};

/// Replace characters that are invalid in Windows/Unix filenames.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "download.bin".to_string()
    } else {
        trimmed
    }
}

/// Produce a filename that does not collide with an existing file or `.part.meta`
/// in `dir`, appending ` (1)`, ` (2)`, ... before the extension.
/// `taken` lets callers reserve names used by other in-memory jobs.
pub fn unique_filename(dir: &Path, name: &str, taken: &dyn Fn(&str) -> bool) -> String {
    let name = sanitize_filename(name);
    let is_free = |n: &str| {
        !taken(n) && !dir.join(n).exists() && !dir.join(format!("{}.part.meta", n)).exists()
    };
    if is_free(&name) {
        return name;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name.as_str(), ""),
    };
    let mut n = 1u32;
    loop {
        let candidate = format!("{} ({}){}", stem, n, ext);
        if is_free(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Folder category derived from the file extension.
pub fn category_for(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if filename.rfind('.').is_none() {
        return "Other";
    }
    match ext.as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" => "Video",
        "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "wma" => "Music",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => "Pictures",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => "Archives",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "epub" | "md" => {
            "Documents"
        }
        "exe" | "msi" | "apk" | "dmg" | "deb" | "rpm" | "msix" => "Programs",
        _ => "Other",
    }
}

/// Destination directory, optionally nested into a category subfolder.
pub fn categorized_dir(base: &Path, filename: &str, use_categories: bool) -> PathBuf {
    if use_categories {
        base.join(category_for(filename))
    } else {
        base.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_invalid_chars() {
        assert_eq!(sanitize_filename("a:b*c?.txt"), "a_b_c_.txt");
        assert_eq!(sanitize_filename("   "), "download.bin");
    }

    #[test]
    fn unique_name_appends_counter() {
        let dir = std::env::temp_dir().join(format!("lodm_naming_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), b"x").unwrap();
        let n = unique_filename(&dir, "a.txt", &|_| false);
        assert_eq!(n, "a (1).txt");
        let n2 = unique_filename(&dir, "a.txt", &|x| x == "a (1).txt");
        assert_eq!(n2, "a (2).txt");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn categories() {
        assert_eq!(category_for("movie.MKV"), "Video");
        assert_eq!(category_for("setup.exe"), "Programs");
        assert_eq!(category_for("noext"), "Other");
    }
}
