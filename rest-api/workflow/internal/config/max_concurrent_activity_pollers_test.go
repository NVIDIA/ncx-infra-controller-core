// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

package config

import (
	"os"
	"testing"
)

func TestMaxConcurrentActivityPollersEdgeCases(t *testing.T) {
	cases := []struct {
		name      string
		envVal    string // "" means unset
		wantPanic bool
		wantValue int
	}{
		{"unset uses default", "", false, DefaultMaxConcurrentActivityPollers},
		{"valid 40", "40", false, 40},
		{"valid 200 (max)", "200", false, 200},
		{"valid 1 (min)", "1", false, 1},
		{"zero rejected", "0", true, 0},
		{"negative rejected", "-5", true, 0},
		{"201 rejected (just over)", "201", true, 0},
		{"non-integer rejected", "abc", true, 0},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			config = nil // reset package-level singleton between cases
			if c.envVal == "" {
				os.Unsetenv(ConfigWorkerMaxConcurrentActivityPollersEnv)
			} else {
				os.Setenv(ConfigWorkerMaxConcurrentActivityPollersEnv, c.envVal)
				defer os.Unsetenv(ConfigWorkerMaxConcurrentActivityPollersEnv)
			}

			panicked := false
			var cfg *Config
			func() {
				defer func() {
					if r := recover(); r != nil {
						panicked = true
					}
				}()
				cfg = NewConfig()
			}()

			if c.wantPanic && !panicked {
				t.Errorf("env=%q: expected panic, got none (value=%d)", c.envVal, cfg.GetMaxConcurrentActivityPollers())
			}
			if !c.wantPanic {
				if panicked {
					t.Fatalf("env=%q: unexpected panic", c.envVal)
				}
				if got := cfg.GetMaxConcurrentActivityPollers(); got != c.wantValue {
					t.Errorf("env=%q: got %d, want %d", c.envVal, got, c.wantValue)
				}
			}
		})
	}
	config = nil // leave clean for other tests in the package
}
