use nom::IResult;

use cards::{parse_card, CardParse};
use cards::parse::card_result;

// TODO: Find a card I'm okay with having its stripe be public.
// Old library card? Middle-school ID?

#[test]
fn parse_err() {
    let data = "%E?\n";
    assert_eq!(card_result(data), IResult::Done("", None));
    assert_eq!(parse_card(data), CardParse::BadRead);
}
