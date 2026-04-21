# initialize_host_ingestion.py

A scraping tool that collects BMC MAC addresses, IPs, and chassis serial numbers from a live Forge/Carbide cluster and builds the expected-machine list needed to register hosts. It is designed to be **run repeatedly** — each run picks up new DHCP leases, retries previously failed machines, and accumulates results into a single output file. Run it as many times as needed, with different credential combinations, as hardware comes online or BMC passwords are rotated, until all machines are collected.

Requires `kubectl` access to the target cluster and network reachability to BMC IPs (port 443).

## Usage

```
python3 initialize_host_ingestion.py --credentials USER:PASS [--credentials USER2:PASS2] [OPTIONS]
```

`--credentials` is repeatable. On a 401, the script tries each credential set in order before giving up. Credentials are tried **sequentially** (not in parallel) to avoid triggering BMC account lockouts.

### Options

| Flag | Default | Description |
|---|---|---|
| `--credentials USER:PASS` | _(required for Redfish)_ | BMC credential pair. Repeat to supply fallback credentials. |
| `--output FILE` | `expected_machines.json` | Path to write (and read, for incremental runs) the output JSON. |
| `--no-redfish` | off | Skip all Redfish calls; serial numbers are left blank. Useful for a dry run or when BMCs are unreachable. |
| `--workers N` | `10` | Number of parallel Redfish workers. |
| `--upload` | off | After writing the output file, copy it to the `carbide-api` pod and run `expected-machine replace-all`. |

### Examples

First pass — collect with the default credential set:

```
python3 initialize_host_ingestion.py --credentials admin:secret
```

Second pass — add a fallback credential for BMCs that rejected the first set:

```
python3 initialize_host_ingestion.py \
  --credentials admin:secret \
  --credentials root:Carbide2026!
```

Collect and immediately push to the cluster:

```
python3 initialize_host_ingestion.py \
  --credentials admin:secret \
  --upload
```

Generate the MAC/IP list without contacting BMCs (useful when BMCs are not yet reachable):

```
python3 initialize_host_ingestion.py --no-redfish
```

## Incremental / resume behavior

Each run reads the existing `--output` file (if present) and skips any entries already marked `successfully_collected_data: true`. Only incomplete entries — auth failures, timeouts, missing serials — are retried. New MACs that appeared in DHCP since the last run are added automatically.

This means you can run the script on a partially provisioned rack, add more machines, change credentials, or wait for BMCs to come online, then run it again. Each pass makes forward progress. Run until `Complete: N/N` is reported, then use `--upload` to push the final result.

## What it does

1. Queries `machine_dhcp_records` in the `forge_system_carbide` Postgres database (via `kubectl exec` into `forge-pg-cluster-0`) to get the current MAC → IP mapping for all BMC interfaces.
2. For each MAC with a known IP, opens an HTTPS connection to the BMC and tries the configured credentials against the Redfish Chassis endpoint to retrieve the `SerialNumber`.
3. Writes results to an output JSON file containing two keys:
   - `all_machines` — every discovered MAC, with per-entry status, vendor info, and failure reasons.
   - `expected_machines` — the subset of entries where all required fields (`bmc_mac_address`, `bmc_username`, `bmc_password`, `chassis_serial_number`) were successfully collected.
4. Optionally uploads the output file to the `carbide-api` pod and runs `carbide-admin-cli expected-machine replace-all` to seed the cluster's expected-machine list in one step (`--upload`).

## Prerequisites

- `kubectl` configured and pointing at the target cluster.
- Network access from the workstation to the BMC IPs returned by DHCP (port 443).
- Python 3.6+.

## Output format

```json
{
  "expected_machines": [
    {
      "bmc_mac_address": "AA:BB:CC:DD:EE:FF",
      "bmc_username": "admin",
      "bmc_password": "secret",
      "chassis_serial_number": "ABC1234"
    }
  ],
  "all_machines": [
    {
      "successfully_collected_data": true,
      "ip_address": "10.0.1.5",
      "mac_address": "AA:BB:CC:DD:EE:FF",
      "bmc_username": "admin",
      "bmc_password": "secret",
      "chassis_serial_number": "ABC1234",
      "incomplete_reason": "",
      "vendor": "NVIDIA",
      "device_description": "BlueField BMC"
    }
  ]
}
```

`all_machines` entries that could not be completed include a non-empty `incomplete_reason`. Possible values:

| Reason | Meaning |
|---|---|
| `Connection refused` | Port 443 on the BMC IP was not open. |
| `Timeout after N attempts` | TCP connection timed out after all retries. |
| `HTTP 401/403 Unauthorized` | All supplied credential sets were rejected. A `credentials_tried` field lists what was attempted. |
| `Account locked out — too many failed authentication attempts` | The BMC returned an account-lockout message; further attempts were aborted immediately. |
| `Authenticated successfully but SerialNumber not found in Chassis response` | Auth succeeded but the `SerialNumber` field was absent or empty in the Redfish response. |
| `Redfish API exists but unable to collect the serial number from default paths` | Redfish root was reachable but neither chassis path returned a serial. |

## Redfish chassis paths probed

The script tries the following paths in order, stopping at the first that returns a `SerialNumber`:

- `/redfish/v1/Chassis/Chassis_0` — standard BMCs (Dell, Supermicro, HPE, etc.)
- `/redfish/v1/Chassis/Bluefield_BMC` — NVIDIA BlueField DPU BMCs

TLS certificate validation is disabled (`CERT_NONE`) since BMC self-signed certificates are the norm in this environment.
