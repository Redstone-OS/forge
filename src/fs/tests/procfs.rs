//! Testes para ProcFS (Process Filesystem)

#![cfg(test)]

use crate::fs::procfs::{ProcEntry, ProcFS};

#[test]
fn test_proc_entry_creation() {
    let entry = ProcEntry::new("cpuinfo", None);
    assert_eq!(entry.name, "cpuinfo");
    assert_eq!(entry.pid, None);
}

#[test]
fn test_proc_entry_with_pid() {
    let entry = ProcEntry::new("status", Some(1234));
    assert_eq!(entry.name, "status");
    assert_eq!(entry.pid, Some(1234));
}

#[test]
fn test_procfs_creation() {
    let procfs = ProcFS::new();
    let _ = procfs;
}

#[test]
fn test_list_processes_empty() {
    let procfs = ProcFS::new();
    let processes = procfs.list_processes();
    assert_eq!(processes.len(), 0);
}

// TODO: Adicionar testes quando implementar:
// - test_read_cpuinfo
// - test_read_meminfo
// - test_read_process_status
// - test_read_process_cmdline
// - test_list_processes
