use crate::error::LogiscoreError;
use crate::protocol::Header;
use crate::protocol::scales::SCALES;

/// 拡張子の情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionInfo {
    pub scale_id: u8,
    pub root_key: u8,
    pub name: &'static str,
}

/// 拡張子からヘッダー情報（Scale ID + Root Key）を決定する。
///
/// 仕様書 §2.2 のマッピングに基づく。
pub fn dispatch(extension: &str) -> ExtensionInfo {
    let ext = extension.trim_start_matches('.').to_lowercase();
    match ext.as_str() {
        "rs" => ExtensionInfo {
            scale_id: 2,
            root_key: 0,
            name: "Rust",
        },
        "py" => ExtensionInfo {
            scale_id: 1,
            root_key: 7,
            name: "Python",
        },
        "ts" | "tsx" | "js" | "jsx" => ExtensionInfo {
            scale_id: 3,
            root_key: 2,
            name: "TypeScript",
        },
        "go" => ExtensionInfo {
            scale_id: 6,
            root_key: 5,
            name: "Go",
        },
        "c" | "cpp" | "h" | "hpp" => ExtensionInfo {
            scale_id: 5,
            root_key: 0,
            name: "C/C++",
        },
        "rb" => ExtensionInfo {
            scale_id: 6,
            root_key: 7,
            name: "Ruby",
        },
        "css" | "scss" | "sass" => ExtensionInfo {
            scale_id: 7,
            root_key: 9,
            name: "CSS",
        },
        "md" | "markdown" => ExtensionInfo {
            scale_id: 0,
            root_key: 4,
            name: "Markdown",
        },
        "json" | "toml" => ExtensionInfo {
            scale_id: 4,
            root_key: 9,
            name: "JSON/TOML",
        },
        "yaml" | "yml" => ExtensionInfo {
            scale_id: 4,
            root_key: 4,
            name: "YAML",
        },
        _ => ExtensionInfo {
            scale_id: 0,
            root_key: 0,
            name: "Default",
        },
    }
}

/// Scale ID と Root Key から代表的な拡張子を特定する。
pub fn extension_for_header(scale_id: u8, root_key: u8) -> String {
    match (scale_id, root_key) {
        (2, 0) => ".rs".to_string(),
        (1, 7) => ".py".to_string(),
        (3, 2) => ".ts".to_string(),
        (6, 5) => ".go".to_string(),
        (5, 0) => ".cpp".to_string(),
        (6, 7) => ".rb".to_string(),
        (7, 9) => ".css".to_string(),
        (0, 4) => ".md".to_string(),
        (4, 9) => ".json".to_string(), // .json と .toml は scale 4, root 9 を共有
        (4, 4) => ".yaml".to_string(),
        _ => ".txt".to_string(),
    }
}

/// 拡張子から Header を生成する。
pub fn header_for_extension(extension: &str) -> Result<Header, LogiscoreError> {
    let info = dispatch(extension);
    Header::new(info.scale_id, info.root_key, 8)
}

/// 拡張子から対応する音階配列を取得する。
pub fn scale_for_extension(extension: &str) -> &'static [u8; 16] {
    let info = dispatch(extension);
    &SCALES[info.scale_id as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_maps_to_industrial() {
        let info = dispatch(".rs");
        assert_eq!(info.scale_id, 2);
        assert_eq!(info.root_key, 0);
    }

    #[test]
    fn python_maps_to_minor() {
        let info = dispatch("py");
        assert_eq!(info.scale_id, 1);
        assert_eq!(info.root_key, 7);
    }

    #[test]
    fn typescript_with_dot() {
        let info = dispatch(".ts");
        assert_eq!(info.scale_id, 3);
    }

    #[test]
    fn tsx_same_as_ts() {
        let ts = dispatch(".ts");
        let tsx = dispatch(".tsx");
        assert_eq!(ts.scale_id, tsx.scale_id);
        assert_eq!(ts.root_key, tsx.root_key);
    }

    #[test]
    fn unknown_defaults_to_major_c() {
        let info = dispatch(".html");
        assert_eq!(info.scale_id, 0);
        assert_eq!(info.root_key, 0);
    }

    #[test]
    fn header_for_all_extensions() {
        for ext in &[".rs", ".py", ".ts", ".go", ".json", ".yaml", ".yml", ".tsx", ".html"] {
            assert!(header_for_extension(ext).is_ok(), "Failed for {}", ext);
        }
    }

    #[test]
    fn yaml_and_yml_same() {
        let yaml = dispatch(".yaml");
        let yml = dispatch(".yml");
        assert_eq!(yaml, yml);
    }
}
