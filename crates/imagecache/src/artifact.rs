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

use rpc::forge::IpxeTemplateArtifactCacheStrategy;

pub fn is_eligible(
    cache_strategy: IpxeTemplateArtifactCacheStrategy,
    cache_as_needed: bool,
) -> bool {
    match cache_strategy {
        IpxeTemplateArtifactCacheStrategy::CachedOnly => true,
        IpxeTemplateArtifactCacheStrategy::CacheAsNeeded => cache_as_needed,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_only_is_always_eligible() {
        assert!(is_eligible(
            IpxeTemplateArtifactCacheStrategy::CachedOnly,
            false
        ));
        assert!(is_eligible(
            IpxeTemplateArtifactCacheStrategy::CachedOnly,
            true
        ));
    }

    #[test]
    fn cache_as_needed_depends_on_flag() {
        assert!(!is_eligible(
            IpxeTemplateArtifactCacheStrategy::CacheAsNeeded,
            false
        ));
        assert!(is_eligible(
            IpxeTemplateArtifactCacheStrategy::CacheAsNeeded,
            true
        ));
    }

    #[test]
    fn local_only_is_never_eligible() {
        assert!(!is_eligible(
            IpxeTemplateArtifactCacheStrategy::LocalOnly,
            true
        ));
        assert!(!is_eligible(
            IpxeTemplateArtifactCacheStrategy::LocalOnly,
            false
        ));
    }

    #[test]
    fn remote_only_is_never_eligible() {
        assert!(!is_eligible(
            IpxeTemplateArtifactCacheStrategy::RemoteOnly,
            true
        ));
        assert!(!is_eligible(
            IpxeTemplateArtifactCacheStrategy::RemoteOnly,
            false
        ));
    }
}
