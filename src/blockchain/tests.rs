use blockchain::{timestamp, Block, Chain, ZERO_DIGEST};

fn example_chain() -> Chain {
    let mut chain = Chain::new();
    chain.mine_at("foo".into(), 1000);
    chain.mine_at("bar".into(), 2500);
    chain
}

fn example_chain_2() -> Chain {
    let mut chain = Chain::new();
    chain.mine_at("foo".into(), 1000);
    chain.mine_at("baz".into(), 2000);
    chain
}

#[test]
fn combine() {
    let combined_1 = example_chain().combine(example_chain_2());
    let combined_2 = example_chain_2().combine(example_chain());

    let mut expected = Chain::new();
    expected.mine_at("foo".into(), 1000);
    expected.mine_at("baz".into(), 2000);
    expected.mine_at("bar".into(), timestamp());

    assert_eq!(combined_1, expected);
    assert_eq!(combined_2, expected);

    assert!(combined_1.is_valid());
    assert!(combined_2.is_valid());
    assert!(expected.is_valid());
}

#[test]
fn iter() {
    let mut expected = vec![
        Block {
            index: 0,
            prev_hash: ZERO_DIGEST,
            timestamp: 1515140055,
            data: "Hello, world!".into(),
            hash: ZERO_DIGEST,
        },
    ];
    expected[0].update_hash();
    let mut next = expected[0].create_at("foo".into(), 1000);
    expected.push(next);
    next = expected[1].create_at("bar".into(), 2500);
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
