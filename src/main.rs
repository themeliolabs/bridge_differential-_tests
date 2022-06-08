use blake3;
use clap::Parser;
use ed25519_compact::{KeyPair, Seed, Noise};
use rand::Rng;
use rs_merkle::{MerkleTree, Hasher};
use themelio_structs::{
    Address,
    BlockHeight,
    CoinData,
    CoinID,
    Denom,
    Header,
    NetID,
    CoinValue,
    StakeDoc,
    Transaction,
    TxKind,
    TxHash,
};
use tmelcrypt::{ed25519_keygen, HashVal};

const DATA_BLOCK_HASH_KEY: &[u8; 13] = b"smt_datablock";
const NODE_HASH_KEY: &[u8; 8] = b"smt_node";

const ERR_STRING: &str = "0x4572726f7220696e204646492070726f6772616d2e";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "")]
    blake3: String,

    #[clap(long, default_value = "")]
    ed25519: String,

    #[clap(long, default_value = "")]
    decode_integer: String,

    #[clap(long, default_value_t = 0)]
    start: isize,

    #[clap(long, default_value_t = 0, allow_hyphen_values = true)]
    end: isize,

    #[clap(long, default_value = "")]
    integer_size: String,

    #[clap(long, default_value = "")]
    slice: String,

    #[clap(long, default_value = "")]
    extract_transactions_hash: String,

    #[clap(long, default_value = "")]
    extract_block_height: String,

    #[clap(long, default_value = "")]
    modifier: String,

    #[clap(long, default_value = "")]
    extract_value_and_recipient: String,

    #[clap(long, default_value = "")]
    recipient: String,

    #[clap(long, default_value = "")]
    build_tree: String,

    #[clap(long)]
    big_hash: bool,
}

#[derive(Clone)]
struct Blake3Algorithm {}

impl Hasher for Blake3Algorithm {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        *blake3::keyed_hash(blake3::hash(NODE_HASH_KEY).as_bytes(), data).as_bytes()
    }
}

fn random_header(modifier: u128) -> Header {
    if modifier == 0 {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u64::MIN),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u128::MIN),
            fee_multiplier: u128::MIN,
            dosc_speed: u128::MIN,
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else if modifier == u8::MAX.into() {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u8::MAX.into()),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u8::MAX.into()),
            fee_multiplier: u8::MAX.into(),
            dosc_speed: u8::MAX.into(),
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else if modifier == u16::MAX.into() {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u16::MAX.into()),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u16::MAX.into()),
            fee_multiplier: u16::MAX.into(),
            dosc_speed: u16::MAX.into(),
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else if modifier == u32::MAX.into() {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u32::MAX.into()),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u32::MAX.into()),
            fee_multiplier: u32::MAX.into(),
            dosc_speed: u32::MAX.into(),
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else if modifier == u64::MAX.into() {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u64::MAX),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u64::MAX.into()),
            fee_multiplier: u64::MAX.into(),
            dosc_speed: u64::MAX.into(),
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else if modifier == u128::MAX {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(u64::MAX),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(u128::MAX),
            fee_multiplier: u128::MAX,
            dosc_speed: u128::MAX,
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    } else {
        Header {
            network: NetID::Mainnet,
            previous: HashVal::random(),
            height: BlockHeight(rand::thread_rng().gen()),
            history_hash: HashVal::random(),
            coins_hash: HashVal::random(),
            transactions_hash: HashVal::random(),
            fee_pool: CoinValue(rand::thread_rng().gen()),
            fee_multiplier: rand::thread_rng().gen(),
            dosc_speed: rand::thread_rng().gen(),
            pools_hash: HashVal::random(),
            stakes_hash: HashVal::random(),
        }
    }
}

fn random_coin_id() -> CoinID {
    CoinID {
        txhash: TxHash(HashVal::random()),
        index: rand::thread_rng().gen(),
    }
}

fn random_coin_data() -> CoinData {
    let additional_data_size: u32 = rand::thread_rng().gen_range(0..32);
    let additional_data_range = 0..additional_data_size;
    let additional_data: Vec<u8> = additional_data_range
        .map(|_| {
            rand::thread_rng().gen::<u8>()
        })
        .collect();

    CoinData {
        covhash: Address(HashVal::random()),
        value: CoinValue(rand::thread_rng().gen()),
        denom: Denom::Mel,
        additional_data
    }
}

fn random_transaction() -> Transaction {
    let limit: u32 = 32;

    let num_inputs: u32 = rand::thread_rng().gen_range(1..limit);
    let inputs_range = 0..num_inputs;

    let inputs = inputs_range
        .into_iter()
        .map(|_| {
            random_coin_id()
        })
        .collect();

    let num_outputs: u32 = rand::thread_rng().gen_range(1..limit);
    let outputs_range = 0..num_outputs;

    let outputs = outputs_range
        .into_iter()
        .map(|_| {
            random_coin_data()
        })
        .collect();

    let num_covenants: u32 = rand::thread_rng().gen_range(1..limit);
    let convenants_range = 0..num_covenants;
    let covenants = convenants_range
        .into_iter()
        .map(|_| {
            let size = rand::thread_rng().gen_range(0..limit);
            let range = 0..size;
            let covenant = range
                .into_iter()
                .map(|_| {
                    rand::thread_rng().gen::<u8>()
                })
                .collect();

            covenant
        })
        .collect();

    let num_sigs: u32 = rand::thread_rng().gen_range(1..limit);
    let sigs_range = 0..num_sigs;
    let sigs = sigs_range
        .into_iter()
        .map(|_| {
            let size = rand::thread_rng().gen_range(0..limit);
            let range = 0..size;
            let sig = range
                .into_iter()
                .map(|_| {
                    rand::thread_rng().gen::<u8>()
                })
                .collect();

            sig
        })
        .collect();

    Transaction {
        kind: TxKind::Swap,
        inputs,
        outputs,
        fee: CoinValue(rand::thread_rng().gen()),
        covenants,
        data: (0..2).map(|_| { rand::thread_rng().gen::<u8>() }).collect(),
        sigs,
    }
}

fn random_stakedoc() -> StakeDoc {
    StakeDoc {
        pubkey: ed25519_keygen().0,
        e_start: rand::thread_rng().gen(),
        e_post_end: rand::thread_rng().gen(),
        syms_staked: CoinValue(rand::thread_rng().gen::<u128>()),
    }
}

fn create_datablocks(num: u32) -> Vec<StakeDoc> {
    let range = 0..num;

    range
        .into_iter()
        .map(|_| {
            random_stakedoc()
        })
        .collect::<Vec<StakeDoc>>()
}

fn as_leaves(datablocks: Vec<StakeDoc>) -> Vec<[u8; 32]> {
    datablocks
        .iter()
        .map(|datablock| {
            *blake3::keyed_hash(
                blake3::hash(DATA_BLOCK_HASH_KEY).as_bytes(),
                &stdcode::serialize(datablock).unwrap()
            ).as_bytes()
        })
        .collect()
}

fn blake3_differential(data: &[u8]) -> String {
    let hash = *blake3::keyed_hash(
        blake3::hash(NODE_HASH_KEY).as_bytes(),
        data
    ).as_bytes();

    hex::encode(hash)
}

fn ed25519_differential(data: &[u8]) -> String {
    let keypair = KeyPair::from_seed(Seed::default());

    let signature = keypair.sk.sign(data, Some(Noise::generate()));

    format!("{}{}", hex::encode(*keypair.pk), hex::encode(*signature))
}

fn decode_integer_differential(integer: u128) -> String {
    let encoded_integer = stdcode::serialize(&integer)
        .expect(ERR_STRING);

    hex::encode(encoded_integer)
}

fn integer_size_differential(integer: u128) -> String {
    let encoded_integer = stdcode::serialize(&integer)
        .expect(ERR_STRING);

    let encoded_integer_length = encoded_integer.len() as u128;

    format!("{:0>64x}{:0>64x}{:0>64x}{:0<64}", 0x40, encoded_integer_length, encoded_integer_length, hex::encode(encoded_integer))
}

fn slice_differential(data: &[u8], start: isize, end: isize) -> String {
    if start < end {
        let start = start as usize;
        let end = end as usize;

        hex::encode(&data[start..end])
    } else {
        let r_start = (end + 1) as usize;
        let r_end = (start + 1) as usize;
    
        let mut reverse_slice = data[r_start..r_end].to_vec();
        reverse_slice.reverse();

        hex::encode(reverse_slice)
    }
}

    fn extract_transactions_hash_differential(modifier: u128) -> String {
        let header = random_header(modifier);
            
        let mut serialized_header = stdcode::serialize(&header)
        .expect(ERR_STRING);

        let serialized_header_length = serialized_header.len();

        let padding_length = serialized_header_length % 64;

        serialized_header.resize(serialized_header_length + padding_length, 0);

        format!(
            "{:0>64x}{}{:0>64x}{:0<64}",
            0x40,
            hex::encode(header.transactions_hash),
            serialized_header_length,
            hex::encode(serialized_header)
        )
    }

    fn extract_block_height_differential(block_height:u64, modifier: u128) -> String {
        let mut header = random_header(modifier);
        header.height = BlockHeight(block_height);

        let serialized_header = stdcode::serialize(&header)
            .expect(ERR_STRING);

        hex::encode(serialized_header)
    }

    fn extract_value_and_recipient_differential(
        value: u128,
        recipient: String,
    ) -> String {
        let mut transaction = random_transaction();

        transaction.outputs[0].value = CoinValue(value);

        transaction.outputs[0].additional_data = hex::decode(recipient)
            .expect(ERR_STRING);
        
        let serialized_transaction = stdcode::serialize(&transaction)
            .expect(ERR_STRING);

        hex::encode(serialized_transaction)
    }

    fn build_tree_differential(num_leaves: u32) -> String {
        let datablocks = create_datablocks(num_leaves);

        let leaves = as_leaves(datablocks.clone());

        let tree = MerkleTree::<Blake3Algorithm>::from_leaves(&leaves);

        let serialized_datablocks: Vec<String> = datablocks
            .into_iter()
            .map(|datablock| {
                let mut serialized_datablock = stdcode::serialize(&datablock).unwrap();
                let serialized_datablock_len = serialized_datablock.len();
                let padding_length = serialized_datablock.len() % 64;

                serialized_datablock.resize(
                    serialized_datablock_len + padding_length,
                    0
                );

                format!("{:0>64}{}", serialized_datablock_len, hex::encode(serialized_datablock))
            })
            .collect();

        let mut concatenated_datablock = String::new();
        for i in 0..serialized_datablocks.len() {
            concatenated_datablock += &serialized_datablocks[i];
        }

        format!("{} {}", concatenated_datablock, tree.root_hex().unwrap())
    }

    fn big_hash() -> String {
        let mut stakedocs = vec![];
        let range = 0..800;

        for _ in range {
            stakedocs.append(&mut stdcode::serialize(&random_stakedoc()).unwrap())
        }

        let stakedocs_length = stakedocs.len();
        let padding_length = stakedocs_length % 64;
        stakedocs.resize(stakedocs_length + padding_length, 0);

        let stakedocs = hex::encode(stakedocs);

        let big_hash = *blake3::keyed_hash(
            blake3::hash(DATA_BLOCK_HASH_KEY).as_bytes(),
            stakedocs.as_bytes(),
        ).as_bytes();
        let big_hash = hex::encode(big_hash);

        format!("{:0>64x}{}{:0>64x}{}", 0x40, big_hash, stakedocs.len() / 2, stakedocs)
    }

fn main() {
    let args = Args::parse();

    if args.blake3.len() > 0 {
        let data = hex::decode(args.blake3.strip_prefix("0x").unwrap())
            .expect(ERR_STRING);

        print!("0x{}", blake3_differential(&data));
    } else if args.ed25519.len() > 0 {
        let data = hex::decode(args.ed25519.strip_prefix("0x").unwrap())
            .expect(ERR_STRING);

        let key_and_signature = ed25519_differential(&data);

        print!("0x{}", key_and_signature);
    } else if args.decode_integer.len() > 0 {
        let integer: u128 = args.decode_integer.parse()
            .expect(ERR_STRING);

        let encoded_integer = decode_integer_differential(integer);

        print!("0x{}", encoded_integer);
    } else if args.integer_size.len() > 0 {
        let integer: u128 = args.integer_size
            .parse()
            .expect(ERR_STRING);
    
        let abi_encoded_integer_and_size = integer_size_differential(integer);

        print!("0x{}", abi_encoded_integer_and_size);
    } else if args.slice.len() > 0 {
        let data = hex::decode(args.slice.strip_prefix("0x").unwrap())
            .expect(ERR_STRING);

        print!("0x{}", slice_differential(&data, args.start, args.end));
    } else if args.extract_transactions_hash.len() > 0 {
        let modifier: u128 = args.extract_transactions_hash
            .parse()
            .expect(ERR_STRING);

        let serialized_header_and_root = extract_transactions_hash_differential(modifier);

        print!("0x{}", serialized_header_and_root);
    } else if args.extract_block_height.len() > 0 {
        let block_height: u64 = args.extract_block_height
            .parse()
            .expect(ERR_STRING);
        
        let modifier: u128 = args.modifier
            .parse()
            .expect(ERR_STRING);

        let serialized_header = extract_block_height_differential(block_height, modifier);

        print!("0x{}", serialized_header);
    } else if args.extract_value_and_recipient.len() > 0 {
        let value: u128 = args.extract_value_and_recipient
            .parse()
            .expect(ERR_STRING);

        let recipient = args.recipient.strip_prefix("0x")
            .expect(ERR_STRING);

        let serialized_transaction = extract_value_and_recipient_differential(value, recipient.to_string());

        print!("0x{}", serialized_transaction);
    } else if args.build_tree.len() > 0 {
        let num_leaves: u32 = args.build_tree
            .parse()
            .expect(ERR_STRING);
        let num_leaves = num_leaves % 16;

        let datablocks_and_root = build_tree_differential(num_leaves);
        print!("{}", datablocks_and_root);
    } else if args.big_hash == true {
        print!("0x{}", big_hash());
    } else {
        print!("0x");
    }
}
