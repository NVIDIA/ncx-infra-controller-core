# On-Demand Rack Maintenance

On-demand maintenance allows an operator to trigger a maintenance cycle on a rack that is in the **Ready** or **Error** state. It supports both **full-rack** and **partial-rack** scoping — the caller can optionally specify which machines, switches, or power shelves to maintain.

## Scope: Full Rack vs Partial Rack

The maintenance request carries an optional **`MaintenanceScope`** that specifies which devices to include:

| Scenario | `machine_ids` | `switch_ids` | `power_shelf_ids` | Effect |
|----------|--------------|--------------|-------------------|--------|
| Full rack | *(empty)* | *(empty)* | *(empty)* | All machines, switches, and power shelves in the rack are maintained. |
| Machines only | `[id1, id2]` | *(empty)* | *(empty)* | Only the specified machines are reprovisioned and firmware-upgraded. Switches and power shelves are skipped. |
| Mixed | `[id1]` | `[sid1]` | *(empty)* | Only the listed machines and switches are maintained; power shelves are skipped. |

When no device IDs are specified (all three lists empty), the scope is treated as **full rack** — identical to the existing `reprovision_requested` behavior.

## Flow

```text
┌────────┐       ┌──────────────┐       ┌──────────┐       ┌─────────────────┐
│ Caller │──────▶│ gRPC Endpoint│──────▶│ Postgres │       │ Rack State      │
│ (CLI)  │       │ OnDemandRack │       │          │       │ Handler (Ready) │
│        │       │ Maintenance  │       │          │       │                 │
└────────┘       └──────┬───────┘       └────┬─────┘       └────────┬────────┘
                        │                    │                      │
                        │ 1. Load rack       │                      │
                        │    verify Ready    │                      │
                        │ 2. Set config.     │                      │
                        │    maintenance_    │                      │
                        │    requested=scope │                      │
                        │──────────────────▶ │                      │
                        │                    │                      │
                        │ 3. Return OK       │                      │
                        │◀───────────────────│                      │
                        │                    │                      │
                        │                    │  4. Poll rack (Ready)│
                        │                    │◀─────────────────────│
                        │                    │                      │
                        │                    │  5. Detect           │
                        │                    │     maintenance_     │
                        │                    │     requested        │
                        │                    │                      │
                        │                    │  6. Transition to    │
                        │                    │     Maintenance      │
                        │                    │    (FirmwareUpgrade/  │
                        │                    │     Start)           │
                        │                    │◀─────────────────────│
```

1. The caller invokes the `OnDemandRackMaintenance` gRPC method with a `rack_id` and optional device-ID lists.
2. The handler validates that the rack is in `Ready` or `Error` state and no maintenance is already pending.
3. It writes a `MaintenanceScope` to `RackConfig.maintenance_requested` and persists the config.
4. On the next state-handler tick, `handle_ready` detects `maintenance_requested` and transitions the rack to `Maintenance(FirmwareUpgrade(Start))`.
5. The maintenance handler consumes the scope, filtering device reprovisioning and firmware-upgrade operations to only the specified devices (or all devices if the scope is full-rack).
6. After maintenance completes, the rack flows through `Validating` back to `Ready` as usual.

## gRPC API

**Service method** (in `Forge` service):

```protobuf
rpc OnDemandRackMaintenance(RackMaintenanceOnDemandRequest) returns (RackMaintenanceOnDemandResponse);
```

**Messages**:

```protobuf
message RackMaintenanceOnDemandRequest {
  common.RackId rack_id = 1;
  repeated string machine_ids = 2;
  repeated string switch_ids = 3;
  repeated string power_shelf_ids = 4;
}

message RackMaintenanceOnDemandResponse {}
```

## Preconditions

The gRPC handler rejects the request with an error if:

- The rack is **not in `Ready` or `Error` state** — maintenance can only be triggered from these two states.
- A maintenance request is **already pending** (`maintenance_requested` is already set).
- Any provided device ID is **malformed** (cannot be parsed).

## RBAC

The `OnDemandRackMaintenance` permission is granted to the `ForgeAdminCLI` role.

## Ready State Priority

When the rack is in `Ready`, three config flags are checked in order. The first match wins:

1. **`topology_changed`** → transition to `Discovering`
2. **`reprovision_requested`** → transition to `Maintenance(FirmwareUpgrade/Start)` *(clears any pending `maintenance_requested`)*
3. **`maintenance_requested`** → transition to `Maintenance(FirmwareUpgrade/Start)` with device scope

## Implementation

| Component | Location |
|-----------|----------|
| Scope model (`MaintenanceScope`) | `crates/api-model/src/rack.rs` |
| Config field (`maintenance_requested`) | `RackConfig` in the same file |
| gRPC handler | `on_demand_rack_maintenance` in `crates/api/src/handlers/rack.rs` |
| API wiring | `crates/api/src/api.rs` |
| RBAC rule | `crates/api/src/auth/internal_rbac_rules.rs` |
| Ready state check | `handle_ready` in `crates/api/src/state_controller/rack/ready.rs` |
| Device filtering in maintenance | `handle_maintenance` in `crates/api/src/state_controller/rack/maintenance.rs` |
| Protobuf definitions | `crates/rpc/proto/forge.proto` |
