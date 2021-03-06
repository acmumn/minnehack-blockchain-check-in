//! Functions for the U Cards.

pub(crate) mod parse;
#[cfg(test)]
mod tests;

/// The result of reading a card.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CardParse<'a> {
    /// A successful read.
    Card(Vec<&'a str>),

    /// A read failed.
    BadRead,

    /// An invalid input was attempted to be parsed.
    BadParse,
}

/// Parse a card. Returns `Card` on a successful card read, `BadRead` on a bad
/// card read, and `BadParse` if the input was invalid.
pub fn parse_card(input: &str) -> CardParse {
    use nom::IResult;

    match parse::card_result(input) {
        IResult::Done("", Some(card)) => CardParse::Card(card),
        IResult::Done("", None) => CardParse::BadRead,
        _ => CardParse::BadParse,
    }
}
