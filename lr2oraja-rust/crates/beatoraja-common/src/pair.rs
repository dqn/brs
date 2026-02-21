use std::cmp::Ordering;
use std::fmt;

/// Corresponds to Java's `bms.tool.util.Pair<K, V>`.
/// A generic pair of two values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair<K, V> {
    first: K,
    second: V,
}

impl<K, V> Pair<K, V> {
    /// Corresponds to Java's `Pair(K first, V second)` constructor.
    pub fn new(first: K, second: V) -> Self {
        Self { first, second }
    }

    /// Corresponds to Java's `of(K first, V second)` factory method.
    pub fn of(first: K, second: V) -> Self {
        Self::new(first, second)
    }

    /// Corresponds to Java's `getFirst()`.
    pub fn get_first(&self) -> &K {
        &self.first
    }

    /// Corresponds to Java's `getSecond()`.
    pub fn get_second(&self) -> &V {
        &self.second
    }

    /// Corresponds to Java's `apply(Function<K, U> onFirst, Function<V, P> onSecond)`.
    /// Maps both first and second using the provided functions.
    pub fn apply<U, P, F1, F2>(&self, on_first: F1, on_second: F2) -> Pair<U, P>
    where
        F1: FnOnce(&K) -> U,
        F2: FnOnce(&V) -> P,
    {
        Pair::of(on_first(&self.first), on_second(&self.second))
    }

    /// Corresponds to Java's `apply(Function<Pair<K, V>, Pair<U, P>> transfer)`.
    /// Transforms the pair using a single function.
    pub fn apply_pair<U, P, F>(&self, transfer: F) -> Pair<U, P>
    where
        F: FnOnce(&Pair<K, V>) -> Pair<U, P>,
    {
        transfer(self)
    }

    /// Corresponds to Java's `apply(BiFunction<K, V, U> transfer)`.
    /// Applies a bi-function to first and second, returning a single result.
    pub fn apply_bi<U, F>(&self, transfer: F) -> U
    where
        F: FnOnce(&K, &V) -> U,
    {
        transfer(&self.first, &self.second)
    }

    /// Corresponds to Java's `partiallyApplyFirst(Function<K, U> onFirst)`.
    pub fn partially_apply_first<U, F>(&self, on_first: F) -> U
    where
        F: FnOnce(&K) -> U,
    {
        on_first(&self.first)
    }

    /// Corresponds to Java's `applyFirstKeepSecond(Function<K, U> onFirst)`.
    pub fn apply_first_keep_second<U, F>(&self, on_first: F) -> Pair<U, V>
    where
        F: FnOnce(&K) -> U,
        V: Clone,
    {
        Pair::of(on_first(&self.first), self.second.clone())
    }

    /// Corresponds to Java's `partiallyApplySecond(Function<V, P> onSecond)`.
    pub fn partially_apply_second<P, F>(&self, on_second: F) -> P
    where
        F: FnOnce(&V) -> P,
    {
        on_second(&self.second)
    }

    /// Corresponds to Java's `applySecondKeepFirst(Function<V, P> onSecond)`.
    pub fn apply_second_keep_first<P, F>(&self, on_second: F) -> Pair<K, P>
    where
        F: FnOnce(&V) -> P,
        K: Clone,
    {
        Pair::of(self.first.clone(), on_second(&self.second))
    }

    /// Corresponds to Java's `consume(Consumer<K> onFirst, Consumer<V> onSecond)`.
    pub fn consume<F1, F2>(&self, on_first: F1, on_second: F2)
    where
        F1: FnOnce(&K),
        F2: FnOnce(&V),
    {
        on_first(&self.first);
        on_second(&self.second);
    }

    /// Corresponds to Java's `consume(Consumer<Pair<K, V>> consumer)`.
    pub fn consume_pair<F>(&self, consumer: F)
    where
        F: FnOnce(&Pair<K, V>),
    {
        consumer(self);
    }

    /// Corresponds to Java's `consume(BiConsumer<K, V> consumer)`.
    pub fn consume_bi<F>(&self, consumer: F)
    where
        F: FnOnce(&K, &V),
    {
        consumer(&self.first, &self.second);
    }

    /// Corresponds to Java's `partiallyConsumeFirst(Consumer<K> onFirst)`.
    pub fn partially_consume_first<F>(&self, on_first: F)
    where
        F: FnOnce(&K),
    {
        on_first(&self.first);
    }

    /// Corresponds to Java's `partiallyConsumeSecond(Consumer<V> onSecond)`.
    pub fn partially_consume_second<F>(&self, on_second: F)
    where
        F: FnOnce(&V),
    {
        on_second(&self.second);
    }

    /// Corresponds to Java's `predicate(Predicate<K> onFirst, Predicate<V> onSecond)`.
    pub fn predicate<F1, F2>(&self, on_first: F1, on_second: F2) -> bool
    where
        F1: FnOnce(&K) -> bool,
        F2: FnOnce(&V) -> bool,
    {
        on_first(&self.first) && on_second(&self.second)
    }

    /// Corresponds to Java's `predicate(BiPredicate<K, V> condition)`.
    pub fn predicate_bi<F>(&self, condition: F) -> bool
    where
        F: FnOnce(&K, &V) -> bool,
    {
        condition(&self.first, &self.second)
    }

    /// Corresponds to Java's `equalsOnFirst(Pair<T, U> rhs)`.
    pub fn equals_on_first<T, U>(&self, rhs: &Pair<T, U>) -> bool
    where
        K: PartialEq<T>,
    {
        self.first == rhs.first
    }

    /// Corresponds to Java's `projectFirst(Collection<Pair<T, U>> col)`.
    /// Extracts the first element from each pair in the collection.
    pub fn project_first(col: &[Pair<K, V>]) -> Vec<K>
    where
        K: Clone,
    {
        col.iter().map(|p| p.first.clone()).collect()
    }

    /// Corresponds to Java's `projectSecond(Collection<Pair<T, U>> col)`.
    /// Extracts the second element from each pair in the collection.
    pub fn project_second(col: &[Pair<K, V>]) -> Vec<V>
    where
        V: Clone,
    {
        col.iter().map(|p| p.second.clone()).collect()
    }

    /// Corresponds to Java's `into_first` / consuming version of getFirst.
    pub fn into_first(self) -> K {
        self.first
    }

    /// Corresponds to Java's `into_second` / consuming version of getSecond.
    pub fn into_second(self) -> V {
        self.second
    }

    /// Corresponds to Java's `into_tuple` / deconstruct into tuple.
    pub fn into_tuple(self) -> (K, V) {
        (self.first, self.second)
    }
}

/// Corresponds to Java's `shunt(Collection<T> col, Predicate<T> filter)`.
/// Splits a collection into two lists based on a predicate.
/// First list contains elements matching the predicate, second list contains the rest.
pub fn shunt<T, F>(col: impl IntoIterator<Item = T>, filter: F) -> Pair<Vec<T>, Vec<T>>
where
    F: Fn(&T) -> bool,
{
    let mut first = Vec::new();
    let mut second = Vec::new();
    for elem in col {
        if filter(&elem) {
            first.push(elem);
        } else {
            second.push(elem);
        }
    }
    Pair::of(first, second)
}

/// Corresponds to Java's `DEFAULT_COMPARATOR()`.
/// Returns a comparator for Pair<T, U> where both T and U are Ord.
/// Compares by first, then by second if first is equal.
pub fn default_comparator<T: Ord, U: Ord>(o1: &Pair<T, U>, o2: &Pair<T, U>) -> Ordering {
    if o1.first == o2.first {
        o1.second.cmp(&o2.second)
    } else {
        o1.first.cmp(&o2.first)
    }
}

impl<K: fmt::Display, V: fmt::Display> fmt::Display for Pair<K, V> {
    /// Corresponds to Java's `toString()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pair(first={}, second={})", self.first, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_of_and_getters() {
        let p = Pair::of(1, "hello");
        assert_eq!(*p.get_first(), 1);
        assert_eq!(*p.get_second(), "hello");
    }

    #[test]
    fn test_to_string() {
        let p = Pair::of(42, "world");
        assert_eq!(format!("{}", p), "Pair(first=42, second=world)");
    }

    #[test]
    fn test_apply_both() {
        let p = Pair::of(10, 20);
        let result = p.apply(|k| k * 2, |v| v + 5);
        assert_eq!(*result.get_first(), 20);
        assert_eq!(*result.get_second(), 25);
    }

    #[test]
    fn test_apply_pair() {
        let p = Pair::of(1, 2);
        let result = p.apply_pair(|pair| Pair::of(pair.get_first() + 10, pair.get_second() + 20));
        assert_eq!(*result.get_first(), 11);
        assert_eq!(*result.get_second(), 22);
    }

    #[test]
    fn test_apply_bi() {
        let p = Pair::of(3, 4);
        let result = p.apply_bi(|a, b| a + b);
        assert_eq!(result, 7);
    }

    #[test]
    fn test_partially_apply_first() {
        let p = Pair::of(5, 10);
        let result = p.partially_apply_first(|k| k * 3);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_apply_first_keep_second() {
        let p = Pair::of(5, 10);
        let result = p.apply_first_keep_second(|k| k.to_string());
        assert_eq!(*result.get_first(), "5");
        assert_eq!(*result.get_second(), 10);
    }

    #[test]
    fn test_partially_apply_second() {
        let p = Pair::of(5, 10);
        let result = p.partially_apply_second(|v| v * 2);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_apply_second_keep_first() {
        let p = Pair::of(5, 10);
        let result = p.apply_second_keep_first(|v| v.to_string());
        assert_eq!(*result.get_first(), 5);
        assert_eq!(*result.get_second(), "10");
    }

    #[test]
    fn test_consume() {
        let p = Pair::of(1, 2);
        let mut first_val = 0;
        let mut second_val = 0;
        p.consume(|k| first_val = *k, |v| second_val = *v);
        assert_eq!(first_val, 1);
        assert_eq!(second_val, 2);
    }

    #[test]
    fn test_consume_pair() {
        let p = Pair::of(1, 2);
        let mut sum = 0;
        p.consume_pair(|pair| sum = pair.get_first() + pair.get_second());
        assert_eq!(sum, 3);
    }

    #[test]
    fn test_consume_bi() {
        let p = Pair::of(3, 4);
        let mut product = 0;
        p.consume_bi(|a, b| product = a * b);
        assert_eq!(product, 12);
    }

    #[test]
    fn test_partially_consume_first() {
        let p = Pair::of(42, "unused");
        let mut val = 0;
        p.partially_consume_first(|k| val = *k);
        assert_eq!(val, 42);
    }

    #[test]
    fn test_partially_consume_second() {
        let p = Pair::of("unused", 99);
        let mut val = 0;
        p.partially_consume_second(|v| val = *v);
        assert_eq!(val, 99);
    }

    #[test]
    fn test_predicate_both_true() {
        let p = Pair::of(10, 20);
        assert!(p.predicate(|k| *k > 5, |v| *v > 15));
    }

    #[test]
    fn test_predicate_first_false() {
        let p = Pair::of(1, 20);
        assert!(!p.predicate(|k| *k > 5, |v| *v > 15));
    }

    #[test]
    fn test_predicate_bi() {
        let p = Pair::of(3, 4);
        assert!(p.predicate_bi(|a, b| a + b == 7));
        assert!(!p.predicate_bi(|a, b| a + b == 8));
    }

    #[test]
    fn test_default_comparator() {
        let p1 = Pair::of(1, 2);
        let p2 = Pair::of(1, 3);
        let p3 = Pair::of(2, 1);

        assert_eq!(default_comparator(&p1, &p2), Ordering::Less);
        assert_eq!(default_comparator(&p2, &p1), Ordering::Greater);
        assert_eq!(default_comparator(&p1, &p3), Ordering::Less);
        assert_eq!(default_comparator(&p1, &p1), Ordering::Equal);
    }

    #[test]
    fn test_equals_on_first() {
        let p1 = Pair::of(1, "a");
        let p2 = Pair::of(1, "b");
        let p3 = Pair::of(2, "a");

        assert!(p1.equals_on_first(&p2));
        assert!(!p1.equals_on_first(&p3));
    }

    #[test]
    fn test_project_first() {
        let pairs = vec![Pair::of(1, "a"), Pair::of(2, "b"), Pair::of(3, "c")];
        assert_eq!(Pair::project_first(&pairs), vec![1, 2, 3]);
    }

    #[test]
    fn test_project_second() {
        let pairs = vec![Pair::of(1, "a"), Pair::of(2, "b"), Pair::of(3, "c")];
        assert_eq!(Pair::project_second(&pairs), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_shunt() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let result = shunt(items, |x| x % 2 == 0);
        assert_eq!(*result.get_first(), vec![2, 4, 6]);
        assert_eq!(*result.get_second(), vec![1, 3, 5]);
    }

    #[test]
    fn test_shunt_empty() {
        let items: Vec<i32> = vec![];
        let result = shunt(items, |x| x % 2 == 0);
        assert!(result.get_first().is_empty());
        assert!(result.get_second().is_empty());
    }

    #[test]
    fn test_into_tuple() {
        let p = Pair::of(1, "hello");
        let (a, b) = p.into_tuple();
        assert_eq!(a, 1);
        assert_eq!(b, "hello");
    }

    #[test]
    fn test_into_first_second() {
        let p1 = Pair::of(42, "x");
        assert_eq!(p1.into_first(), 42);

        let p2 = Pair::of(42, "x");
        assert_eq!(p2.into_second(), "x");
    }
}
