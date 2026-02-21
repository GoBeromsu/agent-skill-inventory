#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod discovery;

use discovery::{group_records, resolve_duplicate_groups, DiscoveryRecord};
use std::process::Command;
use tauri::command;
use discovery::{discover_all as internal_discover_all, AgentType};

fn parse_agent_filter(agent: Option<String>) -> Result<Option<AgentType>, String> {
    agent
        .map(|value| value.parse::<AgentType>())
        .transpose()
        .map_err(|error| format!("invalid agent filter: {error}"))
}

#[command]
fn discover_all() -> Result<Vec<DiscoveryRecord>, String> {
    internal_discover_all(None, false)
}

#[command]
fn refresh(agent: Option<String>) -> Result<Vec<DiscoveryRecord>, String> {
    let filter = parse_agent_filter(agent)?;
    internal_discover_all(filter, true)
}

#[command]
fn group_by(records: Vec<DiscoveryRecord>, key: String) -> Result<Vec<discovery::GroupedRecords>, String> {
    if key != "agent" && key != "scope" && key != "location" {
        return Err(format!("unsupported group_by key: {key}"));
    }
    Ok(group_records(records, key))
}

#[command]
fn resolve_duplicates(records: Vec<DiscoveryRecord>) -> Result<Vec<Vec<DiscoveryRecord>>, String> {
    Ok(resolve_duplicate_groups(records))
}

#[command]
fn open_path(path: String) -> Result<(), String> {
    let status = Command::new("open")
        .arg(path)
        .status()
        .map_err(|e| format!("failed to run open command: {}", e))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("open command returned non-zero exit: {:?}", status.code()))
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            discover_all,
            refresh,
            group_by,
            resolve_duplicates,
            open_path
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
