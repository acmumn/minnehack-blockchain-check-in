use iron::{IronResult, Response};
use iron::headers::ContentType;
use iron::modifier::Modifier;
use iron::modifiers::Header;
use iron::typemap::Key;
use serde::Serialize;
use serde_json::to_string_pretty as json_to_string;

pub struct Client;

impl Key for Client {
    type Value = ::minnehack_check_in::Client;
}

pub fn json_response<M: Modifier<Response>, T: Serialize>(
    modifiers: M,
    value: T,
) -> IronResult<Response> {
    let s = itry!(json_to_string(&value));
    Ok(Response::with((modifiers, s, Header(ContentType::json()))))
}
