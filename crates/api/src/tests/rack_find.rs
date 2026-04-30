/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use common::api_fixtures::create_test_env;
use rpc::forge::forge_server::Forge;

use crate::tests::common;
use crate::tests::common::api_fixtures::site_explorer::TestRackDbBuilder;

#[crate::sqlx_test]
async fn test_find_rack_by_id(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let rack_id1: RackId = "Rack1".parse().unwrap();
    let rack_id2: RackId = "Rack2".parse().unwrap();
    let mut txn = env.pool.acquire().await.unwrap();
    TestRackDbBuilder::new()
        .with_rack_id(rack_id1.clone())
        .persist(&mut txn)
        .await
        .unwrap();
    TestRackDbBuilder::new()
        .with_rack_id(rack_id2.clone())
        .persist(&mut txn)
        .await
        .unwrap();
    drop(txn);

    let rack_ids = env
        .api
        .find_rack_ids(tonic::Request::new(rpc::forge::RackSearchFilter::default()))
        .await
        .unwrap()
        .into_inner()
        .rack_ids;
    assert_eq!(rack_ids, vec![rack_id1.clone(), rack_id2.clone()]);

    let racks = env
        .api
        .find_racks_by_ids(tonic::Request::new(rpc::forge::RacksByIdsRequest {
            rack_ids: vec![rack_id1.clone()],
        }))
        .await
        .unwrap()
        .into_inner()
        .racks;
    assert_eq!(racks.len(), 1);
    assert_eq!(racks[0].id, Some(rack_id1));

    let racks = env
        .api
        .find_racks_by_ids(tonic::Request::new(rpc::forge::RacksByIdsRequest {
            rack_ids: vec![rack_id2.clone()],
        }))
        .await
        .unwrap()
        .into_inner()
        .racks;
    assert_eq!(racks.len(), 1);
    assert_eq!(racks[0].id, Some(rack_id2));
}
