#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Bytes, Env, String,
};

/// Stores the notarized record for a document
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotarizedDocument {
    pub doc_hash: Bytes,        // SHA-256 hash of the document
    pub notary: Address,        // Address of the notary
    pub timestamp: u64,         // Ledger timestamp at notarization
    pub metadata: String,       // Optional label/description
}

/// Storage key per document hash
#[contracttype]
pub enum DataKey {
    Document(Bytes),
}

#[contract]
pub struct NotaryContract;

#[contractimpl]
impl NotaryContract {
    /// Notarize a document hash.
    /// - `doc_hash`: SHA-256 hash of the document (32 bytes)
    /// - `metadata`: Short label or description (e.g., "Contract v2")
    ///
    /// The caller (notary) must sign the transaction.
    /// Fails if the hash has already been notarized.
    pub fn notarize(
        env: Env,
        notary: Address,
        doc_hash: Bytes,
        metadata: String,
    ) -> NotarizedDocument {
        // Require notary's signature
        notary.require_auth();

        // Reject if already notarized — immutability is the whole point
        let key = DataKey::Document(doc_hash.clone());
        if env.storage().persistent().has(&key) {
            panic!("document already notarized");
        }

        // Enforce 32-byte hash (SHA-256)
        if doc_hash.len() != 32 {
            panic!("doc_hash must be exactly 32 bytes (SHA-256)");
        }

        let record = NotarizedDocument {
            doc_hash: doc_hash.clone(),
            notary,
            timestamp: env.ledger().timestamp(),
            metadata,
        };

        // Persist with no expiration — notarization is permanent
        env.storage().persistent().set(&key, &record);

        record
    }

    /// Verify a document hash. Returns the notarization record if found.
    /// Returns None if the document was never notarized.
    pub fn verify(env: Env, doc_hash: Bytes) -> Option<NotarizedDocument> {
        let key = DataKey::Document(doc_hash);
        env.storage().persistent().get(&key)
    }

    /// Check whether a document hash has been notarized.
    pub fn is_notarized(env: Env, doc_hash: Bytes) -> bool {
        let key = DataKey::Document(doc_hash);
        env.storage().persistent().has(&key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger, vec, Env};

    fn mock_hash(env: &Env, seed: u8) -> Bytes {
        Bytes::from_array(env, &[seed; 32])
    }

    #[test]
    fn test_notarize_and_verify() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, NotaryContract);
        let client = NotaryContractClient::new(&env, &contract_id);

        let notary = Address::generate(&env);
        let hash = mock_hash(&env, 0xab);
        let label = String::from_str(&env, "Agreement v1");

        // Notarize
        let record = client.notarize(&notary, &hash, &label);
        assert_eq!(record.notary, notary);
        assert_eq!(record.doc_hash, hash);

        // Verify
        let fetched = client.verify(&hash).unwrap();
        assert_eq!(fetched.metadata, label);

        // is_notarized
        assert!(client.is_notarized(&hash));
    }

    #[test]
    #[should_panic(expected = "document already notarized")]
    fn test_double_notarize_fails() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, NotaryContract);
        let client = NotaryContractClient::new(&env, &contract_id);

        let notary = Address::generate(&env);
        let hash = mock_hash(&env, 0x01);
        let label = String::from_str(&env, "test");

        client.notarize(&notary, &hash, &label);
        client.notarize(&notary, &hash, &label); // must panic
    }

    #[test]
    #[should_panic(expected = "doc_hash must be exactly 32 bytes")]
    fn test_invalid_hash_length() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, NotaryContract);
        let client = NotaryContractClient::new(&env, &contract_id);

        let notary = Address::generate(&env);
        let bad_hash = Bytes::from_array(&env, &[0u8; 16]); // 16 bytes, not 32
        client.notarize(&notary, &bad_hash, &String::from_str(&env, "bad"));
    }

    #[test]
    fn test_verify_nonexistent_returns_none() {
        let env = Env::default();

        let contract_id = env.register_contract(None, NotaryContract);
        let client = NotaryContractClient::new(&env, &contract_id);

        let hash = mock_hash(&env, 0xff);
        assert!(client.verify(&hash).is_none());
        assert!(!client.is_notarized(&hash));
    }
}