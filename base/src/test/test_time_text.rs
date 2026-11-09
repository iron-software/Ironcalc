#![allow(clippy::unwrap_used)]

// IronXL: TIMEVALUE, VALUE, the text-to-number cast, and TEXT read a time text with 1 to 3
// digits after the seconds, as Excel does.

use crate::test::util::new_empty_model;

fn results(formulas: &[&str]) -> Vec<String> {
    let mut model = new_empty_model();
    for (index, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", index + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(|row| model._get_text(&format!("A{row}")))
        .collect()
}

fn assert_results(cases: &[(&str, &str)]) {
    let formulas: Vec<&str> = cases.iter().map(|(formula, _)| *formula).collect();
    let actual = results(&formulas);
    for ((formula, expected), actual) in cases.iter().zip(actual) {
        assert_eq!(actual, *expected, "{formula}");
    }
}

#[test]
fn timevalue_reads_digits_after_the_seconds() {
    assert_results(&[
        ("=TIMEVALUE(\"12:00:00.25\")", "0.500002894"),
        ("=TIMEVALUE(\"6:30:15.5 PM\")", "0.771012731"),
        ("=TIMEVALUE(\"9:05:07.125\")*86400", "32707.125"),
        ("=TIMEVALUE(\"9:05:07.125\")=0.3785546875", "TRUE"),
        ("=TIMEVALUE(\"12:00:00.25\")*86400", "43200.25"),
        ("=TIMEVALUE(\"6:30:15.5 PM\")*86400", "66615.5"),
        ("=TIMEVALUE(\"12:00:00.5am\")*86400", "0.5"),
        ("=TIMEVALUE(\" 0:00:00.001 \")*86400000", "1"),
        ("=TIMEVALUE(\"12:00:00\")", "0.5"),
        ("=TIMEVALUE(\"24:00:00\")", "0"),
        ("=TIMEVALUE(\"24:30:00\")", "0.020833333"),
        ("=TIMEVALUE(\"23:59:60\")", "0"),
    ]);
}

#[test]
fn value_and_the_cast_read_a_time_text() {
    assert_results(&[
        ("=VALUE(\"12:00:00.25\")", "0.500002894"),
        ("=\"12:00:00.25\"+0", "0.500002894"),
        ("=VALUE(\"12:00:00.25\")=TIMEVALUE(\"12:00:00.25\")", "TRUE"),
        ("=VALUE(\"12:00:00\")", "0.5"),
        ("=VALUE(\"12:00\")", "0.5"),
        ("=VALUE(\"6:30:15.5 PM\")*86400", "66615.5"),
        ("=VALUE(\"12:30 AM\")*1440", "30"),
        ("=VALUE(\"12:30 pm\")*1440", "750"),
        ("=VALUE(\"1:05:07.12PM\")*86400", "47107.12"),
        ("=ABS(\"6:00\")", "0.25"),
        ("=SUM(\"6:00\",\"12:00\")", "0.75"),
    ]);
}

#[test]
fn text_formats_a_text_that_reads_as_a_number() {
    assert_results(&[
        ("=TEXT(\"12:00:00.25\",\"h:mm:ss.000\")", "12:00:00.250"),
        ("=TEXT(\"1\",\"0.00\")", "1.00"),
        ("=TEXT(\"50%\",\"0.0\")", "0.5"),
        ("=TEXT(\"abc\",\"0\")", "abc"),
        ("=TEXT(\"\",\"0\")", ""),
        ("=TEXT(\"inf\",\"0\")", "inf"),
        ("=TEXT(\"1\",\"[h\")", "#VALUE!"),
        ("=TEXT(\"-1\",\"h:mm\")", "#VALUE!"),
        ("=INDEX(TEXT({\"1\",\"abc\"},\"0.00\"),1,1)", "1.00"),
        ("=INDEX(TEXT({\"1\",\"abc\"},\"0.00\"),1,2)", "abc"),
        (
            "=INDEX(TEXT({\"12:00:00.25\",\"inf\"},\"h:mm:ss.000\"),1,1)",
            "12:00:00.250",
        ),
        (
            "=INDEX(TEXT({\"12:00:00.25\",\"inf\"},\"h:mm:ss.000\"),1,2)",
            "inf",
        ),
        ("=INDEX(TEXT({\"-1\",1},\"h:mm\"),1,1)", "#VALUE!"),
    ]);
}

#[test]
fn invalid_time_text_keeps_the_value_error() {
    assert_results(&[
        ("=TIMEVALUE(\"12:00:00.2.5\")", "#VALUE!"),
        ("=TIMEVALUE(\"12:00:00.1234\")", "#VALUE!"),
        ("=TIMEVALUE(\"12:00:00.\")", "#VALUE!"),
        ("=TIMEVALUE(\"12:00:00.x\")", "#VALUE!"),
        ("=VALUE(\"12:00:00.2.5\")", "#VALUE!"),
        ("=VALUE(\"12:00:00.x\")", "#VALUE!"),
        ("=VALUE(\"12:00:00.\")", "#VALUE!"),
        ("=VALUE(\"12:00:00.1234\")", "#VALUE!"),
        ("=VALUE(\"12:00.5\")", "#VALUE!"),
        ("=VALUE(\"24:00:00\")", "#VALUE!"),
        ("=VALUE(\"12:60\")", "#VALUE!"),
        ("=VALUE(\"12:00:60\")", "#VALUE!"),
        ("=VALUE(\"123:00\")", "#VALUE!"),
        ("=VALUE(\"1:2:3:4\")", "#VALUE!"),
        ("=VALUE(\":30\")", "#VALUE!"),
        ("=VALUE(\"13:00 PM\")", "#VALUE!"),
        ("=VALUE(\"0:30 AM\")", "#VALUE!"),
        ("=VALUE(\"12:00 XM\")", "#VALUE!"),
        ("=\"abc\"+0", "#VALUE!"),
    ]);
}

#[test]
fn other_text_keeps_its_number() {
    assert_results(&[
        ("=VALUE(\"1/2/2020\")", "43832"),
        ("=VALUE(\"50%\")", "0.5"),
        ("=VALUE(\"$1,000\")", "1000"),
        ("=VALUE(\"1e3\")", "1000"),
        ("=VALUE(\"2020-01-02\")", "43832"),
    ]);
}
