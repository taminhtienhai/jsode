use jsode::prelude::*;

macro_rules! should_pass_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let mut json = JsonParser::new(include_str!(concat!("../resources/valid/", stringify!($name), ".json")));
            assert!(json.parse().inspect_err(|err| println!("{err:?}")).is_ok())
        }
    };
}

should_pass_test!(y_structure_lonely_string);
should_pass_test!(y_number_real_capital_e);
should_pass_test!(y_number_real_capital_e_pos_exp);
should_pass_test!(y_number_real_capital_e_neg_exp);
should_pass_test!(y_structure_lonely_negative_real);