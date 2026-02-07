use std::marker::PhantomData;

use tokio::task::JoinHandle;

use crate::Service;

pub struct TaskService<F> {
    _fut: PhantomData<F>,
}

impl<F> Default for TaskService<F> {
    fn default() -> Self {
        Self { _fut: PhantomData }
    }
}

impl<F: Future<Output: Send + 'static> + Send + 'static> Service for TaskService<F> {
    type Req = F;
    type Resp = JoinHandle<F::Output>;

    fn request(
        &self,
        msg: Self::Req,
    ) -> impl std::future::Future<Output = Result<Self::Resp, crate::error::ServiceError>> + Send
    {
        async { Ok(tokio::spawn(msg)) }
    }
}
