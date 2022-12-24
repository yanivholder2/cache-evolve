use super::*;

#[test]
fn get_without_put() {
    let dash: Dash<i64, i64>= Dash::new();
    let key: i64 = 0;

    assert_eq!(dash.get(&key), None);
}

#[test]
fn get_after_different_value_put() {
    let mut dash: Dash<i64, i64>= Dash::new();
    let key: i64 = 0;

    dash.put(key+1, key+1);

    assert_eq!(dash.contains(&key), false);
    assert_eq!(dash.get(&key), None);
}

#[test]
fn get_after_same_value_put() {
    let mut dash: Dash<i64, i64>= Dash::new();
    let key: i64 = 0;

    dash.put(key, key);

    assert_eq!(dash.contains(&key), true);
    assert_eq!(dash.get(&key), Some(&key));
}