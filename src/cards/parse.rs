named!(pub card_result(&str) -> Option<[&str; 3]>,
    alt_complete!(card_result_err | card_result_ok));
named!(card_result_ok(&str) -> Option<[&str; 3]>,
    map!(card_stripes, Some));
named!(card_result_err(&str) -> Option<[&str; 3]>,
    map!(tag_s!("%E?\n"), |_| None));

named!(card_stripes(&str) -> [&str; 3], do_parse!(
    tag_s!("%") >>
    tk1: take_until_s!("?") >>
    tag_s!("?;") >>
    tk2: take_until_s!("?") >>
    tag_s!("?+") >>
    tk3: take_until_s!("?") >>
    tag_s!("?\n") >>
    ( [tk1, tk2, tk3] )));
