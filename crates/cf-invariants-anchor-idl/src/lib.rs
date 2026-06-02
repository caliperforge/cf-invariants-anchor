// cf-invariants-anchor-idl — Anchor IDL → ContractSurface.
//
// Targets the Anchor 0.30+ IDL shape (the same one Codama emits and
// Crucible's `declare_fuzz_program!` consumes). We intentionally do
// NOT depend on `anchor-lang-idl` — it would drag the full Anchor +
// solana-program closure for one struct parse. Phase 0 reads the
// fields we actually use (program id, name, instructions, account
// types) and ignores the rest.

use cf_invariants_anchor_core::{BalanceField, ContractSurface, Instruction};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdlError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("idl missing required field: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, Deserialize)]
struct RawIdl {
    address: Option<String>,
    metadata: Option<RawMetadata>,
    name: Option<String>,
    instructions: Vec<RawInstruction>,
    #[serde(default)]
    accounts: Vec<RawAccount>,
    #[serde(default)]
    types: Vec<RawTypeDef>,
}

#[derive(Debug, Deserialize)]
struct RawMetadata {
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawInstruction {
    name: String,
    #[serde(default)]
    args: Vec<RawArg>,
    #[serde(default)]
    accounts: Vec<RawIxAccount>,
}

#[derive(Debug, Deserialize)]
struct RawArg {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawIxAccount {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawAccount {
    name: String,
}

#[derive(Debug, Deserialize)]
struct RawTypeDef {
    name: String,
    #[serde(rename = "type")]
    ty: RawTypeBody,
}

#[derive(Debug, Deserialize)]
struct RawTypeBody {
    kind: String,
    #[serde(default)]
    fields: Vec<RawField>,
}

#[derive(Debug, Deserialize)]
struct RawField {
    name: String,
    #[serde(rename = "type")]
    ty: serde_json::Value,
}

/// Parse an Anchor IDL JSON file into a ContractSurface.
pub fn ingest_path(path: &Path) -> Result<ContractSurface, IdlError> {
    let bytes = std::fs::read(path)?;
    ingest_bytes(&bytes)
}

/// Parse Anchor IDL JSON bytes into a ContractSurface.
pub fn ingest_bytes(bytes: &[u8]) -> Result<ContractSurface, IdlError> {
    let raw: RawIdl = serde_json::from_slice(bytes)?;

    let program_id = raw
        .address
        .clone()
        .ok_or(IdlError::MissingField("address"))?;
    let program_name = raw
        .metadata
        .as_ref()
        .and_then(|m| m.name.clone())
        .or_else(|| raw.name.clone())
        .ok_or(IdlError::MissingField("metadata.name | name"))?;

    let instructions = raw
        .instructions
        .into_iter()
        .map(|ix| Instruction {
            name: ix.name,
            args: ix.args.into_iter().map(|a| a.name).collect(),
            accounts: ix.accounts.into_iter().map(|a| a.name).collect(),
        })
        .collect();

    // Balance-bearing fields = scalar uint fields on struct accounts
    // referenced by the `accounts` array. Phase 0 heuristic; refined
    // by structural reasoning in Phase 1 (account-mutability set across
    // movement instructions).
    let account_names: Vec<&str> = raw.accounts.iter().map(|a| a.name.as_str()).collect();
    let mut balance_fields = Vec::new();
    for tdef in &raw.types {
        if tdef.ty.kind != "struct" {
            continue;
        }
        if !account_names.contains(&tdef.name.as_str()) {
            continue;
        }
        for f in &tdef.ty.fields {
            if let Some(ty_str) = scalar_uint_type(&f.ty) {
                balance_fields.push(BalanceField {
                    account: tdef.name.clone(),
                    field: f.name.clone(),
                    ty: ty_str.to_string(),
                });
            }
        }
    }

    Ok(ContractSurface {
        program_id,
        program_name,
        instructions,
        balance_fields,
    })
}

/// Return the IDL type string iff this field is a scalar unsigned integer.
fn scalar_uint_type(ty: &serde_json::Value) -> Option<&'static str> {
    let s = ty.as_str()?;
    match s {
        "u8" => Some("u8"),
        "u16" => Some("u16"),
        "u32" => Some("u32"),
        "u64" => Some("u64"),
        "u128" => Some("u128"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../../references/vault_ref/idls/vault_ref.json");

    #[test]
    fn ingests_vault_ref_surface() {
        let surface = ingest_bytes(FIXTURE.as_bytes()).expect("parse");
        assert_eq!(surface.program_name, "vault_ref");
        assert_eq!(
            surface.program_id,
            "Va111tRef1111111111111111111111111111111111"
        );
        let names: Vec<_> = surface.instructions.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"deposit") && names.contains(&"withdraw"));
        assert!(surface
            .balance_fields
            .iter()
            .any(|f| f.account == "Vault" && f.field == "amount" && f.ty == "u64"));
    }

    #[test]
    fn movement_instructions_pick_up_deposit_withdraw() {
        let surface = ingest_bytes(FIXTURE.as_bytes()).unwrap();
        let movement: Vec<_> = surface
            .movement_instructions()
            .into_iter()
            .map(|i| i.name.as_str())
            .collect();
        assert!(movement.contains(&"deposit"));
        assert!(movement.contains(&"withdraw"));
        assert!(!movement.contains(&"initialize"));
    }
}
