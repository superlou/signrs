use std::collections::HashSet;
use std::hash::Hash;

pub fn iter_unique<T>(x: impl IntoIterator<Item = T>) -> impl IntoIterator<Item = T>
where
    T: Eq + Hash
{
    x.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>()
}