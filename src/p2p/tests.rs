use p2p::Message;

quickcheck! {
    fn serialize_parse_is_identity(msg: Message) -> () {
        let mut buf = Vec::new();
        msg.write_to(&mut buf).expect("Failed to serialize");
        let msg2 = Message::parse_from(&buf).expect("Failed to parse");
        assert_eq!(msg, msg2);
    }
}
