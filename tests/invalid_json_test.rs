use jsode::prelude::*;

#[test]
fn n_array_1_true_without_comma() -> Result<()> {
    let mut json = JsonParser::new(include_str!("../resources/invalid/n_array_1_true_without_comma.json"));
    let out = json.parse()?;

    Ok(())
}