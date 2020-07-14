use core::{
    borrow::{Borrow, BorrowMut},
    fmt,
    iter::FromIterator,
    marker::PhantomData,
    ops,
    slice::{self /*SliceIndex*/},
};

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::{
    boxed::Box,
    vec::{self, Drain, Splice, Vec},
};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};

use crate::{Index, TiEnumerated, TiRangeBounds, TiSlice};

/// A contiguous growable array type
/// that only accepts keys of the type `K`.
///
/// `TiVec<K, V>` is a wrapper around Rust primitive type [`std::vec::Vec`].
/// The struct mirrors the stable API of Rust [`std::vec::Vec`]
/// and forwards to it as much as possible.
///
/// `TiVec<K, V>` uses `K` instead of `usize` for element indices and
/// require the index to implement [`Index`] trait.
/// If default feature `impl-index-from` is not disabled, this trait is automatically implemented
/// when [`From<usize>`] and [`Into<usize>`] are implemented.
/// And their implementation can be easily done
/// with [`derive_more`] crate and `#[derive(From, Into)]`.
///
/// `TiVec<K, V>` can be converted to [`std::vec::Vec<V>`] and back
/// using [`From`] and [`Into`].
///
#[cfg_attr(
    feature = "impl-index-from",
    doc = r#"
    # Example

    ```
    use typed_index_collections::TiVec;
    use derive_more::{From, Into};
    #[derive(From, Into)]
    struct FooId(usize);
    let mut foos: TiVec<FooId, usize> = std::vec![10, 11, 13].into();
    foos.insert(FooId(2), 12);
    assert_eq!(foos[FooId(2)], 12);
    ```

"#
)]
/// [`Index`]: trait.Index.html
/// [`std::vec::Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`std::vec::Vec<V>`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
/// [`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
/// [`From<usize>`]: https://doc.rust-lang.org/std/convert/trait.From.html
/// [`Into<usize>`]: https://doc.rust-lang.org/std/convert/trait.Into.html
/// [`derive_more`]: https://crates.io/crates/derive_more
#[derive(Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TiVec<K, V> {
    /// Raw slice property
    pub raw: Vec<V>,

    /// Tied slice index type
    ///
    /// `fn(T) -> T` is *[PhantomData pattern][phantomdata patterns]*
    /// used to relax auto trait implementations bounds for
    /// [`Send`], [`Sync`], [`Unpin`], [`UnwindSafe`] and [`RefUnwindSafe`].
    ///
    /// [phantomdata patterns]: https://doc.rust-lang.org/nomicon/phantom-data.html#table-of-phantomdata-patterns
    /// [`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html
    /// [`Sync`]: https://doc.rust-lang.org/core/marker/trait.Sync.html
    /// [`Unpin`]: https://doc.rust-lang.org/core/marker/trait.Unpin.html
    /// [`UnwindSafe`]: https://doc.rust-lang.org/core/std/panic/trait.UnwindSafe.html
    /// [`RefUnwindSafe`]: https://doc.rust-lang.org/core/std/panic/trait.RefUnwindSafe.html
    _marker: PhantomData<fn(K) -> K>,
}

impl<K, V> TiVec<K, V> {
    /// Constructs a new, empty `TiVec<K, V>`.
    ///
    /// See [`Vec::new`].
    ///
    /// [`Vec::new`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.new
    #[inline]
    pub fn new() -> Self {
        Self {
            raw: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Constructs a new, empty `TiVec<K, V>` with the specified capacity.
    ///
    /// See [`Vec::with_capacity`].
    ///
    /// [`Vec::with_capacity`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.with_capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    /// Decomposes a `TiVec<K, V>` into its raw components.
    ///
    /// See [`Vec::into_raw_parts`].
    ///
    /// [`Vec::into_raw_parts`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_raw_parts
    #[cfg(feature = "unstable_vec_into_raw_parts")]
    pub fn into_raw_parts(self) -> (*mut V, usize, usize) {
        self.raw.into_raw_parts()
    }

    /// Creates a `TiVec<K, V>` directly from the raw components of another vector.
    ///
    /// See [`Vec::from_raw_parts`].
    ///
    /// [`Vec::from_raw_parts`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(ptr: *mut V, length: usize, capacity: usize) -> Self {
        Self {
            raw: Vec::from_raw_parts(ptr, length, capacity),
            _marker: PhantomData,
        }
    }

    /// Returns the number of elements the vector can hold without
    /// reallocating.
    ///
    /// See [`Vec::capacity`].
    ///
    /// [`Vec::capacity`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `TiVec<K, V>`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// See [`Vec::reserve`].
    ///
    /// [`Vec::reserve`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reserve
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional)
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given `TiVec<K, V>`. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// See [`Vec::reserve_exact`].
    ///
    /// [`Vec::reserve_exact`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reserve_exact
    pub fn reserve_exact(&mut self, additional: usize) {
        self.raw.reserve_exact(additional)
    }

    /// Tries to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `TiVec<K, V>`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// See [`Vec::try_reserve`].
    ///
    /// [`Vec::try_reserve`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.try_reserve
    #[cfg(feature = "unstable_try_reserve")]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve(additional)
    }

    /// Tries to reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given `TiVec<K, V>`. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// See [`Vec::try_reserve_exact`].
    ///
    /// [`Vec::try_reserve_exact`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.try_reserve_exact
    #[cfg(feature = "unstable_try_reserve")]
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.raw.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of the vector as much as possible.
    ///
    /// See [`Vec::shrink_to_fit`].
    ///
    /// [`Vec::shrink_to_fit`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.shrink_to_fit
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit()
    }

    /// Shrinks the capacity of the vector with a lower bound.
    ///
    /// See [`Vec::shrink_to`].
    ///
    /// [`Vec::shrink_to`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.shrink_to
    #[cfg(feature = "unstable_shrink_to")]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.raw.shrink_to()
    }

    /// Converts the vector into [`Box<TiSlice<K, V>>`][`Box`].
    ///
    /// See [`Vec::into_boxed_slice`].
    ///
    /// [`Vec::into_boxed_slice`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_boxed_slice
    /// [`Box`]: ../../std/boxed/struct.Box.html
    pub fn into_boxed_slice(self) -> Box<TiSlice<K, V>> {
        self.raw.into_boxed_slice().into()
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// See [`Vec::truncate`].
    ///
    /// [`Vec::truncate`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.truncate
    pub fn truncate(&mut self, len: usize) {
        self.raw.truncate(len)
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// See [`Vec::as_slice`].
    ///
    /// [`Vec::as_slice`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.as_slice
    #[inline]
    pub fn as_slice(&self) -> &TiSlice<K, V> {
        self.raw.as_slice().into()
    }

    /// Extracts a mutable slice of the entire vector.
    ///
    /// See [`Vec::as_mut_slice`].
    ///
    /// [`Vec::as_mut_slice`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.as_mut_slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut TiSlice<K, V> {
        self.raw.as_mut_slice().into()
    }

    /// Returns a raw pointer to the vector's buffer.
    ///
    /// See [`Vec::as_ptr`].
    ///
    /// [`Vec::as_ptr`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.as_ptr
    #[inline]
    pub fn as_ptr(&self) -> *const V {
        self.raw.as_ptr()
    }

    /// Returns an unsafe mutable pointer to the vector's buffer.
    ///
    /// See [`Vec::as_mut_ptr`].
    ///
    /// [`Vec::as_mut_ptr`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.as_mut_ptr
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut V {
        self.raw.as_mut_ptr()
    }

    /// Forces the length of the vector to `new_len`.
    ///
    /// See [`Vec::set_len`].
    ///
    /// [`Vec::set_len`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.set_len
    #[allow(clippy::missing_safety_doc)]
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.raw.set_len(new_len)
    }

    /// Removes an element from the vector and returns it.
    ///
    /// See [`Vec::swap_remove`].
    ///
    /// [`Vec::swap_remove`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.swap_remove
    #[inline]
    pub fn swap_remove(&mut self, index: K) -> V
    where
        usize: From<K>,
    {
        self.raw.swap_remove(index.into())
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// See [`Vec::insert`].
    ///
    /// [`Vec::insert`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.insert
    pub fn insert(&mut self, index: K, element: V)
    where
        usize: From<K>,
    {
        self.raw.insert(index.into(), element)
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// See [`Vec::remove`].
    ///
    /// [`Vec::remove`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.remove
    pub fn remove(&mut self, index: K) -> V
    where
        usize: From<K>,
    {
        self.raw.remove(index.into())
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// See [`Vec::retain`].
    ///
    /// [`Vec::retain`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&V) -> bool,
    {
        self.raw.retain(f)
    }

    /// Removes all but the first of consecutive elements in the vector that resolve to the same
    /// key.
    ///
    /// See [`Vec::dedup_by_key`].
    ///
    /// [`Vec::dedup_by_key`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.dedup_by_key
    #[inline]
    pub fn dedup_by_key<F, K2>(&mut self, key: F)
    where
        F: FnMut(&mut V) -> K2,
        K2: PartialEq,
    {
        self.raw.dedup_by_key(key)
    }

    /// Removes all but the first of consecutive elements in the vector satisfying a given equality
    /// relation.
    ///
    /// See [`Vec::dedup_by`].
    ///
    /// [`Vec::dedup_by`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.dedup_by
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut V, &mut V) -> bool,
    {
        self.raw.dedup_by(same_bucket)
    }

    /// Appends an element to the back of a collection.
    ///
    /// See [`Vec::push`].
    ///
    /// [`Vec::push`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push
    #[inline]
    pub fn push(&mut self, value: V) {
        self.raw.push(value)
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// See [`Vec::pop`].
    ///
    /// [`Vec::pop`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.pop
    #[inline]
    pub fn pop(&mut self) -> Option<V> {
        self.raw.pop()
    }

    /// Moves all the elements of `other` into `Self`, leaving `other` empty.
    ///
    /// See [`Vec::append`].
    ///
    /// [`Vec::append`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.append
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        self.raw.append(&mut other.raw)
    }

    /// Creates a draining iterator that removes the specified range in the vector
    /// and yields the removed items.
    ///
    /// See [`Vec::drain`].
    ///
    /// [`Vec::drain`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, V>
    where
        R: TiRangeBounds<K>,
        //R::Index: SliceIndex<[V], Output = [V]> + RangeBounds<usize>,
    {
        self.raw.drain(range.into_range())
    }

    /// Clears the vector, removing all values.
    ///
    /// See [`Vec::clear`].
    ///
    /// [`Vec::clear`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.clear
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear()
    }

    /// Returns the number of elements in the vector, also referred to
    /// as its 'length'.
    ///
    /// See [`Vec::len`].
    ///
    /// [`Vec::len`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.len
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// See [`Vec::is_empty`].
    ///
    /// [`Vec::is_empty`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.is_empty
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Splits the collection into two at the given index.
    ///
    /// See [`Vec::split_off`].
    ///
    /// [`Vec::split_off`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.split_off
    #[inline]
    #[must_use = "use `.truncate()` if you don't need the other half"]
    pub fn split_off(&mut self, at: K) -> Self
    where
        usize: From<K>,
    {
        self.raw.split_off(at.into()).into()
    }

    /// Resizes the `TiVec` in-place so that `len` is equal to `new_len`.
    ///
    /// See [`Vec::resize_with`].
    ///
    /// [`Vec::resize_with`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.resize_with
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> V,
    {
        self.raw.resize_with(new_len, f)
    }

    /// Resizes the `TiVec` in-place so that `len` is equal to `new_len`.
    ///
    /// See [`Vec::resize`].
    ///
    /// [`Vec::resize`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.resize
    pub fn resize(&mut self, new_len: usize, value: V)
    where
        V: Clone,
    {
        self.raw.resize(new_len, value)
    }

    /// Clones and appends all elements in a slice to the `TiVec`.
    ///
    /// See [`Vec::extend_from_slice`].
    ///
    /// [`Vec::extend_from_slice`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.extend_from_slice
    pub fn extend_from_slice(&mut self, other: &TiSlice<K, V>)
    where
        V: Clone,
    {
        self.raw.extend_from_slice(&other.raw)
    }

    /// [`PartialEq`] trait implementation.
    ///
    /// See [`Vec::dedup`].
    ///
    /// [`Vec::dedup`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.dedup
    #[inline]
    pub fn dedup(&mut self)
    where
        V: PartialEq,
    {
        self.raw.dedup()
    }

    /// Creates a splicing iterator that replaces the specified range in the vector
    /// with the given `replace_with` iterator and yields the removed items.
    /// `replace_with` does not need to be the same length as `range`.
    ///
    /// See [`Vec::splice`].
    ///
    /// [`Vec::splice`]: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.splice
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter>
    where
        R: TiRangeBounds<K>,
        //R::Index: SliceIndex<[V], Output = [V]> + RangeBounds<usize>,
        I: IntoIterator<Item = V>,
    {
        self.raw.splice(range.into_range(), replace_with)
    }

    /// Converts the vector into iterator over all key-value pairs.
    /// with `K` used for iteration indices.
    ///
    /// # Example
    ///
    /// ```
    /// # use derive_more::{From, Into};
    /// # use typed_index_collections::TiVec;
    /// #[derive(Eq, Debug, From, Into, PartialEq)]
    /// pub struct Id(usize);
    /// let vec: TiVec<Id, usize> = vec![1, 2, 4].into();
    /// let mut iterator = vec.into_iter_enumerated();
    /// assert_eq!(iterator.next(), Some((Id(0), 1)));
    /// assert_eq!(iterator.next(), Some((Id(1), 2)));
    /// assert_eq!(iterator.next(), Some((Id(2), 4)));
    /// assert_eq!(iterator.next(), None);
    /// ```
    #[inline(always)]
    pub fn into_iter_enumerated(self) -> TiEnumerated<vec::IntoIter<V>, K, V>
    where
        K: From<usize>,
    {
        self.raw
            .into_iter()
            .enumerate()
            .map(|(key, value)| (key.into(), value))
    }
}

impl<K, V> fmt::Debug for TiVec<K, V>
where
    K: fmt::Debug + Index,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter_enumerated()).finish()
    }
}

impl<K, V> AsRef<TiVec<K, V>> for TiVec<K, V> {
    fn as_ref(&self) -> &TiVec<K, V> {
        self
    }
}

impl<K, V> AsMut<TiVec<K, V>> for TiVec<K, V> {
    fn as_mut(&mut self) -> &mut TiVec<K, V> {
        self
    }
}

impl<K, V> AsRef<TiSlice<K, V>> for TiVec<K, V> {
    fn as_ref(&self) -> &TiSlice<K, V> {
        self
    }
}

impl<K, V> AsMut<TiSlice<K, V>> for TiVec<K, V> {
    fn as_mut(&mut self) -> &mut TiSlice<K, V> {
        self
    }
}

impl<K, V> Borrow<TiSlice<K, V>> for TiVec<K, V> {
    fn borrow(&self) -> &TiSlice<K, V> {
        self.as_slice()
    }
}

impl<K, V> BorrowMut<TiSlice<K, V>> for TiVec<K, V> {
    fn borrow_mut(&mut self) -> &mut TiSlice<K, V> {
        self.as_mut_slice()
    }
}

impl<K, V> ops::Deref for TiVec<K, V> {
    type Target = TiSlice<K, V>;

    fn deref(&self) -> &TiSlice<K, V> {
        Self::Target::from_ref(&self.raw)
    }
}

impl<K, V> ops::DerefMut for TiVec<K, V> {
    fn deref_mut(&mut self) -> &mut TiSlice<K, V> {
        Self::Target::from_mut(&mut self.raw)
    }
}

impl<K, V> FromIterator<V> for TiVec<K, V> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self {
            raw: Vec::from_iter(iter),
            _marker: PhantomData,
        }
    }
}

impl<K, V> From<Vec<V>> for TiVec<K, V> {
    fn from(vec: Vec<V>) -> Self {
        Self {
            raw: vec,
            _marker: PhantomData,
        }
    }
}

impl<K, V> From<TiVec<K, V>> for Vec<V> {
    fn from(vec: TiVec<K, V>) -> Self {
        vec.raw
    }
}

impl<K, V> IntoIterator for TiVec<K, V> {
    type Item = V;
    type IntoIter = vec::IntoIter<V>;

    #[inline]
    fn into_iter(self) -> vec::IntoIter<V> {
        self.raw.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a TiVec<K, V> {
    type Item = &'a V;
    type IntoIter = slice::Iter<'a, V>;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, V> {
        self.raw.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut TiVec<K, V> {
    type Item = &'a mut V;
    type IntoIter = slice::IterMut<'a, V>;

    #[inline]
    fn into_iter(self) -> slice::IterMut<'a, V> {
        self.raw.iter_mut()
    }
}

#[cfg(feature = "serde")]
impl<K, V: Serialize> Serialize for TiVec<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, K, V: Deserialize<'de>> Deserialize<'de> for TiVec<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer).map(Into::into)
    }
}
