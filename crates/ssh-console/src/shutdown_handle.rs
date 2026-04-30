/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::future;

use futures_util::FutureExt;
use tokio::sync::oneshot;

/// Convenience trait for a task with a shutdown handle (in the form of a [`oneshot::Sender<()>`])
///
/// The shutdown handle must be treated such that dropping it means "shut down now", (because any
/// call which is awaiting the channel will immediately return.) By convention, dropping the
/// channel and sending the shutdown message mean the same thing.
pub trait ShutdownHandle<R> {
    fn into_parts(self) -> (oneshot::Sender<()>, tokio::task::JoinHandle<R>);

    fn shutdown_and_wait(self) -> impl std::future::Future<Output = R> + Send
    where
        Self: Send + Sized,
        R: Send,
    {
        async move {
            let (shutdown_tx, join_handle) = self.into_parts();
            // Let the shutdown handle drop, which causes any reads to finish (semantically the same as
            // sending an empty tuple over the channel, both mean "shut down now").
            std::mem::drop(shutdown_tx);
            join_handle.await.expect("task panicked")
        }
    }
}

pub trait ReadyHandle {
    fn take_ready_rx(&mut self) -> Option<oneshot::Receiver<()>>;

    fn wait_until_ready(
        &mut self,
    ) -> impl std::future::Future<Output = Result<(), oneshot::error::RecvError>> + Send {
        if let Some(ready_rx) = self.take_ready_rx() {
            ready_rx.boxed()
        } else {
            future::ready(Ok(())).boxed()
        }
    }
}
