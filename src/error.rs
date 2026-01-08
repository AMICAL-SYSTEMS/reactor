use std::marker::PhantomData;

use crate::Service;

use rootcause::markers::{Cloneable, Dynamic};
pub use rootcause::prelude::*;
pub type ServiceError = rootcause::Report<Dynamic, Cloneable>;

impl<Req: Send + Sync + 'static, Resp: Send + Sync + 'static, T: Service<Req = Req, Resp = Resp>>
    ErrorContext<Req, Resp> for T
{
}

pub trait ErrorContext<Req: Send + Sync + 'static, Resp: Send + Sync + 'static>:
    Service<Req = Req, Resp = Resp>
where
    Self: Sized,
{
    /// Insert a new service to be executed immediately after this current service.
    fn with_error_context(self, context: &'static str) -> ErrorContextService<Req, Resp, Self> {
        ErrorContextService {
            inner: self,
            context,
            _phantom: PhantomData,
        }
    }
}

pub struct ErrorContextService<
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
    S: Service<Req = Req, Resp = Resp>,
> {
    inner: S,
    context: &'static str,
    _phantom: PhantomData<(Req, Resp)>,
}

impl<
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
    S: Service<Req = Req, Resp = Resp> + Sync,
> Service for ErrorContextService<Req, Resp, S>
{
    type Req = Req;
    type Resp = Resp;

    async fn request(&self, msg: Req) -> Result<Self::Resp, ServiceError> {
        self.inner
            .request(msg)
            .await
            .context(self.context)
            .map_err(|report| report.into_dynamic().into_cloneable())
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::StackService;

    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct AddOneService;

    impl Service for AddOneService {
        type Req = u64;
        type Resp = u64;

        async fn request(&self, msg: u64) -> Result<Self::Resp, ServiceError> {
            if msg > 2 {
                bail!("Err can't add msg > 2")
            } else {
                Ok(msg + 1)
            }
        }
    }

    #[tokio::test]
    async fn error() {
        let add_one_service = AddOneService;

        add_one_service
            .with_error_context("Adding one 1")
            .then_after(add_one_service)
            .with_error_context("Adding one 2")
            .then_after(add_one_service)
            .with_error_context("Adding one 3")
            .then_after(add_one_service)
            .with_error_context("Adding one 4")
            .then_after(add_one_service)
            .with_error_context("Adding one 5")
            .request(1)
            .await
            .unwrap_err();
    }
}
