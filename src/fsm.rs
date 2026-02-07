use std::hash::{DefaultHasher, Hash, Hasher};

use crate::Service;

/// A wrapper for finite state machines that contains
/// a hash chain. When calling FSMs over fallible I/O,
/// this can be used to guarantee that the state machine
/// is in the expected state, and is resilient to failures.
///
/// The hash is a value that the caller needs to keep track
/// of across requests.
pub struct FsmHandler<S: Service> {
    inner: S,
    hash: u64,
}

impl<S: Service> From<S> for FsmHandler<S> {
    fn from(value: S) -> Self {
        Self {
            inner: value,
            hash: rand::random(),
        }
    }
}

impl<Req: Send, S: Service<Req = Req> + Sync> Service for FsmHandler<S> {
    type Req = (S::Req, u64);
    type Resp = Option<(S::Resp, u64)>;

    fn request(
        &self,
        msg: Self::Req,
    ) -> impl std::future::Future<Output = Result<Self::Resp, crate::error::ServiceError>> + Send
    {
        async move {
            match self.inner.request(msg.0).await {
                Ok(yay) => {
                    if self.hash == msg.1 {
                        let new_hash = {
                            let mut s = DefaultHasher::new();
                            msg.1.hash(&mut s);
                            s.finish()
                        };

                        Ok(Some((yay, new_hash)))
                    } else {
                        Ok(None)
                    }
                }
                Err(aw) => Err(aw),
            }
        }
    }
}
