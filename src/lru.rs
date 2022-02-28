//! # LRU cache provider
//!
//! This is an implementation of a [`CacheLayer`] provider using [`lru::LruCache`].

use crate::{ProviderRequest, ProviderResponse};
use lru::LruCache;
use std::{
    convert::Infallible,
    future::{ready, Future},
    hash::Hash,
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tower::Service;

/// Local LRU cache provider
#[derive(Debug, Clone)]
pub struct LruProvider<'a, K, V>
where
    K: Eq + Hash,
{
    inner: Arc<Mutex<LruCache<K, V>>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, K, V> LruProvider<'a, K, V>
where
    K: Eq + Hash,
{
    /// Create a new LRU cache provider with the desired capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(capacity))),
            _phantom: PhantomData,
        }
    }
}

impl<'a, K, V> Service<ProviderRequest<K, V>> for LruProvider<'a, K, V>
where
    K: Eq + Hash,
    V: Clone + Send + 'a,
{
    type Response = ProviderResponse<V>;
    type Error = Infallible;
    type Future = ProviderFuture<'a, V>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: ProviderRequest<K, V>) -> Self::Future {
        Box::pin(ready(Ok(match request {
            ProviderRequest::Get(key) => match self.inner.lock().unwrap().get(&key) {
                Some(value) => ProviderResponse::Found(value.clone()),
                None => ProviderResponse::NotFound,
            },
            ProviderRequest::Insert(key, value) => {
                self.inner.lock().unwrap().put(key, value.clone());
                ProviderResponse::Found(value)
            }
        })))
    }
}

type ProviderFuture<'a, V> =
    Pin<Box<dyn Future<Output = Result<ProviderResponse<V>, Infallible>> + Send + 'a>>;
