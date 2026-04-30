#!/bin/bash
# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

find /var/support/core/ -maxdepth 1 -type f -printf '%T+ %p\n' | sort | head -n -1 | cut -d' ' -f2- | xargs -r -I {} rm -v {}

