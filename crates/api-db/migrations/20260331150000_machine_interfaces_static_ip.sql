-- Add is_static_ip column to machine_interfaces table to distinguish
-- statically-assigned BMC IPs (from ExpectedMachine.ip_address or
-- ExpectedPowerShelf.ip_address) from DHCP-discovered IPs.
--
-- This helps users understand when they see an IP address that may be
-- outside of Carbide-managed subnets.

ALTER TABLE machine_interfaces
ADD COLUMN is_static_ip BOOLEAN NOT NULL DEFAULT FALSE;

-- Create an index for faster lookups when checking if an IP is static
CREATE INDEX idx_machine_interfaces_is_static_ip ON machine_interfaces(is_static_ip)
WHERE is_static_ip = TRUE;

-- Backfill existing static IPs by checking if the MAC address exists in
-- expected_machines or expected_power_shelves with a non-null ip_address
UPDATE machine_interfaces mi
SET is_static_ip = TRUE
WHERE EXISTS (
    SELECT 1 FROM expected_machines em
    WHERE em.bmc_mac_address = mi.mac_address
    AND em.ip_address IS NOT NULL
)
OR EXISTS (
    SELECT 1 FROM expected_power_shelves eps
    WHERE eps.bmc_mac_address = mi.mac_address
    AND eps.ip_address IS NOT NULL
);
