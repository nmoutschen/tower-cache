/// # Request transformation trait
///
/// In many cases, it's not useful to cache based on the entire request payload,
/// or the request might not be directly usable with the desired cache provider.
/// This trait provide an abstraction over functions that transform a request
/// into a value that can be used by a caching provider.
///
/// ## Usage
///
/// In most case, you don't need to implement this trait directly. It is
/// automatically implemented for functions that take one argument and return
/// another.
///
/// ```rust
/// use tower_cache::Transform;
///
/// fn my_func(req: usize) -> usize {
///     req * 2
/// }
///
/// // Calling my_func using the Transform trait
/// assert_eq!((my_func).transform(2), 4);
/// ```
///
/// This is also implemented for `()` as a no-op transformer:
///
/// ```rust
/// use tower_cache::Transform;
///
/// assert_eq!(().transform(2), 2);
/// ```
///
pub trait Transform<R> {
    /// Output of the transformer
    type Output;

    /// Transform a key into a reference value for a cache provider.
    fn transform(&self, req: R) -> Self::Output;
}

impl<R> Transform<R> for () {
    type Output = R;

    fn transform(&self, req: R) -> Self::Output {
        req
    }
}

impl<F, R, O> Transform<R> for F
where
    F: Fn(R) -> O,
{
    type Output = F::Output;

    fn transform(&self, req: R) -> Self::Output {
        (self)(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit() {
        assert_eq!(().transform(2), 2);
    }

    #[test]
    fn test_closure() {
        assert_eq!((|v| v * 2).transform(2), 4);
    }

    #[test]
    fn test_function() {
        fn t(v: usize) -> usize {
            v * 2
        }

        assert_eq!(t.transform(2), 4);
    }
}
