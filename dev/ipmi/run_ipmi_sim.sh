#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

DIR="$(cd "$(dirname "${0}")" && pwd)"
cd "${DIR}"
mkdir -p /tmp/ipmi_state
exec ipmi_sim -c "${DIR}/lan.conf" -f "${DIR}/cmd.conf" -s /tmp/ipmi_state
