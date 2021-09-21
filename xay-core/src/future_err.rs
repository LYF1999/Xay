use futures::{ready, Future};
use pin_project::pin_project;
use std::{fmt::Display, pin::Pin, task::Poll};

#[pin_project]
pub struct WithLogFuture<F, T, E: Display + Send>
where
    F: Future<Output = Result<T, E>>,
{
    #[pin]
    inner: F,
}

impl<F, T, E> Future for WithLogFuture<F, T, E>
where
    E: Display + Send,
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<<Self as futures::Future>::Output> {
        let future = self.as_mut().project().inner;
        let output: Result<T, E> = ready!(future.poll(cx));
        match output {
            o @ Ok(_) => Poll::Ready(o),
            Err(err) => {
                tracing::error!("{}", err);
                Poll::Ready(Err(err))
            }
        }
    }
}

pub trait WithResultLog<T, E>: Future<Output = Result<T, E>>
where
    E: Display + Send,
{
    fn with_log_err(self) -> WithLogFuture<Self, T, E>
    where
        Self: Sized,
    {
        WithLogFuture { inner: self }
    }
}

impl<F: ?Sized, T, E> WithResultLog<T, E> for F
where
    E: Display + Send,
    F: Future<Output = Result<T, E>>,
{
}
