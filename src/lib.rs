use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        CompactSize { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        match self.value {
            0..=252 => vec![self.value as u8],
            253..=0xFFFF => {
                let mut bytes = vec![0xFD];
                bytes.extend_from_slice(&(self.value as u16).to_le_bytes());
                bytes
            }
            0x10000..=0xFFFFFFFF => {
                let mut bytes = vec![0xFE];
                bytes.extend_from_slice(&(self.value as u32).to_le_bytes());
                bytes
            }
            _ => {
                let mut bytes = vec![0xFF];
                bytes.extend_from_slice(&(self.value).to_le_bytes());
                bytes
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }
        // Check that enough bytes are available based on prefix.
        match bytes[0] {
            0..=252 => Ok((
                CompactSize {
                    value: bytes[0] as u64,
                },
                1,
            )),
            0xFD => {
                if bytes.len() < 3 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let val = u16::from_le_bytes([bytes[1], bytes[2]]);
                Ok((CompactSize { value: val as u64 }, 3))
            }

            0xFE => {
                if bytes.len() < 5 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let val = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
                Ok((CompactSize { value: val as u64 }, 5))
            }

            0xFF => {
                if bytes.len() < 9 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let mut array = [0u8; 8];
                array.copy_from_slice(&bytes[1..9]);
                let val = u64::from_le_bytes(array);
                Ok((CompactSize { value: val }, 9))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        let hex_string = hex::encode(self.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid length: Must be 32"));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Txid(array))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        OutPoint {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut result = Vec::new();
        result.extend_from_slice(&self.txid.0);
        result.extend_from_slice(&self.vout.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let txid_byte: [u8; 32] = bytes[0..32].try_into().expect("Invalid error");
        let vout_bytes: [u8; 4] = bytes[32..36].try_into().expect("Invalid error");
        let vout = u32::from_le_bytes(vout_bytes);
        Ok((
            OutPoint {
                txid: Txid(txid_byte),
                vout,
            },
            36,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Script { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let mut result: Vec<u8> = Vec::new();
        result.extend_from_slice(&CompactSize::new(self.bytes.len() as u64).to_bytes());
        result.extend_from_slice(&self.bytes);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        let (compact_size, prefix_len) = CompactSize::from_bytes(bytes)?;
        let script_size = compact_size.value as usize;
        let total_bytes_needed = prefix_len + script_size;
        if bytes.len() < total_bytes_needed {
            return Err(BitcoinError::InsufficientBytes);
        }

        let script_bytes = bytes[prefix_len..total_bytes_needed].to_vec();
        let script = Script {
            bytes: script_bytes,
        };

        Ok((script, total_bytes_needed))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        TransactionInput {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut result = Vec::new();
        result.extend_from_slice(&self.previous_output.to_bytes());
        result.extend_from_slice(&self.script_sig.to_bytes());
        result.extend_from_slice(&self.sequence.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let mut cursor = 0;
        let (previous_output, outpoint_len) = OutPoint::from_bytes(&bytes[cursor..])?;
        cursor += outpoint_len;
        let (script_sig, script_len) = Script::from_bytes(&bytes[cursor..])?;
        cursor += script_len;

        if bytes[cursor..].len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let seq_bytes: [u8; 4] = bytes[cursor..cursor + 4].try_into().expect("Invalid bytes");
        cursor += 4;
        let sequence = u32::from_le_bytes(seq_bytes);

        let txin = TransactionInput {
            previous_output,
            script_sig,
            sequence,
        };
        Ok((txin, cursor))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        BitcoinTransaction {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&u32::to_le_bytes(self.version));
        bytes.extend_from_slice(&CompactSize::new(self.inputs.len() as u64).to_bytes());
        for input in &self.inputs {
            bytes.extend_from_slice(&input.to_bytes());
        }
        bytes.extend_from_slice(&u32::to_le_bytes(self.lock_time));

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        let mut cursor = 0;
        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let version_bytes: [u8; 4] = bytes[cursor..cursor + 4].try_into().expect("Invalid");
        let version = u32::from_le_bytes(version_bytes);
        cursor += 4;

        let (input_count_cs, compact_len) = CompactSize::from_bytes(&bytes[cursor..])?;
        let input_count = input_count_cs.value;
        cursor += compact_len;

        let mut inputs = Vec::with_capacity(input_count as usize);
        for _ in 0..input_count {
            let (tx_in, tx_len) = TransactionInput::from_bytes(&bytes[cursor..])?;
            inputs.push(tx_in);
            cursor += tx_len;
        }

        if bytes[cursor..].len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let lock_time_bytes: [u8; 4] = bytes[cursor..cursor + 4].try_into().expect("Invalid");
        let lock_time = u32::from_le_bytes(lock_time_bytes);
        cursor += 4;
        Ok((
            BitcoinTransaction {
                version,
                inputs,
                lock_time,
            },
            cursor,
        ))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        write!(
            f,
            "Version: {}, Lock Time: {}, Previous Output Vout: {:?}",
            self.version, self.lock_time, self.inputs[0].previous_output.vout
        )
    }
}
