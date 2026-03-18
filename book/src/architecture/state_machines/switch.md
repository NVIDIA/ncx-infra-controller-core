# Switch State Diagram

This document describes the Finite State Machine (FSM) for Switches in Carbide: lifecycle from creation through configuration, validation, ready, optional reprovisioning, and deletion.

## High-Level Overview

The main flow shows the primary states and transitions:

<div style="width: 180%; background: white; margin-left: -40%;">

```plantuml
@startuml
skinparam state {
  BackgroundColor White
}

state "Initializing" as Initializing
state "Configuring\n(RotateOsPassword)" as Configuring
state "Validating" as Validating
state "BomValidating" as BomValidating
state "Ready" as Ready
state "ReProvisioning\n(Start → WaitFirmware)" as ReProvisioning
state "Error" as Error
state "Deleting" as Deleting

[*] --> Initializing : Switch created

Initializing --> Configuring : init complete
Configuring --> Validating : rotate password done
Validating --> BomValidating : validation complete
BomValidating --> Ready : BOM validation complete

Ready --> Deleting : marked for deletion
Ready --> ReProvisioning : reprovision requested

ReProvisioning --> Ready : firmware upgrade Completed
ReProvisioning --> Error : firmware upgrade Failed

Error --> Deleting : marked for deletion

Deleting --> [*] : final delete
@enduml
```

</div>

## States

| State | Description |
|-------|-------------|
| **Initializing** | Switch is created in Carbide; controller performs initial setup. |
| **Configuring** | Switch is being configured (rotate OS password). Sub-state: `RotateOsPassword`. |
| **Validating** | Switch is being validated. Sub-state: `ValidationComplete`. |
| **BomValidating** | BOM (Bill of Materials) validation. Sub-state: `BomValidationComplete`. |
| **Ready** | Switch is ready for use. From here it can be deleted, or reprovisioning can be requested. |
| **ReProvisioning** | Reprovisioning (e.g. firmware update) in progress. Sub-states: `Start`, `WaitFirmwareUpdateCompletion`. Completion is driven by `firmware_upgrade_status` (Completed → Ready, Failed → Error). |
| **Error** | Switch is in error (e.g. firmware upgrade failed). Can transition to Deleting if marked for deletion; otherwise waits for manual intervention. |
| **Deleting** | Switch is being removed; ends in final delete (terminal). |

## Transitions (by trigger)

| From | To | Trigger / Condition |
|------|-----|----------------------|
| *(create)* | Initializing | Switch created |
| Initializing | Configuring (RotateOsPassword) | Initialization complete |
| Configuring (RotateOsPassword) | Validating (ValidationComplete) | OS password rotated |
| Validating (ValidationComplete) | BomValidating (BomValidationComplete) | Validation complete |
| BomValidating (BomValidationComplete) | Ready | BOM validation complete |
| Ready | Deleting | `deleted` set (marked for deletion) |
| Ready | ReProvisioning (Start) | `switch_reprovisioning_requested` is set |
| ReProvisioning (Start) | ReProvisioning (WaitFirmwareUpdateCompletion) | Reprovision triggered |
| ReProvisioning (WaitFirmwareUpdateCompletion) | Ready | `firmware_upgrade_status == Completed` |
| ReProvisioning (WaitFirmwareUpdateCompletion) | Error | `firmware_upgrade_status == Failed { cause }` |
| Error | Deleting | `deleted` set (marked for deletion) |
| Deleting | *(end)* | Final delete committed |

## Implementation

- **State type**: `SwitchControllerState` in `crates/api-model/src/switch/mod.rs`.
- **Handlers**: `crates/api/src/state_controller/switch/` — one module per top-level state (`initializing`, `configuring`, `validating`, `bom_validating`, `ready`, `reprovisioning`, `error_state`, `deleting`).
- **Orchestration**: `SwitchStateHandler` in `handler.rs` delegates to the handler for the current `controller_state`.
