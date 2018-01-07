named!(pub card_result(&str) -> Option<Vec<&str>>,
    alt_complete!(card_result_err | card_result_ok));
named!(card_result_ok(&str) -> Option<Vec<&str>>,
    map!(card_stripes, Some));
named!(card_result_err(&str) -> Option<Vec<&str>>,
    map!(tag_s!("%E?\n"), |_| None));

named!(card_stripes(&str) -> Vec<&str>, do_parse!(
    tag_s!("%") >>
    init: many0!(map!(pair!(take_until_s!("^"), tag_s!("^")), |(s, _)| s)) >>
    last: take_until_s!("?") >>
    tag_s!("?\n") >>
    ({ let mut init = init; init.push(last); init })));
