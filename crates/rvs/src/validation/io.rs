use std::collections::HashMap;

use uuid::Uuid;

use super::{Report, ValidationJob};
use crate::client::NiccClient;
use crate::error::RvsError;
use crate::partitions::Partitions;
use crate::rack::Tray;

/// Decided run ID and the label writes needed to apply it.
struct RunIdPlan {
    run_id: String,
    /// Per-tray label maps to write. Empty when the run ID was reused as-is.
    updates: Vec<(String, HashMap<String, String>)>,
}

/// Determine the run ID for this set of trays and compute any label updates.
///
/// Reuses the existing `rv.run-id` if all trays share the same value.
/// Otherwise generates a fresh UUID and prepares updated labels for every tray.
fn prepare_run_id(trays: &HashMap<String, Tray>) -> RunIdPlan {
    match existing_run_id(trays) {
        Some(id) => RunIdPlan { run_id: id, updates: vec![] },
        None => {
            let id = Uuid::new_v4().to_string();
            let updates = trays
                .iter()
                .map(|(tray_id, tray)| {
                    let mut labels = tray.rv_labels.clone();
                    labels.insert("rv.run-id".to_string(), id.clone());
                    (tray_id.clone(), labels)
                })
                .collect();
            RunIdPlan { run_id: id, updates }
        }
    }
}

/// Convert filtered partitions into validation jobs.
///
/// Full preparation pipeline per partition:
///   1. assign_run_id   - stamp every tray with a consistent rv.run-id
///   2. allocate_instances - boot a validation OS instance on each tray
///   3. wait_for_boot   - block until every instance is ready
pub async fn plan(
    partitions: Partitions,
    nicc: &NiccClient,
    os_uri: &str,
) -> Result<Vec<ValidationJob>, RvsError> {
    if partitions.all.is_empty() {
        return Ok(vec![]);
    }
    assign_run_id(&partitions.all, nicc).await?;
    allocate_instances(&partitions.all, os_uri, nicc).await?;
    wait_for_boot(&partitions.all, nicc).await?;
    Ok(vec![ValidationJob {
        trays: partitions.all.into_values().collect(),
    }])
}

/// Ensure every tray carries a consistent `rv.run-id`, writing it if absent.
async fn assign_run_id(trays: &HashMap<String, Tray>, nicc: &NiccClient) -> Result<(), RvsError> {
    let plan = prepare_run_id(trays);
    tracing::info!(run_id = %plan.run_id, reused = plan.updates.is_empty(), "validation: run ID resolved");
    for (tray_id, labels) in plan.updates {
        nicc.update_rv_labels(&tray_id, &labels).await?;
    }
    Ok(())
}

/// Allocate a validation OS instance on each tray in the partition.
///
/// TODO[#416]: stub - wire in nicc.allocate_machine_instance per tray and collect
/// instance IDs for boot tracking. ValidationJob will carry them once expanded.
async fn allocate_instances(
    _trays: &HashMap<String, Tray>,
    _os_uri: &str,
    _nicc: &NiccClient,
) -> Result<(), RvsError> {
    let () = std::future::ready(()).await; // phantom await: keeps async sig for future wiring
    Ok(())
}

/// Wait until every allocated instance has booted and reached READY state.
///
/// TODO[#416]: stub - wire in polling loop with exponential backoff and timeout once
/// allocate_instances populates instance IDs on ValidationJob.
async fn wait_for_boot(_trays: &HashMap<String, Tray>, _nicc: &NiccClient) -> Result<(), RvsError> {
    let () = std::future::ready(()).await; // phantom await: keeps async sig for future wiring
    Ok(())
}

/// Run validation against a single job and produce a report.
///
/// Stub: counts trays in the partition as a stand-in for real validation output.
pub async fn validate_partition(job: ValidationJob) -> Result<Report, RvsError> {
    let trays_cnt = job.trays.len() as u32;
    tracing::info!(trays_cnt, "validation: partition validated (stub)");
    Ok(Report { trays_cnt })
}

/// Submit a completed report.
///
/// Stub: prints tray count to console.
pub async fn submit_report(report: Report) -> Result<(), RvsError> {
    tracing::info!(trays_cnt = report.trays_cnt, "validation report");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partitions::{IbNode, NvlNode};

    fn tray(rv_labels: &[(&str, &str)]) -> Tray {
        Tray::new(
            "rack-1".to_string(),
            "Validation(Pending)".to_string(),
            rv_labels.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            NvlNode::new(0),
            IbNode::new(0, 0),
        )
    }

    fn trays(entries: &[(&str, &[(&str, &str)])]) -> HashMap<String, Tray> {
        entries.iter().map(|(id, labels)| (id.to_string(), tray(labels))).collect()
    }

    #[test]
    fn test_prepare_run_id_reuses_existing() {
        let t = trays(&[
            ("m1", &[("rv.run-id", "run-abc")]),
            ("m2", &[("rv.run-id", "run-abc")]),
        ]);
        let plan = prepare_run_id(&t);
        assert_eq!(plan.run_id, "run-abc");
        assert!(plan.updates.is_empty());
    }

    #[test]
    fn test_prepare_run_id_assigns_new_when_missing() {
        let t = trays(&[("m1", &[]), ("m2", &[])]);
        let plan = prepare_run_id(&t);
        assert!(!plan.run_id.is_empty());
        assert_eq!(plan.updates.len(), 2);
        for (_, labels) in &plan.updates {
            assert_eq!(labels["rv.run-id"], plan.run_id);
        }
    }

    #[test]
    fn test_prepare_run_id_assigns_new_when_mixed() {
        // Trays disagree on run-id - treat as missing, assign fresh.
        let t = trays(&[
            ("m1", &[("rv.run-id", "run-abc")]),
            ("m2", &[("rv.run-id", "run-xyz")]),
        ]);
        let plan = prepare_run_id(&t);
        assert_ne!(plan.run_id, "run-abc");
        assert_ne!(plan.run_id, "run-xyz");
        assert_eq!(plan.updates.len(), 2);
    }

    #[test]
    fn test_prepare_run_id_preserves_other_labels() {
        let t = trays(&[("m1", &[("rv.st", "pass")])]);
        let plan = prepare_run_id(&t);
        let (_, labels) = plan.updates.iter().find(|(id, _)| id == "m1").unwrap();
        assert_eq!(labels["rv.st"], "pass");
        assert_eq!(labels["rv.run-id"], plan.run_id);
    }

    #[test]
    fn test_prepare_run_id_empty_trays() {
        let t = trays(&[]);
        let plan = prepare_run_id(&t);
        assert!(!plan.run_id.is_empty());
        assert!(plan.updates.is_empty());
    }
}

/// Return the shared run ID if every tray already carries the same `rv.run-id`.
fn existing_run_id(trays: &HashMap<String, Tray>) -> Option<String> {
    let mut ids = trays.values().filter_map(|t| t.rv_labels.get("rv.run-id"));
    let first = ids.next()?.clone();
    if ids.all(|id| *id == first) {
        Some(first)
    } else {
        None
    }
}
