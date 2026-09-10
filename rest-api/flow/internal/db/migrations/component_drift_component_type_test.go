// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package migrations_test

import (
	"context"
	_ "embed"
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/require"

	"github.com/NVIDIA/infra-controller/rest-api/flow/internal/eventrule/store/storetest"
)

//go:embed 20260909120000_component_drift_component_type.up.sql
var componentDriftComponentTypeUp string

//go:embed 20260909120000_component_drift_component_type.down.sql
var componentDriftComponentTypeDown string

func TestComponentDriftComponentTypeMigration(t *testing.T) {
	ctx := context.Background()
	session := storetest.NewPostgresTestSession(t)

	_, err := session.DB.ExecContext(ctx, componentDriftComponentTypeDown)
	require.NoError(t, err)

	componentID := uuid.New()
	_, err = session.DB.ExecContext(
		ctx,
		`INSERT INTO component (id, type, manufacturer, serial_number, rack_id)
		 VALUES (?, 'Compute', 'test', 'compute-1', ?)`,
		componentID,
		uuid.New(),
	)
	require.NoError(t, err)

	_, err = session.DB.ExecContext(
		ctx,
		`INSERT INTO component_drift (component_id, drift_type, diffs)
		 VALUES (?, 'missing_in_actual', '[]'),
		        (NULL, 'missing_in_expected', '[]')`,
		componentID,
	)
	require.NoError(t, err)

	_, err = session.DB.ExecContext(ctx, componentDriftComponentTypeUp)
	require.NoError(t, err)

	var componentTypes []*string
	err = session.DB.NewSelect().
		Table("component_drift").
		Column("component_type").
		OrderExpr("component_id NULLS LAST").
		Scan(ctx, &componentTypes)
	require.NoError(t, err)
	require.Len(t, componentTypes, 2)
	require.NotNil(t, componentTypes[0])
	require.Equal(t, "Compute", *componentTypes[0])
	require.Nil(t, componentTypes[1], "an unlinked predecessor row must not be assigned a guessed type")

	_, err = session.DB.ExecContext(
		ctx,
		`INSERT INTO component_drift (external_id, drift_type, diffs)
		 VALUES ('predecessor-write', 'missing_in_expected', '[]')`,
	)
	require.NoError(t, err, "the additive column must remain optional for a predecessor writer")
}
