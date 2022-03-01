//! # LRU cache provider
//!
//! This is an implementation of a cache provider for [`crate::CacheLayer`]
//! using [`lru::LruCache`].
//!
//! ## Usage
//!
//! ```rust
//! use std::convert::Infallible;
//! use tower::{ServiceBuilder, service_fn};
//! use tower_cache::{
//!     CacheLayer,
//!     lru::LruProvider,
//! };
//! async fn handler(req: String) -> Result<String, Infallible> {
//!     Ok(req.to_uppercase())
//! }
//!
//! // Initialize the cache provider service
//! let lru_provider = LruProvider::new::<String, String>(20);
//!
//! // Wrap the service with CacheLayer.
//! let my_service = ServiceBuilder::new()
//!     .layer(CacheLayer::new(lru_provider))
//!     .service(service_fn(handler));
//! ```
//!

use crate::{ProviderRequest, ProviderResponse};
use lru::LruCache;
use std::{
    clone::Clone,
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
#[derive(Debug)]
pub struct LruProvider<'a, K, V>
where
    K: Eq + Hash,
{
    inner: Arc<Mutex<LruCache<K, V>>>,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> LruProvider<'a, (), ()> {
    /// Create a new LRU cache provider with the desired capacity
    pub fn new<K, V>(capacity: usize) -> Self
    where
        K: Eq + Hash,
    {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(capacity))),
            _phantom: PhantomData,
        }
    }
}

// Custom implementation of Clone as the Clone derive doesn't mark LruProvider
// as Clone if K or V is not clone.
impl<'a, K, V> Clone for LruProvider<'a, K, V>
where
    K: Eq + Hash,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
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
