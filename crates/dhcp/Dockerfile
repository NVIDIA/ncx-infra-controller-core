# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

WORKDIR /app/code

RUN mkdir /var/run/kea/
RUN mkdir /var/lib/kea/

# This is copying the examples instead of the dev/ resource that the base image contains
# Should this be removed altogether?
COPY examples/kea-dhcp4-carbide.conf /etc/kea/kea-dhcp4-carbide.conf


EXPOSE 1079
EXPOSE 1089
