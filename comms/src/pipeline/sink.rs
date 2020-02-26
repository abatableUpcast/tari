// Copyright 2020, The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use futures::{task::Context, Future, Sink, SinkExt};
use log::*;
use std::{error::Error, pin::Pin, task::Poll};
use tower::Service;

const LOG_TARGET: &str = "comms::pipeline::sink";

/// A service which forwards and messages it gets to the given Sink
#[derive(Clone)]
pub struct SinkService<TSink>(TSink);

impl<TSink> SinkService<TSink> {
    pub fn new(sink: TSink) -> Self {
        SinkService(sink)
    }
}

impl<T, TSink> Service<T> for SinkService<TSink>
where
    TSink: Sink<T> + Unpin + Clone + 'static,
    TSink::Error: Error + Send + Sync + 'static,
{
    // A boxed error gives the most flexibility when building a pipeline. Using TSink::Error, in practise, requires a
    // conversion between the error being used for other services and TSink::Error.
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Response = ();

    type Future = impl Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.0).poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, item: T) -> Self::Future {
        let mut sink = self.0.clone();
        trace!(target: LOG_TARGET, "Sending item to sink");
        async move { sink.send(item).await.map_err(Into::into) }
    }
}