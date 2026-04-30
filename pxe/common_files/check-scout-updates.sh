#!/bin/bash
# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

MIN_UPTIME=86400
ROOTFS_INFO_FILE="/rootfs_info.txt"

# If we've not reached the minimum uptime, exit straight away
uptime=$(awk -F. '{print $1}' < /proc/uptime)
if (( $uptime < $MIN_UPTIME ))
then
	echo "Min uptime not reached ($uptime < $MIN_UPTIME)"
	exit 0
fi

# Get the last updated value for the main scout image.
# Use the PXE URL from the kernel command line if available (supports
# external hosts that can't resolve internal hostnames), otherwise fall
# back to the default internal hostname.
pxe_uri=$(sed 's/ /\n/g' /proc/cmdline | grep '^pxe_uri=' | cut -d'=' -f2)
static_pxe_base_url=${pxe_uri:-http://carbide-static-pxe.forge}
arch=$(uname -m)
scout_url="${static_pxe_base_url}/public/blobs/internal/${arch}/scout.cpio.zst"
www_last_modified_str=$(curl -sf --head ${scout_url} 2>/dev/null | sed 's/\r//g' | grep Last-Modified)
if (( $? != 0 ))
then
	echo "Unable to query last-modified value for ${scout_url}"
	exit 1
fi

# Get the last updated value from when we booted
# This should have been stored by the scout-loader script
my_last_modified_str=$(grep Last-Modified ${ROOTFS_INFO_FILE})
if (( $? != 0 ))
then
	echo "Unable to query last-modified value from when we booted"
	echo "Scout loader should have put that in ${ROOTFS_INFO_FILE}"
	exit 1
fi

if [ "${www_last_modified_str}" != "${my_last_modified_str}" ]
then
	echo "Newer scout available, rebooting"
	reboot
fi
