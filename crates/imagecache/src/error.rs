/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[derive(thiserror::Error, Debug)]
pub enum ImageCacheError {
    #[error("API error: {0}")]
    Api(#[from] tonic::Status),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("File too large: {size} bytes exceeds limit of {limit} bytes")]
    TooLarge { size: u64, limit: u64 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
