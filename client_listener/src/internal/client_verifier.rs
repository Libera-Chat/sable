use x509_parser::prelude::*;

pub struct AcceptAnyClientCertVerifier(rustls::DistinguishedNames);

impl AcceptAnyClientCertVerifier {
    pub fn new(cert_for_name: &rustls::Certificate) -> Self {
        let mut inner = Vec::new();

        if let Ok((_, cert)) = X509Certificate::from_der(&cert_for_name.0) {
            let name = cert.subject();
            inner.push(rustls::internal::msgs::base::PayloadU16::new(
                name.as_raw().to_vec(),
            ));
        }

        Self(inner)
    }
}

impl rustls::server::ClientCertVerifier for AcceptAnyClientCertVerifier {
    fn client_auth_root_subjects(&self) -> Option<rustls::DistinguishedNames> {
        // We have to provide some data here - None will abort the connection,
        // and Some(vec![]) will not send a certificate request
        Some(self.0.clone())
    }

    fn verify_client_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _now: std::time::SystemTime,
    ) -> Result<rustls::server::ClientCertVerified, rustls::Error> {
        tracing::error!(?end_entity, ?_intermediates, ?_now, "verifying");

        if X509Certificate::from_der(&end_entity.0).is_err() {
            Err(rustls::Error::InvalidCertificateEncoding)
        } else {
            Ok(rustls::server::ClientCertVerified::assertion())
        }
    }

    fn client_auth_mandatory(&self) -> Option<bool> {
        Some(false)
    }
}
