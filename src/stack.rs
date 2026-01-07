use std::marker::PhantomData;

use rootcause::report_collection::ReportCollection;

use crate::{Service, error::ServiceError};

impl<Req: Send + Sync + 'static, Resp: Send + Sync + 'static, T: Service<Req, Resp = Resp>>
    StackService<Req, Resp> for T
{
}

pub trait StackService<Req: Send + Sync + 'static, Resp: Send + Sync + 'static>:
    Service<Req, Resp = Resp>
{
    /// Insert a service to be executed first, which then executes this current service.
    fn with_before<
        AsReq: Send + Sync + 'static,
        Bs: Service<AsReq, Resp = Req> + Sync + Send + 'static,
    >(
        self,
        before_service: Bs,
    ) -> Stack<AsReq, Req, Bs, Self>
    where
        Self: Sync + Send + 'static + Sized,
    {
        Stack {
            before_service,
            after_service: self,
            _phantom: PhantomData,
        }
    }

    /// Insert a new service to be executed immediately after this current service.
    fn then_after<AsResp, As: Service<Self::Resp, Resp = AsResp> + Sync + Send + 'static + Sized>(
        self,
        after_service: As,
    ) -> Stack<Req, Self::Resp, Self, As>
    where
        Self: Sync + Send + 'static + Sized,
    {
        Stack {
            before_service: self,
            after_service,
            _phantom: PhantomData,
        }
    }
}
pub struct Stack<
    BsReq: Send + Sync + 'static,
    AsReq: Send + Sync + 'static,
    Bs: Service<BsReq, Resp = AsReq> + Send + Sync + 'static,
    As: Service<AsReq> + Send + Sync + 'static,
> {
    pub(crate) before_service: Bs,
    pub(crate) after_service: As,
    _phantom: PhantomData<(BsReq, AsReq)>,
}

impl<
    BsReq: Send + Sync + 'static,
    AsReq: Send + Sync + 'static,
    Bs: Service<BsReq, Resp = AsReq> + Send + Sync + 'static,
    As: Service<AsReq> + Send + Sync + 'static,
> Service<BsReq> for Stack<BsReq, AsReq, Bs, As>
{
    type Resp = As::Resp;

    async fn request(&self, msg: BsReq) -> Result<Self::Resp, ServiceError> {
        let mut rc = ReportCollection::new_sendsync();

        match self.before_service.request(msg).await {
            Ok(tsr) => match self.after_service.request(tsr).await {
                Ok(bsr) => {
                    return Ok(bsr);
                }
                Err(err) => {
                    rc.push(err.into_cloneable());
                }
            },
            Err(err) => {
                rc.push(err.into_cloneable());
            }
        };

        Err(rc
            .context("Inner stack service failed")
            .into_dynamic()
            .into_cloneable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct AddOneService;

    impl Service<u64> for AddOneService {
        type Resp = u64;

        async fn request(&self, msg: u64) -> Result<Self::Resp, ServiceError> {
            Ok(msg + 1)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DoubleService;

    impl Service<u64> for DoubleService {
        type Resp = u64;

        async fn request(&self, msg: u64) -> Result<Self::Resp, ServiceError> {
            Ok(msg * 2)
        }
    }

    #[tokio::test]
    async fn stack() {
        let add_one_service = AddOneService;
        let double_service = DoubleService;

        // (1 + 1) * 2
        assert_eq!(
            4,
            add_one_service
                .then_after(double_service)
                .request(1)
                .await
                .unwrap()
        );

        // (1 * 2) + 1
        assert_eq!(
            3,
            double_service
                .then_after(add_one_service)
                .request(1)
                .await
                .unwrap()
        );
    }
}
