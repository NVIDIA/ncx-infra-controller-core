// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package temporal

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel"

	cotel "github.com/NVIDIA/infra-controller/rest-api/common/pkg/otel"
)

func TestClientOptions(t *testing.T) {
	tcs := []struct {
		descr            string
		bootstrapEnabled bool
		endpoint         string
		serviceName      string
		propagators      string
		wantInterceptors bool
	}{
		{
			descr:            "tracing interceptor attached when provider is active",
			bootstrapEnabled: true,
			endpoint:         "http://localhost:14318",
			wantInterceptors: true,
		},
		{
			descr:            "environment service name keeps interceptor active",
			bootstrapEnabled: true,
			endpoint:         "http://localhost:14318",
			serviceName:      "workflow-from-environment",
			wantInterceptors: true,
		},
		{
			descr:            "propagation remains when config disables export",
			endpoint:         "http://localhost:14318",
			wantInterceptors: true,
		},
		{
			descr:            "propagation remains without exporter endpoint",
			bootstrapEnabled: true,
			wantInterceptors: true,
		},
		{
			descr:       "explicitly disabled propagation without export",
			propagators: "none",
		},
	}

	for _, tc := range tcs {
		t.Run(tc.descr, func(t *testing.T) {
			previousProvider := otel.GetTracerProvider()
			previousPropagator := otel.GetTextMapPropagator()
			t.Cleanup(func() {
				otel.SetTracerProvider(previousProvider)
				otel.SetTextMapPropagator(previousPropagator)
			})
			t.Setenv("OTEL_EXPORTER_OTLP_ENDPOINT", tc.endpoint)
			t.Setenv("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "")
			t.Setenv("OTEL_TRACES_SAMPLER", "always_off")
			t.Setenv("OTEL_PROPAGATORS", tc.propagators)
			t.Setenv("OTEL_SERVICE_NAME", tc.serviceName)
			shutdown, err := cotel.Bootstrap(context.Background(), tc.bootstrapEnabled, "temporal-test")
			require.NoError(t, err)
			t.Cleanup(func() { require.NoError(t, shutdown(context.Background())) })

			opts, err := ClientOptions("temporal:7233", "cloud", nil, nil)
			require.NoError(t, err)

			assert.Equal(t, "temporal:7233", opts.HostPort)
			assert.Equal(t, "cloud", opts.Namespace)
			assert.NotNil(t, opts.DataConverter)
			if tc.wantInterceptors {
				assert.NotEmpty(t, opts.Interceptors, "expected the OTel tracing interceptor")
			} else {
				assert.Empty(t, opts.Interceptors)
			}
		})
	}
}
