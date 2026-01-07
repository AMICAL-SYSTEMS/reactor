use crate::error::ServiceError;

pub mod actor;
pub mod error;
pub mod stack;

#[allow(async_fn_in_trait)]
pub trait Service<Req: Sync + Send + 'static>: Send + Sync + 'static {
    type Resp: Sync + Send + 'static;

    fn request(
        &self,
        msg: Req,
    ) -> impl std::future::Future<Output = Result<Self::Resp, ServiceError>> + Send;
}
