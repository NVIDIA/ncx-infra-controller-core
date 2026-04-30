/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::tests::common::api_fixtures::create_test_env;

#[tokio::test]
async fn test_power_shelf_controller_integration() {
    // Create a test environment
    let pool = sqlx_test::new_pool("postgresql://localhost/carbide_test").await;
    let env = create_test_env(pool).await;

    // Verify that the power shelf controller is available
    assert!(env.power_shelf_controller.lock().await.is_some());

    // Run a power shelf controller iteration (should not panic)
    env.run_power_shelf_controller_iteration().await;

    // Test the conditional iteration method
    let mut iteration_count = 0;
    env.run_power_shelf_controller_iteration_until_condition(5, || {
        iteration_count += 1;
        iteration_count >= 3 // Stop after 3 iterations
    })
    .await;

    assert_eq!(iteration_count, 3);
}
