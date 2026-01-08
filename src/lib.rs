use crate::error::ServiceError;

pub mod actor;
pub mod error;
pub mod stack;
pub mod tower;

#[allow(async_fn_in_trait)]
pub trait Service {
    type Req;
    type Resp;

    fn request(
        &self,
        msg: Self::Req,
    ) -> impl std::future::Future<Output = Result<Self::Resp, ServiceError>> + Send;
}
