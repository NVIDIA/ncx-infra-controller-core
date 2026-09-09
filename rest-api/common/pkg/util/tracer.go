// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package util

import (
	"context"

	"github.com/rs/zerolog"
	"go.opentelemetry.io/otel/attribute"
	oteltrace "go.opentelemetry.io/otel/trace"

	cotel "github.com/NVIDIA/infra-controller/rest-api/common/pkg/otel"
)

// TracerSpan preserves the existing handler tracing API while span creation
// is provided by the process-wide OpenTelemetry TracerProvider.
type TracerSpan struct{}

func NewTracerSpan() *TracerSpan {
	return &TracerSpan{}
}

// LoadFromContext returns the active span when ctx contains valid trace state.
func (*TracerSpan) LoadFromContext(ctx context.Context) (oteltrace.Span, bool) {
	span := oteltrace.SpanFromContext(ctx)
	if span.SpanContext().IsValid() {
		return span, true
	}
	return nil, false
}

// SetAttribute preserves the existing helper signature for handlers.
func (*TracerSpan) SetAttribute(span oteltrace.Span, attr attribute.KeyValue, _ zerolog.Logger) oteltrace.Span {
	cotel.SetAttribute(span, attr)
	return span
}

// CreateChildInContext starts a child through the global TracerProvider. The
// parent remains request-local and is taken from ctx, so this does not depend
// on an Echo-specific tracer stored under a custom context key.
func (*TracerSpan) CreateChildInContext(ctx context.Context, spanName string, logger zerolog.Logger) (context.Context, oteltrace.Span) {
	if ctx == nil {
		logger.Warn().Msg("input context is nil, can't create child spanner")
		return ctx, nil
	}
	if spanName == "" {
		logger.Warn().Msg("spanner name is empty, can't create child spanner")
		return ctx, nil
	}

	return cotel.StartSpan(ctx, spanName)
}
