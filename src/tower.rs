use std::{marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

use rootcause::prelude::ResultExt;

use crate::error::ServiceError;

/// Tower compat trait

impl<
    Req: Sync + Send + 'static,
    Resp: Sync + Send + 'static,
    Err: Sync + Send + 'static + core::error::Error,
    T: tower::Service<Req, Response = Resp, Error = Err> + Sync + Send + 'static,
> crate::Service<Req> for TowerCompat<Req, Resp, T>
where
    <T as tower::Service<Req>>::Future: Send,
{
    type Resp = Resp;

    async fn request(&self, msg: Req) -> Result<Self::Resp, ServiceError> {
        let resp = self.tower_service.lock().await.call(msg).await;
        Ok(resp.context("ee").into_report()?)
    }
}

pub struct TowerCompat<
    Req: Sync + Send + 'static,
    Resp: Sync + Send + 'static,
    T: tower::Service<Req, Response = Resp> + Sync + Send + 'static,
> {
    tower_service: Arc<Mutex<T>>,
    _phantom: PhantomData<(Req, Resp)>,
}
