use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
use std::time;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use sha2::{Digest, Sha256};
use hex;

pub struct Identity {
    node_id: String,
    cert_der: CertificateDer<'static>,
    private_key_der: PrivateKeyDer<'static>,
    pub_key_hash: String,
}

impl Identity {
    pub fn new(node_id: String) -> Result<Self, rcgen::Error> {
        let mut params = CertificateParams::default();

        let now = std::time::SystemTime::now();
        let years_delay = 20 * 365 * 24 * 60 * 60;
        let expiry = now + std::time::Duration::from_secs(years_delay);

        params.not_before = now.into();
        params.not_after = expiry.into();

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, &node_id);
        params.distinguished_name = dn;
        params.subject_alt_names = vec![SanType::DnsName(rcgen::string::Ia5String::try_from("netcats.local")?)];
        let key_pair = KeyPair::generate_for(&rcgen::PKCS_ED25519)?;

        let certified_key = params.self_signed(&key_pair)?;

        let cert_bytes = certified_key.der().to_vec();
        let private_key_bytes= certified_key.der().as_ref().to_vec();

        let cert_der = CertificateDer::from(cert_bytes);

        let private_key_der = PrivateKeyDer::Pkcs8(private_key_bytes.into());

        let mut hasher = Sha256::new();

        hasher.update(cert_der.as_ref());

        let hash_result = hasher.finalize();
        let pub_key_hash = hex::encode(hash_result);

        Ok( Self {
            node_id,
            cert_der,
            private_key_der,
            pub_key_hash,
        })

    }
}