//! # Signature Verifier
//!
//! Verifica assinaturas criptográficas de módulos.
//!
//! ## Esquema de Assinatura
//! - Hash: SHA-256 ou SHA-3
//! - Assinatura: Ed25519 ou RSA-4096
//! - Certificado na whitelist do kernel

use alloc::vec::Vec;

/// Resultado da verificação
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyResult {
    /// Assinatura válida
    Valid,
    /// Assinatura inválida
    Invalid,
    /// Certificado não encontrado na whitelist
    UntrustedCert,
    /// Módulo não tem assinatura
    NotSigned,
    /// Erro interno na verificação
    Error,
}

/// Verificador de assinaturas de módulos
pub struct SignatureVerifier {
    /// Se verificação está habilitada
    enabled: bool,
    /// Hash dos certificados confiáveis
    trusted_certs: Vec<[u8; 32]>,
}

impl SignatureVerifier {
    /// Cria um novo verificador
    pub const fn new() -> Self {
        Self {
            enabled: true,
            trusted_certs: Vec::new(),
        }
    }

    /// Verifica assinatura de um módulo
    pub fn verify(&self, module_data: &[u8]) -> bool {
        if !self.enabled {
            return true;
        }

        // Verificar se tem seção de assinatura
        if !self.has_signature(module_data) {
            crate::kwarn!("(Verifier) Módulo não assinado");
            return false;
        }

        // Extrair assinatura e dados
        let (data, signature) = match self.extract_signature(module_data) {
            Some(parts) => parts,
            None => {
                crate::kerror!("(Verifier) Falha ao extrair assinatura");
                return false;
            }
        };

        // Verificar assinatura
        match self.verify_signature(data, signature) {
            VerifyResult::Valid => true,
            VerifyResult::Invalid => {
                crate::kerror!("(Verifier) Assinatura inválida!");
                false
            }
            VerifyResult::UntrustedCert => {
                crate::kerror!("(Verifier) Certificado não confiável!");
                false
            }
            VerifyResult::NotSigned => {
                crate::kwarn!("(Verifier) Módulo sem assinatura");
                false
            }
            VerifyResult::Error => {
                crate::kerror!("(Verifier) Erro na verificação");
                false
            }
        }
    }

    /// Desabilita verificação (APENAS para desenvolvimento!)
    #[cfg(debug_assertions)]
    pub fn disable(&mut self) {
        crate::kwarn!("(Verifier) VERIFICAÇÃO DESABILITADA - APENAS DEBUG!");
        self.enabled = false;
    }

    /// Adiciona certificado à whitelist
    pub fn add_trusted_cert(&mut self, cert_hash: [u8; 32]) {
        if !self.trusted_certs.contains(&cert_hash) {
            self.trusted_certs.push(cert_hash);
        }
    }

    /// Remove certificado da whitelist
    pub fn remove_trusted_cert(&mut self, cert_hash: &[u8; 32]) {
        self.trusted_certs.retain(|c| c != cert_hash);
    }

    // --- Funções internas ---

    fn has_signature(&self, data: &[u8]) -> bool {
        // Verificar magic de seção de assinatura
        // Formato: últimos 4 bytes são "RSIG"
        if data.len() < 4 {
            return false;
        }

        let sig_magic = &data[data.len() - 4..];
        sig_magic == b"RSIG"
    }

    fn extract_signature<'a>(&self, data: &'a [u8]) -> Option<(&'a [u8], &'a [u8])> {
        // Formato esperado:
        // [ELF data][signature 64 bytes]["RSIG"]
        if data.len() < 68 {
            return None;
        }

        let elf_end = data.len() - 68;
        let elf_data = &data[..elf_end];
        let signature = &data[elf_end..data.len() - 4];

        Some((elf_data, signature))
    }

    fn verify_signature(&self, _data: &[u8], _signature: &[u8]) -> VerifyResult {
        // TODO: Implementar verificação real Ed25519
        // Por agora, aceita tudo em debug, rejeita tudo em release

        #[cfg(debug_assertions)]
        {
            crate::ktrace!("(Verifier) DEBUG: Aceitando assinatura");
            VerifyResult::Valid
        }

        #[cfg(not(debug_assertions))]
        {
            // Em release, requer implementação real
            VerifyResult::NotSigned
        }
    }
}
