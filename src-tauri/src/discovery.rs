use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};
use toml::Value as TomlValue;
use walkdir::WalkDir;

const EXTRACTOR_VERSION: &str = "agent-inventory-scanner-1.0.0";
const CACHE_VERSION: u32 = 1;
const CACHE_DIR_NAME: &str = "agent-skill-inventory";
const CACHE_FILE_NAME: &str = "inventory-cache.json";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Codex,
    Claude,
    Gemini,
    Antigravity,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Codex => "codex",
            AgentType::Claude => "claude",
            AgentType::Gemini => "gemini",
            AgentType::Antigravity => "antigravity",
        }
    }
}

impl fmt::Display for AgentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AgentType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "codex" => Ok(AgentType::Codex),
            "claude" => Ok(AgentType::Claude),
            "gemini" => Ok(AgentType::Gemini),
            "antigravity" => Ok(AgentType::Antigravity),
            _ => Err(format!("Unknown agent: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Mcp,
    Skill,
    Soul,
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemKind::Mcp => "mcp",
            ItemKind::Skill => "skill",
            ItemKind::Soul => "soul",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeKind {
    Global,
    Personal,
    Project,
    Managed,
    System,
    Session,
    AntigravityConfig,
    Unknown,
}

impl ScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScopeKind::Global => "global",
            ScopeKind::Personal => "personal",
            ScopeKind::Project => "project",
            ScopeKind::Managed => "managed",
            ScopeKind::System => "system",
            ScopeKind::Session => "session",
            ScopeKind::AntigravityConfig => "antigravity-config",
            ScopeKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatusKind {
    Enabled,
    Disabled,
    Unknown,
}

impl StatusKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusKind::Enabled => "enabled",
            StatusKind::Disabled => "disabled",
            StatusKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySource {
    pub kind: String,
    pub file_path: String,
    pub extractor_version: String,
    pub last_modified: i64,
    pub file_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceSignature {
    pub last_modified: i64,
    pub file_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryCache {
    pub version: u32,
    pub generated_at: i64,
    pub records: Vec<DiscoveryRecord>,
    pub file_signatures: BTreeMap<String, SourceSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryDetails {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub env: Option<BTreeMap<String, String>>,
    pub file: Option<String>,
    pub transport: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRecord {
    pub id: String,
    pub agent: AgentType,
    pub kind: ItemKind,
    pub name: String,
    pub location: String,
    pub scope: ScopeKind,
    pub scope_hint: String,
    pub status: StatusKind,
    pub source: DiscoverySource,
    pub details: DiscoveryDetails,
    pub raw: String,
    pub fingerprint: String,
    pub canonical_group_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupedRecords {
    pub key: String,
    pub items: Vec<DiscoveryRecord>,
}

fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn file_signature(path: &Path) -> Option<SourceSignature> {
    let last_modified = match fs::metadata(path).and_then(|m| m.modified()) {
        Ok(modified) => modified
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        Err(_) => 0,
    };

    Some(SourceSignature {
        last_modified,
        file_hash: file_hash(path),
    })
}

fn file_hash(path: &Path) -> Option<String> {
    let mut buffer = Vec::new();
    let mut handle = fs::File::open(path).ok()?;
    if handle.read_to_end(&mut buffer).is_ok() {
        return Some(hash_bytes(&buffer));
    }
    None
}

fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn hash_text(input: &str) -> String {
    hash_bytes(input.as_bytes())
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn normalize_path(path: &str) -> String {
    path.trim().to_ascii_lowercase()
}

fn build_record_id(agent: &AgentType, kind: &ItemKind, name: &str, location: &str) -> String {
    format!(
        "{}-{}-{}-{}",
        agent.as_str(),
        kind.as_str(),
        normalize_name(name),
        hash_text(&normalize_path(location)).chars().take(10).collect::<String>()
    )
}

fn string_or_none(v: Option<&str>) -> Option<String> {
    match v {
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        _ => None,
    }
}

fn parse_toml_env(table: Option<&TomlValue>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(env_table) = table.and_then(TomlValue::as_table) {
        for (k, v) in env_table {
            match v {
                TomlValue::String(s) => {
                    map.insert(k.clone(), s.clone());
                }
                other => {
                    map.insert(k.clone(), other.to_string());
                }
            }
        }
    }
    map
}

fn parse_json_env(value: Option<&JsonValue>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(env_obj) = value.and_then(JsonValue::as_object) {
        for (k, v) in env_obj {
            map.insert(
                k.clone(),
                match v.as_str() {
                    Some(v) => v.to_string(),
                    None => v.to_string(),
                },
            );
        }
    }
    map
}

fn push_mcp_record(
    agent: AgentType,
    kind: ItemKind,
    name: &str,
    scope: ScopeKind,
    scope_hint: &str,
    source_file: &Path,
    command: Option<String>,
    args: Option<Vec<String>>,
    url: Option<String>,
    env: BTreeMap<String, String>,
    transport: Option<String>,
    disabled: bool,
    raw: &str,
    out: &mut Vec<DiscoveryRecord>,
) {
    if name.trim().is_empty() {
        return;
    }

    let status = if disabled {
        StatusKind::Disabled
    } else {
        StatusKind::Enabled
    };
    let source_signature = file_signature(source_file);

    let location = source_file.to_string_lossy().to_string();
    let details = DiscoveryDetails {
        command,
        args,
        url,
        env: if env.is_empty() { None } else { Some(env) },
        file: Some(location.clone()),
        transport,
        description: None,
    };

    let fingerprint_base = format!(
        "{}|{}|{}|{}",
        name,
        location,
        details.command.clone().unwrap_or_default(),
        details.url.clone().unwrap_or_default()
    );
    let fingerprint = hash_text(&fingerprint_base);
    let canonical_group_id = hash_text(&format!("{}|{}", normalize_name(name), normalize_path(&location)));

    out.push(DiscoveryRecord {
        id: build_record_id(&agent, &kind, name, &location),
        agent,
        kind,
        name: name.trim().to_string(),
        location,
        scope,
        scope_hint: scope_hint.to_string(),
        status,
        source: DiscoverySource {
                kind: "config".to_string(),
                file_path: source_file.to_string_lossy().to_string(),
                extractor_version: EXTRACTOR_VERSION.to_string(),
                last_modified: source_signature.as_ref().map_or(0, |sig| sig.last_modified),
                file_hash: source_signature.and_then(|sig| sig.file_hash),
            },
        details,
        raw: raw.to_string(),
        fingerprint,
        canonical_group_id,
    });
}

fn parse_frontmatter(content: &str) -> (Option<String>, Option<String>) {
    let mut name: Option<String> = None;
    let mut description: Option<String> = None;

    let mut blocks = content.splitn(3, "---");
    let _ = blocks.next();
    let frontmatter = match blocks.next() {
        Some(v) if !v.contains("\n---") => v,
        _ => return (None, None),
    };

    let mut in_description_block = false;
    let mut description_lines: Vec<String> = Vec::new();

    for line in frontmatter.lines() {
        if in_description_block {
            if line.starts_with(' ') {
                description_lines.push(line.trim().to_string());
                continue;
            }
            in_description_block = false;
        }

        if line.starts_with("name:") {
            name = string_or_none(Some(line.trim_start_matches("name:").trim()))
                .or_else(|| string_or_none(Some(line.trim_start_matches("name: ").trim())));
        }
        if line.starts_with("description:") {
            let value = line
                .trim_start_matches("description:")
                .trim_start_matches('>')
                .trim();
            if value.is_empty() {
                in_description_block = true;
            } else {
                description = Some(value.to_string());
            }
        }
    }

    if !description_lines.is_empty() {
        let merged = description_lines
            .into_iter()
            .map(|line| line.trim().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        if !merged.trim().is_empty() {
            description = Some(merged);
        }
    }

    (name, description)
}

fn add_skill_records(
    agent: AgentType,
    directory: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    out: &mut Vec<DiscoveryRecord>,
) {
    if !directory.exists() {
        return;
    }

    for entry in WalkDir::new(directory).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "SKILL.md" {
            continue;
        }

        let path = entry.path().to_path_buf();
        let content = match read_text_file(&path) {
            Some(v) => v,
            None => continue,
        };
        let (name_hint, description) = parse_frontmatter(&content);
        let name = name_hint.unwrap_or_else(|| {
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("skill")
                .to_string()
        });

        let location = path.to_string_lossy().to_string();
        let details = DiscoveryDetails {
            command: None,
            args: None,
            url: None,
            env: None,
            file: Some(location.clone()),
            transport: None,
            description: description.clone(),
        };

        let fingerprint = hash_text(&format!("{}|{}|{}", name, location, description.clone().unwrap_or_default()));
        let canonical_group_id = hash_text(&format!("{}|{}", normalize_name(&name), normalize_path(&location)));

        let signature = file_signature(&path);
        out.push(DiscoveryRecord {
            id: build_record_id(&agent, &ItemKind::Skill, &name, &location),
            agent,
            kind: ItemKind::Skill,
            name,
            location,
            scope,
            scope_hint: scope_hint.to_string(),
            status: StatusKind::Unknown,
            source: DiscoverySource {
                kind: "skill".to_string(),
                file_path: path.to_string_lossy().to_string(),
                extractor_version: EXTRACTOR_VERSION.to_string(),
                last_modified: signature.as_ref().map_or(0, |sig| sig.last_modified),
                file_hash: signature.and_then(|sig| sig.file_hash),
            },
            details,
            raw: content,
            fingerprint,
            canonical_group_id,
        });
    }
}

fn add_soul_file(
    agent: AgentType,
    path: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    out: &mut Vec<DiscoveryRecord>,
) {
    if !path.exists() {
        return;
    }

    let content = read_text_file(path).unwrap_or_default();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("soul")
        .to_string();

    let location = path.to_string_lossy().to_string();
    let details = DiscoveryDetails {
        command: None,
        args: None,
        url: None,
        env: None,
        file: Some(location.clone()),
        transport: None,
        description: None,
    };

    let fingerprint = hash_text(&format!("{}|{}", name, location));
    let canonical_group_id = hash_text(&format!("{}|{}", normalize_name(&name), normalize_path(&location)));

    let signature = file_signature(path);
    out.push(DiscoveryRecord {
        id: build_record_id(&agent, &ItemKind::Soul, &name, &location),
        agent,
        kind: ItemKind::Soul,
        name,
        location,
        scope,
        scope_hint: scope_hint.to_string(),
        status: StatusKind::Unknown,
        source: DiscoverySource {
            kind: "soul".to_string(),
            file_path: path.to_string_lossy().to_string(),
            extractor_version: EXTRACTOR_VERSION.to_string(),
            last_modified: signature.as_ref().map_or(0, |sig| sig.last_modified),
            file_hash: signature.and_then(|sig| sig.file_hash),
        },
        details,
        raw: content,
        fingerprint,
        canonical_group_id,
    });
}

fn parse_toml_mcp_config(
    agent: AgentType,
    path: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    out: &mut Vec<DiscoveryRecord>,
) {
    let content = match read_text_file(path) {
        Some(v) => v,
        None => return,
    };

    let value: TomlValue = match content.parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    let Some(servers) = value.get("mcp_servers").and_then(TomlValue::as_table) else {
        return;
    };

    for (name, cfg) in servers {
        let table = match cfg.as_table() {
            Some(v) => v,
            None => continue,
        };

        let command = table
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let args = table.get("args").and_then(TomlValue::as_array).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        });
        let transport = table
            .get("type")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .or_else(|| table.get("transport").and_then(|v| v.as_str()).map(|v| v.to_string()));
        let url = table
            .get("url")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let env = parse_toml_env(table.get("env"));
        let disabled = table.get("disabled").and_then(TomlValue::as_bool).unwrap_or(false);

        push_mcp_record(
            agent,
            ItemKind::Mcp,
            name,
            scope,
            scope_hint,
            path,
            command,
            args,
            url,
            env,
            transport,
            disabled,
            &content,
            out,
        );
    }
}

fn parse_json_mcp_map(
    agent: AgentType,
    path: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    source_node: &JsonValue,
    out: &mut Vec<DiscoveryRecord>,
) {
    let Some(server_map) = source_node.as_object() else {
        return;
    };

    for (name, cfg) in server_map {
        let cfg_obj = match cfg.as_object() {
            Some(v) => v,
            None => continue,
        };

        let command = cfg_obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let args = cfg_obj
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            });
        let url = cfg_obj
            .get("url")
            .or_else(|| cfg_obj.get("endpoint"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let transport = cfg_obj
            .get("type")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let env = parse_json_env(cfg_obj.get("env"));
        let disabled = cfg_obj
            .get("disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let raw = cfg.to_string();

        push_mcp_record(
            agent,
            ItemKind::Mcp,
            name,
            scope,
            scope_hint,
            path,
            command,
            args,
            url,
            env,
            transport,
            disabled,
            &raw,
            out,
        );
    }
}

fn parse_json_object_for_mcp(
    agent: AgentType,
    path: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    root: &JsonValue,
    out: &mut Vec<DiscoveryRecord>,
) {
    if let Some(mcp_node) = root.get("mcpServers").or_else(|| root.get("mcp_servers")) {
        parse_json_mcp_map(agent, path, scope, scope_hint, mcp_node, out);
    }

    if let Some(servers) = root.get("servers") {
        parse_json_mcp_map(agent, path, scope, scope_hint, servers, out);
    }
}

fn parse_antigravity_json_file(
    agent: AgentType,
    path: &Path,
    scope: ScopeKind,
    scope_hint: &str,
    out: &mut Vec<DiscoveryRecord>,
) {
    let raw = match fs::read(path) {
        Ok(v) => match String::from_utf8(v) {
            Ok(text) => text,
            Err(err) => String::from_utf8_lossy(&err.into_bytes()).to_string(),
        },
        Err(_) => return,
    };

    let parsed = match serde_json::from_str::<JsonValue>(&raw) {
        Ok(v) => v,
        Err(_) => {
            let start = match raw.find('{') {
                Some(v) => v,
                None => return,
            };
            let end = match raw.rfind('}') {
                Some(v) => v,
                None => return,
            };
            if start <= end {
                let sliced = &raw[start..=end];
                serde_json::from_str::<JsonValue>(sliced).unwrap_or(JsonValue::Null)
            } else {
                JsonValue::Null
            }
        }
    };

    if parsed.is_null() {
        return;
    }

    parse_json_object_for_mcp(agent, path, scope, scope_hint, &parsed, out);
}

fn scan_codex(home: &Path, roots: &[PathBuf], out: &mut Vec<DiscoveryRecord>) {
    let user_scope = ScopeKind::Personal;
    let base_config = home.join(".codex/config.toml");
    if base_config.exists() {
        parse_toml_mcp_config(AgentType::Codex, &base_config, user_scope, "~/.codex", out);
    }

    let skills_home = home.join(".codex/skills");
    add_skill_records(AgentType::Codex, &skills_home, user_scope, "~/.codex", out);

    let vendor_imports = home.join(".codex/vendor_imports/skills");
    add_skill_records(
        AgentType::Codex,
        &vendor_imports,
        ScopeKind::Global,
        "~/.codex/vendor_imports",
        out,
    );

    // Soul files — personal
    add_soul_file(AgentType::Codex, &home.join(".codex/AGENTS.md"), ScopeKind::Personal, "~/.codex", out);
    let rules_dir = home.join(".codex/rules");
    if rules_dir.exists() {
        for entry in WalkDir::new(&rules_dir).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rules") {
                add_soul_file(AgentType::Codex, entry.path(), ScopeKind::Personal, "~/.codex/rules", out);
            }
        }
    }

    for root in roots {
        let project_hint = root.to_string_lossy().to_string();
        let cfg = root.join(".codex/config.toml");
        if cfg.exists() {
            parse_toml_mcp_config(AgentType::Codex, &cfg, ScopeKind::Project, &project_hint, out);
        }
        add_skill_records(
            AgentType::Codex,
            &root.join(".codex/skills"),
            ScopeKind::Project,
            &project_hint,
            out,
        );
        // Soul files — project
        add_soul_file(AgentType::Codex, &root.join("CODEX.md"), ScopeKind::Project, &project_hint, out);
        add_soul_file(AgentType::Codex, &root.join("AGENTS.md"), ScopeKind::Project, &project_hint, out);
    }
}

fn scan_claude(home: &Path, roots: &[PathBuf], out: &mut Vec<DiscoveryRecord>) {
    let home_cfg = home.join(".claude.json");
    if home_cfg.exists() {
        let raw = match read_text_file(&home_cfg) {
            Some(v) => v,
            None => String::new(),
        };
        if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
            if let Some(global_servers) = json.get("mcpServers") {
                parse_json_mcp_map(
                    AgentType::Claude,
                    &home_cfg,
                    ScopeKind::Personal,
                    "~/.claude.json:mcpServers",
                    global_servers,
                    out,
                );
            }

            let project_obj = json.get("projects")
                .and_then(JsonValue::as_object)
                .or_else(|| json.as_object());
            if let Some(projects) = project_obj {
                for (key, value) in projects {
                    if !key.starts_with('/') {
                        continue;
                    }
                    if let Some(servers) = value.get("mcpServers") {
                        parse_json_mcp_map(AgentType::Claude, &home_cfg, ScopeKind::Project, key, servers, out);
                    }
                }
            }
        }
    }

    let settings_file = home.join(".claude/settings.json");
    if settings_file.exists() {
        let raw = match read_text_file(&settings_file) {
            Some(v) => v,
            None => String::new(),
        };
        if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
            parse_json_object_for_mcp(
                AgentType::Claude,
                &settings_file,
                ScopeKind::Personal,
                "~/.claude/settings.json",
                &json,
                out,
            );
        }
    }

    let projects_dir = home.join(".claude/projects");
    if projects_dir.exists() {
        for entry in WalkDir::new(&projects_dir).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let raw = match read_text_file(entry.path()) {
                Some(v) => v,
                None => continue,
            };
            if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
                let hint = format!("~/.claude/projects/{}", entry.path().display());
                parse_json_object_for_mcp(AgentType::Claude, entry.path(), ScopeKind::Project, &hint, &json, out);
            }
        }
    }

    let managed = Path::new("/Library/Application Support/ClaudeCode/managed-mcp.json");
    if managed.exists() {
        let raw = match read_text_file(managed) {
            Some(v) => v,
            None => String::new(),
        };
        if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
            parse_json_object_for_mcp(
                AgentType::Claude,
                managed,
                ScopeKind::Managed,
                "/Library/Application Support/ClaudeCode",
                &json,
                out,
            );
        }
    }

    let skill_dir = home.join(".claude/skills");
    add_skill_records(AgentType::Claude, &skill_dir, ScopeKind::Personal, "~/.claude", out);

    // Soul files — personal
    add_soul_file(AgentType::Claude, &home.join(".claude/CLAUDE.md"), ScopeKind::Personal, "~/.claude", out);

    for root in roots {
        let hint = root.to_string_lossy().to_string();
        add_skill_records(
            AgentType::Claude,
            &root.join(".claude/skills"),
            ScopeKind::Project,
            &hint,
            out,
        );

        let project_mcp = root.join(".mcp.json");
        if project_mcp.exists() {
            let raw = match read_text_file(&project_mcp) {
                Some(v) => v,
                None => String::new(),
            };
            if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
                parse_json_object_for_mcp(AgentType::Claude, &project_mcp, ScopeKind::Project, &hint, &json, out);
            }
        }

        // Soul files — project
        add_soul_file(AgentType::Claude, &root.join("CLAUDE.md"), ScopeKind::Project, &hint, out);
        add_soul_file(AgentType::Claude, &root.join("AGENTS.md"), ScopeKind::Project, &hint, out);
    }
}

fn scan_gemini(home: &Path, roots: &[PathBuf], out: &mut Vec<DiscoveryRecord>) {
    let global = home.join(".gemini/settings.json");
    if global.exists() {
        let raw = match read_text_file(&global) {
            Some(v) => v,
            None => String::new(),
        };
        if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
            parse_json_object_for_mcp(AgentType::Gemini, &global, ScopeKind::Personal, "~/.gemini", &json, out);
        }
    }

    add_skill_records(
        AgentType::Gemini,
        &home.join(".gemini/skills"),
        ScopeKind::Personal,
        "~/.gemini",
        out,
    );

    // Soul files — personal
    add_soul_file(AgentType::Gemini, &home.join(".gemini/GEMINI.md"), ScopeKind::Personal, "~/.gemini", out);

    let antigravity_global = home.join(".gemini/antigravity/mcp_config.json");
    if antigravity_global.exists() {
        parse_antigravity_json_file(
            AgentType::Gemini,
            &antigravity_global,
            ScopeKind::AntigravityConfig,
            "~/.gemini/antigravity",
            out,
        );
    }

    for root in roots {
        let project_hint = root.to_string_lossy().to_string();
        let setting = root.join(".gemini/settings.json");
        if setting.exists() {
            let raw = read_text_file(&setting).unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<JsonValue>(&raw) {
                parse_json_object_for_mcp(AgentType::Gemini, &setting, ScopeKind::Project, &project_hint, &json, out);
            }
        }
        add_skill_records(AgentType::Gemini, &root.join(".gemini/skills"), ScopeKind::Project, &project_hint, out);
        // Soul files — project
        add_soul_file(AgentType::Gemini, &root.join("GEMINI.md"), ScopeKind::Project, &project_hint, out);
        add_soul_file(AgentType::Gemini, &root.join("AGENTS.md"), ScopeKind::Project, &project_hint, out);
    }
}

fn scan_antigravity(home: &Path, roots: &[PathBuf], out: &mut Vec<DiscoveryRecord>) {
    let base = home.join(".gemini/antigravity");
    if !base.exists() {
        return;
    }

    let mcp_global = base.join("mcp_config.json");
    if mcp_global.exists() {
        parse_antigravity_json_file(
            AgentType::Antigravity,
            &mcp_global,
            ScopeKind::AntigravityConfig,
            "~/.gemini/antigravity",
            out,
        );
    }

    let code_tracker = base.join("code_tracker");
    if code_tracker.exists() {
        for entry in WalkDir::new(&code_tracker).into_iter().flatten() {
            if entry.file_type().is_file() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                if !path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("")
                    .ends_with("_mcp.json")
                {
                    continue;
                }

                let hint = path
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("antigravity");
                parse_antigravity_json_file(
                    AgentType::Antigravity,
                    path,
                    ScopeKind::Session,
                    hint,
                    out,
                );
            }
        }
    }

    let skills = base.join("skills");
    add_skill_records(
        AgentType::Antigravity,
        &skills,
        ScopeKind::Global,
        "~/.gemini/antigravity",
        out,
    );

    for root in roots {
        let project_antigravity = root.join(".antigravity");
        if project_antigravity.exists() {
            add_skill_records(
                AgentType::Antigravity,
                &project_antigravity.join("skills"),
                ScopeKind::Project,
                &root.to_string_lossy(),
                out,
            );
        }
    }
}

fn collect_skill_files(skills_dir: &Path, paths: &mut HashSet<PathBuf>) {
    if !skills_dir.exists() {
        return;
    }
    for entry in WalkDir::new(skills_dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() == "SKILL.md" {
            paths.insert(entry.path().to_path_buf());
        }
    }
}

fn collect_antigravity_code_tracker_files(code_tracker: &Path, paths: &mut HashSet<PathBuf>) {
    if !code_tracker.exists() {
        return;
    }
    for entry in WalkDir::new(code_tracker).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        if path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .ends_with("_mcp.json")
        {
            paths.insert(path.to_path_buf());
        }
    }
}

fn collect_rules_files(rules_dir: &Path, paths: &mut HashSet<PathBuf>) {
    if !rules_dir.exists() {
        return;
    }
    for entry in WalkDir::new(rules_dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rules") {
            paths.insert(entry.path().to_path_buf());
        }
    }
}

fn collect_json_files(dir: &Path, paths: &mut HashSet<PathBuf>) {
    if !dir.exists() {
        return;
    }
    for entry in WalkDir::new(dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        paths.insert(entry.path().to_path_buf());
    }
}

fn collect_candidate_files(home: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths: HashSet<PathBuf> = HashSet::new();

    let add = |paths: &mut HashSet<PathBuf>, p: &Path| {
        if p.exists() {
            paths.insert(p.to_path_buf());
        }
    };

    let base_codex = home.join(".codex/config.toml");
    add(&mut paths, &base_codex);
    collect_skill_files(&home.join(".codex/skills"), &mut paths);
    collect_skill_files(&home.join(".codex/vendor_imports/skills"), &mut paths);
    // Codex soul files — personal
    add(&mut paths, &home.join(".codex/AGENTS.md"));
    collect_rules_files(&home.join(".codex/rules"), &mut paths);

    let claude_json = home.join(".claude.json");
    add(&mut paths, &claude_json);
    add(&mut paths, &home.join(".claude/settings.json"));
    add(&mut paths, &home.join(".claude/skills"));
    collect_skill_files(&home.join(".claude/skills"), &mut paths);
    collect_json_files(&home.join(".claude/projects"), &mut paths);
    add(&mut paths, &Path::new("/Library/Application Support/ClaudeCode/managed-mcp.json"));
    // Claude soul files — personal
    add(&mut paths, &home.join(".claude/CLAUDE.md"));

    let antigravity_dir = home.join(".gemini/antigravity");
    add(&mut paths, &antigravity_dir.join("mcp_config.json"));
    collect_antigravity_code_tracker_files(&antigravity_dir.join("code_tracker"), &mut paths);
    collect_skill_files(&antigravity_dir.join("skills"), &mut paths);

    add(&mut paths, &home.join(".gemini/settings.json"));
    collect_skill_files(&home.join(".gemini/skills"), &mut paths);
    // Gemini soul files — personal
    add(&mut paths, &home.join(".gemini/GEMINI.md"));

    for root in roots {
        add(&mut paths, &root.join(".codex/config.toml"));
        collect_skill_files(&root.join(".codex/skills"), &mut paths);

        add(&mut paths, &root.join(".mcp.json"));

        collect_skill_files(&root.join(".claude/skills"), &mut paths);
        add(&mut paths, &root.join(".gemini/settings.json"));
        collect_skill_files(&root.join(".gemini/skills"), &mut paths);
        collect_skill_files(&root.join(".antigravity/skills"), &mut paths);

        // Soul files — project
        add(&mut paths, &root.join("CLAUDE.md"));
        add(&mut paths, &root.join("CODEX.md"));
        add(&mut paths, &root.join("GEMINI.md"));
        add(&mut paths, &root.join("AGENTS.md"));
    }

    paths.into_iter().collect()
}

fn collect_candidate_project_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots: HashSet<PathBuf> = HashSet::new();

    if let Ok(current) = env::current_dir() {
        roots.insert(current);
    }

    // 1) ~/.claude.json — projects that have been opened in Claude Code
    let claude_json = home.join(".claude.json");
    if let Some(text) = read_text_file(&claude_json) {
        if let Ok(json) = serde_json::from_str::<JsonValue>(&text) {
            let project_map = json.get("projects")
                .and_then(JsonValue::as_object)
                .or_else(|| json.as_object());
            if let Some(obj) = project_map {
                for (path_text, value) in obj {
                    if !path_text.starts_with('/') {
                        continue;
                    }
                    let p = PathBuf::from(path_text);
                    if p.exists() && p.is_dir() {
                        roots.insert(p.clone());
                    }
                    if value.is_object() {
                        let gemini_path = p.join(".gemini");
                        let codex_path = p.join(".codex");
                        let claude_path = p.join(".claude");
                        for extra in [gemini_path, codex_path, claude_path] {
                            if extra.exists() {
                                roots.insert(p.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // 2) Common workspace directories — 1 level deep, only add dirs with agent configs
    let workspace_hints = [
        "Documents/GitHub",
        "Documents/github",
        "Developer",
        "Projects",
        "projects",
        "workspace",
        "code",
        "dev",
        "src",
    ];
    for hint in &workspace_hints {
        let dir = home.join(hint);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let has_agent_config = path.join(".codex").is_dir()
                    || path.join(".gemini").is_dir()
                    || path.join(".claude").is_dir()
                    || path.join(".mcp.json").is_file();
                if has_agent_config {
                    roots.insert(path);
                }
            }
        }
    }

    roots.into_iter().collect()
}

fn dedupe_records(records: &mut Vec<DiscoveryRecord>) {
    let mut seen: HashSet<String> = HashSet::new();
    records.retain(|record| {
        let key = format!(
            "{}|{}|{}|{}|{}",
            record.agent.as_str(),
            record.kind.as_str(),
            normalize_name(&record.name),
            normalize_path(&record.location),
            record.scope.as_str(),
        );
        if seen.contains(&key) {
            false
        } else {
            seen.insert(key);
            true
        }
    });
}

fn sort_records(records: &mut Vec<DiscoveryRecord>) {
    let agent_order: HashMap<&str, usize> = HashMap::from([
        ("codex", 0),
        ("claude", 1),
        ("gemini", 2),
        ("antigravity", 3),
    ]);

    records.sort_by(|a, b| {
        let order_a = *agent_order.get(a.agent.as_str()).unwrap_or(&99);
        let order_b = *agent_order.get(b.agent.as_str()).unwrap_or(&99);

        if order_a != order_b {
            return order_a.cmp(&order_b);
        }

        let scope_cmp = a.scope.as_str().cmp(b.scope.as_str());
        if scope_cmp != std::cmp::Ordering::Equal {
            return scope_cmp;
        }

        let loc_cmp = a.scope_hint.cmp(&b.scope_hint);
        if loc_cmp != std::cmp::Ordering::Equal {
            return loc_cmp;
        }

        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });
}

fn filter_records(records: &[DiscoveryRecord], filter_agent: Option<AgentType>) -> Vec<DiscoveryRecord> {
    match filter_agent {
        Some(agent) => records
            .iter()
            .filter(|record| record.agent == agent)
            .cloned()
            .collect(),
        None => records.to_vec(),
    }
}

fn cache_file_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|cache| cache.join(CACHE_DIR_NAME).join(CACHE_FILE_NAME))
}

fn load_cache() -> Option<DiscoveryCache> {
    let path = cache_file_path()?;
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<DiscoveryCache>(&raw).ok()
}

fn save_cache(cache: &DiscoveryCache) {
    if let Some(path) = cache_file_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec(cache) {
            let _ = fs::write(path, bytes);
        }
    }
}

fn cache_matches(cache: &DiscoveryCache, signatures: &BTreeMap<String, SourceSignature>) -> bool {
    if cache.version != CACHE_VERSION {
        return false;
    }

    if cache.file_signatures.len() != signatures.len() {
        return false;
    }

    for (path, signature) in signatures {
        match cache.file_signatures.get(path) {
            Some(existing) if existing == signature => {}
            _ => return false,
        }
    }

    true
}

fn collect_signature_map(candidate_paths: &[PathBuf]) -> BTreeMap<String, SourceSignature> {
    let mut signatures = BTreeMap::new();
    for path in candidate_paths {
        if let Some(signature) = file_signature(path) {
            signatures.insert(path.to_string_lossy().to_string(), signature);
        }
    }
    signatures
}

pub fn discover_all(filter_agent: Option<AgentType>, force_refresh: bool) -> Result<Vec<DiscoveryRecord>, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    let roots = collect_candidate_project_roots(&home);
    let candidate_files = collect_candidate_files(&home, &roots);
    let signatures = collect_signature_map(&candidate_files);

    if !force_refresh {
        if let Some(cache) = load_cache() {
            if cache_matches(&cache, &signatures) {
                return Ok(filter_records(&cache.records, filter_agent));
            }
        }
    }

    let mut items = Vec::new();
    match filter_agent {
        None => {
            scan_codex(&home, &roots, &mut items);
            scan_claude(&home, &roots, &mut items);
            scan_gemini(&home, &roots, &mut items);
            scan_antigravity(&home, &roots, &mut items);
        }
        Some(AgentType::Codex) => scan_codex(&home, &roots, &mut items),
        Some(AgentType::Claude) => scan_claude(&home, &roots, &mut items),
        Some(AgentType::Gemini) => scan_gemini(&home, &roots, &mut items),
        Some(AgentType::Antigravity) => scan_antigravity(&home, &roots, &mut items),
    }

    if !items.is_empty() {
        dedupe_records(&mut items);
        sort_records(&mut items);
    }

    if filter_agent.is_none() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        save_cache(&DiscoveryCache {
            version: CACHE_VERSION,
            generated_at: now,
            records: items.clone(),
            file_signatures: signatures,
        });
    }

    Ok(items)
}

pub fn group_records(records: Vec<DiscoveryRecord>, key: String) -> Vec<GroupedRecords> {
    let mut grouped: BTreeMap<String, Vec<DiscoveryRecord>> = BTreeMap::new();

    for record in records {
        let group_key = match key.as_str() {
            "agent" => record.agent.to_string(),
            "scope" => format!("{} ({})", record.scope.as_str(), record.scope_hint),
            "location" => record.scope_hint.clone(),
            _ => record.agent.to_string(),
        };
        grouped.entry(group_key).or_default().push(record);
    }

    let mut result = Vec::new();
    for (k, mut v) in grouped {
        v.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        result.push(GroupedRecords { key: k, items: v });
    }
    result
}

pub fn resolve_duplicate_groups(records: Vec<DiscoveryRecord>) -> Vec<Vec<DiscoveryRecord>> {
    let mut grouped: BTreeMap<String, Vec<DiscoveryRecord>> = BTreeMap::new();

    for record in records {
        grouped
            .entry(record.canonical_group_id.clone())
            .or_default()
            .push(record);
    }

    let mut result = Vec::new();
    for (_, mut list) in grouped {
        if list.len() >= 2 {
            list.sort_by(|a, b| {
                a.agent
                    .as_str()
                    .cmp(b.agent.as_str())
                    .then(a.scope.as_str().cmp(&b.scope.as_str()).then(a.location.cmp(&b.location)))
            });
            result.push(list);
        }
    }
    result
}
