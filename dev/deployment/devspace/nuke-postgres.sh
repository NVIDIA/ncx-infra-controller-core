#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
# SPDX-License-Identifier: Apache-2.0

MAX_RETRY=10
i=0
while [[ $i -lt $MAX_RETRY ]]; do
  echo "Attempting to delete forge DB."


  #something is holding the DB connection -- so murder it, i don't care this is local get out of the way.
  kubectl exec -ti postgres-0 -n postgres -- psql -U postgres -c "SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE datname = 'carbide'
  AND pid <> pg_backend_pid();"

  kubectl exec -ti postgres-0 -n postgres -- psql -U postgres -c "DROP DATABASE IF EXISTS carbide;"
  if [ $? -eq 0 ]; then
      echo "carbide DB successfully deleted"
      break
  else
      echo "DB still has connections, waiting to retry."
      sleep 2
  fi

  i=$((i+1))
done

echo "Recreating carbide db"
kubectl exec -ti postgres-0 -n postgres -- psql -U postgres -c 'CREATE DATABASE carbide with owner "carbide";'


