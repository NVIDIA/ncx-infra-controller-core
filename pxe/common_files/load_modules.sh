#!/bin/bash
# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

udevadm trigger --wait-daemon --type=devices --subsystem-match=pci --action=add --settle
