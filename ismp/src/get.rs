use crate::{
    consensus::StateMachineHeight,
    host::StateMachine,
    prelude::{String, Vec},
};
use codec::{Decode, Encode};
use primitive_types::{H160, U256};

/// The ISMP GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct Get {
    /// The source state machine of this request.
    pub source_chain: StateMachine,
    /// The destination state machine of this request.
    pub dest_chain: StateMachine,
    /// The nonce of this request on the source chain
    pub nonce: u64,
    /// Module Id of the sending module
    pub from: Vec<u8>,
    /// Storage keys that this request is interested in.
    pub keys: Vec<StorageKey>,
    /// Height at which to read the state machine.
    pub height: StateMachineHeight,
    /// Host timestamp at which this request expires in seconds
    pub timeout_timestamp: u64,
}

/// The Storage Kind for GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum StorageKey {
    /// This indicates that we are trying to get an Evm storage
    EVM(EvmStorage),
    /// This indicates that we are trying to get a Substrate storage
    Substrate(SubstrateType),
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct EvmStorage {
    /// The contract address which is always 20 bytes
    pub contract_address: H160,
    /// To access contract storage, each variables are stored in a structure called Slot
    /// which is basically an increasing numerical index of the contract storage variables.
    /// We need to know the slot index of a variable before proceeding to query from the State
    /// trie.
    pub slot: u64,
    /// Description for the value at the given slot in storage
    pub layout: ValueDescription,
    /// Number of bytes occupied by the value in storage
    /// For reference a uint256 value has a size of 32, a `struct S { uint256 c; uint256 d }`
    /// would have a size of 64
    /// Provides information for how many offsets the key requires
    pub value_size: u64,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum KeyType {
    /// An index of an array
    Index(U256),
    /// Key pointing to a  value in a map,
    /// Big endian byte representation of the key
    Key(Vec<u8>),
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum ValueDescription {
    /// Represents a slot that holds values other than a map
    Value,
    /// Path to an item in a solidity array or mapping
    /// If the first value in the path is `KeyType::Index` then the root element is an array else
    /// it is a mapping The number of values in the vector indicates the levels of nesting
    /// including the value type at each level of nesting To fetch a value described by
    /// `x[20][30][40]` where x is defined as `uint24[][][] x;` we would have
    /// `vec![KeyType::Index(20), KeyType::Index(30), KeyType::Index(40)]`
    /// To fetch a value described by `x[20][30][40]` where x is defined as `mapping(uint =>
    ///  mapping(uint => uint64[] ) x;` we would have `vec![KeyType::Key(20),
    /// KeyType::Key(30), KeyType::Index(40)]`
    Path(Vec<KeyType>),
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum SubstrateType {
    /// A Pallet
    Pallet(PalletStorageType),
    /// An Ink! smart contract
    Contract(InkContractStorage),
}

/// The Storage Type for Ink Get Request.
/// The storage API operates by storing and loading entries  from single storage cells,
/// where each storage cell is accessed under its own dedicated storage key.
/// Ink Storage keys and are always encoded with the SCALE codec.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub struct InkContractStorage {
    /// The unique child trie id for the contract
    /// Can be retrieved from the `ContractInfo` stored in the `ContractInfoOf` storage map of
    /// pallet contracts using the contract's address
    pub trie_id: Vec<u8>,
    /// Storage root key of the contract
    /// if None is specified the default storage root 0x00000000 would be used
    /// to retrieve the value for this key
    pub root_key: Option<[u8; 4]>,
    /// Key in the contract struct
    pub item_key: InkStorageType,
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum InkStorageType {
    /// Key describing a mapping in a contract
    Mapping {
        /// Scale encoded base key for mapping
        base_key: Vec<u8>,
        /// Scale encoded item key
        item_key: Vec<u8>,
    },
    /// Scale encoded key for any field of the contract struct as specified in the storage metadata
    /// or defined by the ManualKey
    Other(Vec<u8>),
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum PalletStorageType {
    /// Storage Value
    StorageValue {
        /// pallet name as specified in the runtime
        pallet_name: String,
        /// String name of the storage value in the pallet
        storage_name: String,
    },
    /// Storage Map
    StorageMap {
        /// pallet name as specified in the runtime
        pallet_name: String,
        /// String name of the storage map in the pallet
        storage_name: String,
        /// Scale encoded bytes of the actual key
        key: Vec<u8>,
        /// Key hashing algorithm
        hasher: HashingAlgorithm,
    },
    /// Double Storage Map
    DoubleStorageMap {
        /// pallet name as specified in the runtime
        pallet_name: String,
        /// String name of the storage double map in the pallet
        storage_name: String,
        // Scale encoded bytes of the keys
        first_key: Vec<u8>,
        second_key: Vec<u8>,
        /// First key hashing algorithm
        first_hasher: HashingAlgorithm,
        /// Second key hashing algorithm
        second_hasher: HashingAlgorithm,
    },
    /// Storage N Map
    StorageNMap {
        /// pallet name as specified in the runtime
        pallet_name: String,
        /// String name of the storage N map in the pallet
        storage_name: String,
        // A vector of the hashing algorithm and the actual scale encoded keys
        keys: Vec<(HashingAlgorithm, Vec<u8>)>,
    },
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum HashingAlgorithm {
    /// Blake 2
    Blake2_128Concat,
    /// TwoX
    TwoX64Concat,
    /// Identity
    Identity,
}
