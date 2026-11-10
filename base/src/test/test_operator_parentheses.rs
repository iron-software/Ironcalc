#![allow(clippy::unwrap_used)]

// IronXL: two formulas that differ only by parentheses that the operator order needs have two
// formula texts, so each formula keeps its own tree and result, in each write order.

use crate::test::util::new_empty_model;

// Writes `first` to A1 and `second` to A2, in this order or in the reverse order.
fn assert_pair(first: (&str, &str), second: (&str, &str)) {
    for reverse in [false, true] {
        let mut model = new_empty_model();
        model._set("L1", "2");
        model._set("L2", "2");
        if reverse {
            model._set("A2", second.0);
            model._set("A1", first.0);
        } else {
            model._set("A1", first.0);
            model._set("A2", second.0);
        }
        model.evaluate();
        assert_eq!(
            model._get_text("A1"),
            first.1,
            "{} (reverse: {reverse})",
            first.0
        );
        assert_eq!(
            model._get_text("A2"),
            second.1,
            "{} (reverse: {reverse})",
            second.0
        );
    }
}

#[test]
fn minus_before_a_parenthesized_comparison() {
    assert_pair(("=-(L1>1)", "-1"), ("=-L2>1", "FALSE"));
    assert_pair(("=-($L$1>1)", "-1"), ("=-$L$1>1", "FALSE"));
    assert_pair(("=-(L1>1)", "-1"), ("=-L1>1", "FALSE"));
}

#[test]
fn each_operator_pair_gives_its_own_result() {
    assert_pair(("=-(L1&\"x\")", "#VALUE!"), ("=-L2&\"x\"", "-2x"));
    assert_pair(("=(L1>1)%", "0.01"), ("=L2>1%", "TRUE"));
    assert_pair(("=(L1&\"5\")%", "0.25"), ("=L2&\"5\"%", "20.05"));
    assert_pair(("=(L1>1)&\"x\"", "TRUEx"), ("=L2>(1&\"x\")", "FALSE"));
    assert_pair(("=\"x\"&(L1>1)", "xTRUE"), ("=\"x\"&L2>1", "TRUE"));
    assert_pair(("=1+(L1&\"1\")", "22"), ("=1+L2&\"1\"", "31"));
    assert_pair(("=(L1&\"1\")*2", "42"), ("=L2&(\"1\"*2)", "22"));
    assert_pair(("=2*(L1&\"1\")", "42"), ("=2*L2&\"1\"", "41"));
    assert_pair(("=(L1+1)%", "0.03"), ("=L2+1%", "2.01"));
    assert_pair(("=L1=(1>0)", "FALSE"), ("=(L2=1)>0", "TRUE"));
}

#[test]
fn formulas_that_round_trip_keep_their_results() {
    assert_pair(("=-(L1*3)", "-6"), ("=-L2*3", "-6"));
    assert_pair(("=-(L1^2)", "-4"), ("=-L2^2", "4"));
    assert_pair(("=L1-(1-1)", "2"), ("=L2-1-1", "0"));
    assert_pair(("=L1/(2*2)", "0.5"), ("=L2/2*2", "2"));
    assert_pair(("=L1^(2^2)", "16"), ("=L2^2^2", "16"));
    assert_pair(("=-(-L1)", "2"), ("=--L2", "2"));
}
