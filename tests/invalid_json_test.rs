use jsode::prelude::*;

macro_rules! should_fail_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let mut json = JsonParser::new(include_str!(concat!("../resources/invalid/", stringify!($name), ".json")));
            assert!(json.parse().is_err())
        }
    };
}

// #[test]
// fn n_array_1_true_without_comma() {
//     let mut json = JsonParser::new(include_str!(concat!("../resources/invalid/", "n_array_1_true_without_comma", ".json")));
//     assert!(json.parse().is_err())
// }

// #[test]
// fn n_array_comma_and_number() {
//     let mut json = JsonParser::new(include_str!("../resources/invalid/n_array_double_comma.json"));
//     assert!(json.parse().is_err())
// }

// generate_test!(n_array_1_true_without_comma);
// generate_test!(n_array_comma_and_number);
should_fail_test!(n_array_newlines_unclosed);

should_fail_test!(n_array_unclosed_trailing_comma);
should_fail_test!(n_structure_close_unopened_array);
should_fail_test!(n_string_with_trailing_garbage);
should_fail_test!(n_string_single_string_no_double_quotes);