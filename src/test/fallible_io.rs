use crate::Service;

/// Simulates a fallible IO to test resiliency of
/// services. This fallible IO takes a service and a
/// set of messages that the service can receive, and
/// attempts
pub struct FallibleIo<M: Clone, S: Service<Req = M> + Clone> {
    service: S,
    all_possible_message_states: Vec<Vec<M>>,
}

impl<M: Clone, S: Service<Req = M> + Clone> FallibleIo<M, S> {
    pub fn init(service: S, messages: &[M]) -> Self
    where
        M: Clone,
    {
        // Powerset of messages
        let all_possible_message_states = (0..2usize.pow(messages.len() as u32))
            .map(|i| {
                messages
                    .iter()
                    .enumerate()
                    .filter(|&(t, _)| (i >> t) % 2 == 1)
                    .map(|(_, element)| element.clone())
                    .collect()
            })
            .collect();

        Self {
            all_possible_message_states,
            service,
        }
    }

    pub async fn test(self) -> Result<(), ()> {
        for messages in self.all_possible_message_states {
            let s = self.service.clone();
            for message in messages {
                s.request(message).await.map_err(|_| ())?;
            }
        }

        Ok(())
    }
}
