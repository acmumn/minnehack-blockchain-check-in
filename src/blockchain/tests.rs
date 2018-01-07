use blockchain::{now, Block, Chain, ZERO_HASH};
use util::str_to_arrayvec;

fn example_chain() -> Chain {
    let mut chain = Chain::new();
    chain.mine_at(1000, str_to_arrayvec("foo").unwrap());
    chain.mine_at(2500, str_to_arrayvec("bar").unwrap());
    chain
}

fn example_chain_2() -> Chain {
    let mut chain = Chain::new();
    chain.mine_at(1000, str_to_arrayvec("foo").unwrap());
    chain.mine_at(2000, str_to_arrayvec("baz").unwrap());
    chain
}

#[test]
fn combine() {
    let combined_1 = example_chain().combine(example_chain_2());
    let combined_2 = example_chain_2().combine(example_chain());

    let mut expected = Chain::new();
    expected.mine_at(1000, str_to_arrayvec("foo").unwrap());
    expected.mine_at(2000, str_to_arrayvec("baz").unwrap());
    expected.mine_at(now(), str_to_arrayvec("bar").unwrap());

    assert_eq!(combined_1, expected);
    assert_eq!(combined_2, expected);

    assert!(combined_1.is_valid());
    assert!(combined_2.is_valid());
    assert!(expected.is_valid());
}

#[test]
fn iter() {
    let mut expected = vec![
        Block::new(
            0,
            ZERO_HASH,
            1515140055,
            str_to_arrayvec("Hello, world!").unwrap(),
        ),
    ];
    let mut next = expected[0].create_at(1000, str_to_arrayvec("foo").unwrap());
    expected.push(next);
    next = expected[1].create_at(2500, str_to_arrayvec("bar").unwrap());
    expected.push(next);

    for (i, block) in example_chain().into_iter().enumerate() {
        assert_eq!(block, &expected[i]);
    }
}

#[test]
fn find_fork() {
    let l = example_chain();
    let r = example_chain_2();

    assert_eq!(l.find_fork(&r), Some(1));
    assert_eq!(r.find_fork(&l), Some(1));

    assert_eq!(l.find_fork(&l), None);
    assert_eq!(r.find_fork(&r), None);
}

quickcheck! {
    fn serialize_parse_is_identity(block: Block) -> () {
        let mut buf = Vec::new();
        block.write_to(&mut buf).expect("Failed to serialize");
        let block2 = Block::parse_from(&buf).expect("Failed to parse");
        assert_eq!(block, block2);
    }
}
