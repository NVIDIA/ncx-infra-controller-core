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

use std::path::{Path, PathBuf};

use crate::error::ImageCacheError;
use crate::storage::StorageBackend;

pub struct LocalStorage {
    cache_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }
}

impl StorageBackend for LocalStorage {
    async fn object_exists(&self, key: &str) -> Result<bool, ImageCacheError> {
        let path = self.cache_dir.join(key);
        Ok(tokio::fs::try_exists(&path).await?)
    }

    async fn put_object_from_file(
        &self,
        key: &str,
        file_path: &Path,
    ) -> Result<(), ImageCacheError> {
        let dest = self.cache_dir.join(key);
        // Rename is atomic and O(1) when source and dest share a filesystem,
        // which is the common path (download temp_dir defaults to cache_dir).
        match tokio::fs::rename(file_path, &dest).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                // Cross-filesystem: fall back to copy-then-rename for atomicity.
                let temp_dest = self.cache_dir.join(format!(".{key}.tmp"));
                tokio::fs::copy(file_path, &temp_dest).await?;
                tokio::fs::rename(&temp_dest, &dest).await?;
                Ok(())
            }
            Err(e) => Err(ImageCacheError::Io(e)),
        }
    }
}
