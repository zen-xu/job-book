/// https://stackoverflow.com/questions/27582739/how-do-i-create-a-hashmap-literal
#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {
        std::iter::Iterator::collect(std::array::IntoIter::new([$(($k, $v),)*]))
    };
    // set-like
    ($($v:expr),* $(,)?) => {
        std::iter::Iterator::collect(std::array::IntoIter::new([$($v,)*]))
    };
}

#[test]
fn test_map() {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
    use std::iter::FromIterator;

    let l: Vec<_> = collection![1, 2, 3];
    let r = vec![1, 2, 3];
    assert_eq!(l, r);

    let l: BTreeSet<_> = collection! { 1, 2, 3 };
    let r: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    assert_eq!(l, r);

    let l: HashSet<_> = collection! { 1, 2, 3 };
    let r: HashSet<_> = [1, 2, 3].iter().cloned().collect();
    assert_eq!(l, r);

    let l: BTreeMap<_, _> = collection! { 1 => 2, 2 => 3 };
    let r: BTreeMap<_, _> = BTreeMap::<i32, i32>::from_iter((1..=2).map(|n| (n, n + 1)));
    assert_eq!(l, r);

    let l: HashMap<_, _> = collection! { 1 => 2, 2 => 3 };
    let r: HashMap<_, _> = HashMap::<i32, i32>::from_iter((1..=2).map(|n| (n, n + 1)));
    assert_eq!(l, r);
}
