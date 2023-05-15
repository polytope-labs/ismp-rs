use crate::prelude::{String, Vec};
use codec::{Decode, Encode};
use primitive_types::{H160, U256};

/// The Storage Kind for GET request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum StorageKind {
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
    /// Different storage types supported by the EVM
    pub evm_storage_type: EvmStorageType,
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum EvmStorageType {
    /// An EVM Primitive value
    EvmPrimitive(EvmPrimitiveType),
    /// Item index in a solidity array
    Array(U256),
    /// A key in a solidity map, this should be big endian byte representation of the key
    Map(Vec<u8>),
}

/// The Storage Type for EVM Get Request.
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq, scale_info::TypeInfo)]
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
pub enum EvmPrimitiveType {
    /// An EVM Address
    Address,
    /// An EVM uint8
    Uint8,
    /// An EVM uint32
    Uint32,
    /// An EVM uint64
    Uint64,
    /// An EVM uint128
    Uint128,
    /// An EVM uint256
    Uint256,
    /// An EVM boolean type
    Boolean,
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
/// The storage API operates by storing and loading entries into and from a single storage cells,
/// where each storage cell is accessed under its own dedicated storage key.
/// Ink Storage data is always encoded with the SCALE codec.
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
    /// Scale encoded key in a mapping
    Mapping(Vec<u8>),
    /// Key for any field of the contract struct as specified in the storage metadata
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
        // Scale encoded bytes of the actual key
        key: Vec<u8>,
        hasher: HashingAlgorithm,
    },
    /// Double Storage Map
    DoubleStorageMap {
        /// pallet name as specified in the runtime
        pallet_name: String,
        /// String name of the storage double map in the pallet
        storage_name: String,
        // Scale encoded bytes of the actual key
        first_key: Vec<u8>,
        second_key: Vec<u8>,
        first_hasher: HashingAlgorithm,
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
