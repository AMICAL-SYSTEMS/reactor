use tokio::{sync::mpsc, task::JoinHandle};

use crate::{Service, error::ServiceError, task::TaskService};

const DEFAULT_MSG_BUFFER_SIZE: usize = 64;

pub struct Actor<
    Req: Sync + Send + 'static,
    Resp: Sync + Send + 'static,
    S: Service<Req = Req, Resp = Resp> + Sync + Send + 'static,
> {
    tx: mpsc::Sender<Req>,
    rx: mpsc::Receiver<S::Resp>,
    join: JoinHandle<()>,
}

impl<
    Req: Send + Sync + 'static,
    Resp: Send + Sync + 'static,
    T: Service<Req = Req, Resp = Resp> + Send + Sync + 'static,
> ActorService<Req, Resp, T> for T
{
}

impl<
    Req: Sync + Send + 'static,
    Resp: Sync + Send + 'static,
    S: Service<Req = Req, Resp = Resp> + Sync + Send,
> Actor<Req, Resp, S>
{
    pub async fn send(&self, msg: Req) -> Result<(), ()> {
        self.tx.send(msg).await.map_err(|_| ())?;

        Ok(())
    }

    // remove &mut?
    pub async fn recv(&mut self) -> Option<Resp> {
        self.rx.recv().await
    }

    pub fn stop(self) {
        self.join.abort();
    }
}

pub trait ActorService<
    Req: Sync + Send + 'static,
    Resp: Sync + Send + 'static,
    S: Service<Req = Req, Resp = Resp> + Send + Sync + 'static + Sized,
>: Service<Req = Req, Resp = Resp> + Send + Sync + 'static + Sized
{
    fn into_actor(
        self,
    ) -> impl std::future::Future<Output = Result<Actor<Req, Resp, S>, ServiceError>> + Send {
        async {
            let (req_tx, mut req_rx) = mpsc::channel::<Req>(DEFAULT_MSG_BUFFER_SIZE);
            let (resp_tx, resp_rx) = mpsc::channel::<S::Resp>(DEFAULT_MSG_BUFFER_SIZE);

            let join = TaskService::default()
                .request(async move {
                    while let Some(msg) = req_rx.recv().await {
                        let resp = self.request(msg).await.unwrap();
                        resp_tx.send(resp).await.unwrap();
                    }
                })
                .await?;

            Ok(Actor {
                tx: req_tx,
                rx: resp_rx,
                join,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::ServiceError, stack::StackService};

    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct AddOneService;

    impl Service for AddOneService {
        type Req = u64;
        type Resp = u64;

        async fn request(&self, msg: u64) -> Result<Self::Resp, ServiceError> {
            Ok(msg + 1)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DoubleService;

    impl Service for DoubleService {
        type Req = u64;
        type Resp = u64;

        async fn request(&self, msg: u64) -> Result<Self::Resp, ServiceError> {
            Ok(msg * 2)
        }
    }

    #[tokio::test]
    async fn actor() {
        let add_one_service = AddOneService;
        let double_service = DoubleService;

        {
            //let mut actor = Stack::start_actor(add_one_service.then_after(double_service));
            let mut actor = add_one_service
                .then_after(double_service)
                .into_actor()
                .await
                .unwrap();

            actor.send(1).await.unwrap();
            // (1 + 1) * 2
            assert_eq!(4, actor.recv().await.unwrap());
        }
    }
}
