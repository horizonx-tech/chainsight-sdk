use std::{cmp::max, fs, ops::Deref, path::Path};

use anyhow::Ok;
use candid::{
    bindings::rust::{compile, Target},
    check_prog,
    types::{Type, TypeInner},
    IDLProg, TypeEnv,
};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct CanisterMethodIdentifier {
    pub identifier: String,
    type_env: TypeEnv,
}
impl CanisterMethodIdentifier {
    pub const REQUEST_ARGS_TYPE_NAME: &'static str = "RequestArgsType";
    pub const RESPONSE_TYPE_NAME: &'static str = "ResponseType";
    pub const SUFFIX_IN_DID: &'static str = "InDid";

    pub fn new(s: &str) -> anyhow::Result<Self> {
        Self::new_internal(s, None)
    }

    pub fn new_with_did(s: &str, dependended_did: String) -> anyhow::Result<Self> {
        // NOTE: If the reserved Type specified in the .did to import is used, a Compile error will occur due to duplication.
        let (s, dependended_did) = avoid_using_reserved_types(s, &dependended_did);
        Self::new_internal(&s, Some(dependended_did))
    }

    fn new_internal(s: &str, dependended_did: Option<String>) -> anyhow::Result<Self> {
        let (identifier, args_ty, response_ty) = extract_elements(s)?;
        let did: String = Self::generate_did(&args_ty, &response_ty);

        let ast: IDLProg = if let Some(base_did) = dependended_did {
            format!("{}\n\n{}", base_did, did)
        } else {
            did.to_string()
        }
        .parse()?;
        let mut type_env = TypeEnv::new();
        let _ = check_prog(&mut type_env, &ast);

        Ok(Self {
            identifier,
            type_env,
        })
    }

    pub fn compile(&self) -> anyhow::Result<String> {
        anyhow::ensure!(self.compilable(), "Not compilable IDLProg");

        let mut config = candid::bindings::rust::Config::new();
        // // Update the structure derive to chainsight's own settings
        config.type_attributes = "#[derive(Clone, Debug, candid :: CandidType, candid :: Deserialize, serde :: Serialize, chainsight_cdk_macros :: StableMemoryStorable)]".to_string();
        config.target = Target::CanisterStub;
        let contents = compile(&config, &self.type_env, &None);

        let mut lines = contents
            .lines()
            // Delete initial 'use' declarations
            .skip(4)
            // Delete comment lines and blank lines
            .filter(|line| !(line.starts_with("//") || line.is_empty()))
            .map(|line| {
                // convert icp's own Nat and Int types to rust native type
                // NOTE: num-traits can be used, but is not used to reduce dependencies
                //  https://forum.dfinity.org/t/candid-nat-to-u128/16016
                //  https://discord.com/channels/748416164832608337/872791506853978142/1162494173933481984
                // NOTE: when ready to convert to u128/i128, consider with EthAbiEncoder's Encoder trait
                if line.contains("candid::Nat") {
                    return line.replace("candid::Nat", "u128");
                }
                if line.contains("candid::Int") {
                    return line.replace("candid::Int", "i128");
                }
                line.to_string()
            })
            .collect::<Vec<_>>();
        lines.insert(0, "#![allow(dead_code, unused_imports)]".to_string());
        lines.insert(
            1,
            "use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};".to_string(),
        );
        Ok(lines.join("\n"))
    }

    pub fn get_types(&self) -> (Option<&Type>, Option<&Type>) {
        (
            self.find_type(Self::REQUEST_ARGS_TYPE_NAME),
            self.find_type(Self::RESPONSE_TYPE_NAME),
        )
    }

    pub fn get_type(&self, key: &str) -> Option<&Type> {
        self.find_type(key)
    }

    fn compilable(&self) -> bool {
        let (args_ty, response_ty) = self.get_types();
        let not_compilable_type = &TypeInner::Unknown;

        if args_ty.is_some_and(|ty| ty.deref().eq(not_compilable_type)) {
            return false;
        }
        if response_ty.is_some_and(|ty| ty.deref().eq(not_compilable_type)) {
            return false;
        }

        true
    }

    fn generate_did(args_ty: &str, response_ty: &str) -> String {
        let args_ty_did = if args_ty.is_empty() {
            "".to_string()
        } else {
            generate_did_type(Self::REQUEST_ARGS_TYPE_NAME, args_ty)
        };
        let response_ty_did = if response_ty.is_empty() {
            "".to_string()
        } else {
            generate_did_type(Self::RESPONSE_TYPE_NAME, response_ty)
        };
        format!("{}\n{}", args_ty_did, response_ty_did)
    }

    fn find_type(&self, key: &str) -> Option<&Type> {
        let ty = self.type_env.find_type(key);
        ty.ok()
    }
}

fn generate_did_type(key: &str, value: &str) -> String {
    format!("type {} = {};", key, value)
}

fn extract_elements(s: &str) -> anyhow::Result<(String, String, String)> {
    let (identifier, remains) = s
        .split_once(':')
        .expect("Invalid canister method identifier");
    let (args_ty, response_ty) = remains
        .split_once("->")
        .expect("Invalid canister method identifier");

    let trim_type_str = |s: &str| {
        let trimed = s.trim();
        let removed_brackets = trimed.trim_matches(|c| c == '(' || c == ')');
        removed_brackets.trim().to_string()
    };

    Ok((
        identifier.trim().to_string(),
        trim_type_str(args_ty),
        trim_type_str(response_ty),
    ))
}

fn avoid_using_reserved_types(s: &str, did: &str) -> (String, String) {
    let req_ty = CanisterMethodIdentifier::REQUEST_ARGS_TYPE_NAME;
    let res_ty = CanisterMethodIdentifier::RESPONSE_TYPE_NAME;
    let base_suffix = CanisterMethodIdentifier::SUFFIX_IN_DID;

    // Check maximum value of suffix for reserved type
    let pattern = format!(
        r"(?P<name>({}|{}))_{}_(?P<suffix_num>\d+)",
        req_ty, res_ty, base_suffix
    );
    let re = Regex::new(&pattern).expect("Invalid regex pattern");
    let mut max_number = 0;
    for line in did.lines() {
        for cap in re.captures_iter(line) {
            let suffix_num = cap.name("suffix_num").unwrap().as_str();
            if let core::result::Result::Ok(number) = suffix_num.parse::<u32>() {
                max_number = max(max_number, number);
            }
        }
    }
    max_number += 1;

    // Replace reserved type (did)
    let pattern = format!(r" (?P<name>({}|{}))(?P<last_char>(;|,| ))", req_ty, res_ty);
    let re = Regex::new(&pattern).expect("Invalid regex pattern");
    let replace_reserved_type = |s: &str| {
        let replaced_s = re.replace_all(s, |caps: &regex::Captures| {
            let name = caps.name("name").unwrap().as_str();
            let last_char = caps.name("last_char").unwrap().as_str();
            format!(" {}_{}_{}{}", name, base_suffix, max_number, last_char)
        });
        replaced_s.to_string()
    };
    let replaced_did_lines = did.lines().map(replace_reserved_type).collect::<Vec<_>>();

    // Replace reserved type (identifier)
    let pattern = format!(
        r"(?P<first_char>(\(| ))(?P<name>({}|{}))(?P<last_char>(\)|;|,| ))",
        req_ty, res_ty
    );
    let re = Regex::new(&pattern).expect("Invalid regex pattern");
    let replaced_s = re.replace_all(s, |caps: &regex::Captures| {
        let first_char = caps.name("first_char").unwrap().as_str();
        let name = caps.name("name").unwrap().as_str();
        let last_char = caps.name("last_char").unwrap().as_str();
        format!(
            "{}{}_{}_{}{}",
            first_char, name, base_suffix, max_number, last_char
        )
    });

    (replaced_s.to_string(), replaced_did_lines.join("\n"))
}

// Read .did and remove 'service' section
pub fn read_did_to_string_without_service<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let mut data = fs::read_to_string(path)?;
    Ok(exclude_service_from_did_string(&mut data))
}
fn exclude_service_from_did_string(data: &mut String) -> String {
    // remove 'service' section
    if let Some(service_start_idx) = data.find("service : {") {
        data.truncate(service_start_idx);
    }

    data.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    use insta::assert_snapshot;

    const TEST_IDENTS: &'static [(&'static str, &'static str); 6] = &[
        (&"normal", &"update_value : (nat64) -> (text)"),
        (&"normal_nat", &"use_nat : (nat32) -> (nat)"),
        (&"normal_int", &"use_nat : (int32) -> (int)"),
        (&"no_args", &"get_value : () -> (text)"),
        (
            &"record",
            &"get_snapshot : (nat64) -> (record { value : text; timestamp : nat64 })",
        ),
        (
            &"nested_record",
            &"get_last_snapshot : () -> (record { dai : record { usd : float64 } })",
        ),
    ];

    const TEST_IDENTS_WITH_DID: &'static [(&'static str, &'static str, &'static str); 5] = &[
        (
            &"single",
            &"get_snapshot : (nat64) -> (Snapshot)",
            &"type Snapshot = record { value : text; timestamp : nat64 };",
        ),
        (
            &"multiple",
            &"get_snapshot : (nat64) -> (Snapshot)",
            &r#"type CurrencyValue = record { usd : float64 };
type Snapshot = record { value : SnapshotValue; timestamp : nat64 };
type SnapshotValue = record { dai : CurrencyValue };
"#,
        ),
        (
            &"with_variant",
            &"init_in : (Env) -> (Result)",
            &r#"type Env = variant { Production; Test; LocalDevelopment };
type InitError = variant {
  InvalidDestination : text;
  InvalidPrincipal : principal;
  InvalidContent : text;
  InvalidRequest : text;
};
type Result = variant { Ok; Err : InitError };
"#,
        ),
        (
            &"with_vec",
            &"get_sources : () -> (vec Sources)",
            &r#"type HttpsSnapshotIndexerSourceAttrs = record {
  queries : vec record { text; text };
};
type SourceType = variant { evm; https; chainsight };
type Sources = record {
  source : text;
  interval_sec : opt nat32;
  attributes : HttpsSnapshotIndexerSourceAttrs;
  source_type : SourceType;
};"#,
        ),
        (
            &"with_reserved_type",
            &"get_snapshot : (RequestArgsType) -> (ResponseType)",
            &r#"type RequestArgsType = nat64;
type ResponseType = text;"#,
        ),
    ];

    #[test]
    fn test_compile() {
        for (label, s) in TEST_IDENTS {
            let ident = CanisterMethodIdentifier::new(s).expect("Failed to parse");
            let compiled = ident.compile().unwrap();
            assert_snapshot!(format!("compile__{}", label.to_string()), compiled);
        }
    }

    #[test]
    fn test_compile_with_depended_did() {
        for (label, s, did) in TEST_IDENTS_WITH_DID {
            let ident = CanisterMethodIdentifier::new_with_did(s, did.to_string())
                .expect("Failed to parse");
            let compiled = ident.compile().unwrap();
            assert_snapshot!(
                format!("compile_with_depended_did__{}", label.to_string()),
                compiled
            );
        }
    }

    #[test]
    fn test_compile_when_not_compilable() {
        let s = "get_snapshot : (Input) -> (Output)";
        let ident = CanisterMethodIdentifier::new(s).expect("Failed to parse");
        assert_eq!(
            ident.compile().err().unwrap().to_string(),
            "Not compilable IDLProg"
        );
    }

    #[test]
    fn test_compilable() {
        // compilable
        for (_, s) in TEST_IDENTS {
            let ident = CanisterMethodIdentifier::new(s).expect("Failed to parse");
            assert!(ident.compilable());
        }
        for (_, s, did) in TEST_IDENTS_WITH_DID {
            let ident = CanisterMethodIdentifier::new_with_did(s, did.to_string())
                .expect("Failed to parse");
            assert!(ident.compilable());
        }

        // not compilable
        let not_compilable = vec![
            "get_snapshot : (Input) -> (Output)",
            "get_snapshot : (nat64) -> (Snapshot)",
            "get_snapshot : (Snapshot) -> (nat64)",
        ];
        for s in not_compilable {
            let ident = CanisterMethodIdentifier::new(s).expect("Failed to parse");
            assert!(!ident.compilable());
        }

        let not_compilable_with_did = vec![
            (
                "get_snapshot : (Snapshot) -> (Snapshot)",
                "type Snapshot_1 = record { value : text; timestamp : nat64 };",
            ),
            (
                "get_snapshot : (nat64) -> (Snapshot)",
                "type Snapshot_1 = record { value : text; timestamp : nat64 };",
            ),
            (
                "get_snapshot : (Snapshot) -> (nat64)",
                "type Snapshot_1 = record { value : text; timestamp : nat64 };",
            ),
        ];
        for (s, did) in not_compilable_with_did {
            let ident = CanisterMethodIdentifier::new_with_did(s, did.to_string())
                .expect("Failed to parse");
            assert!(!ident.compilable());
        }
    }

    #[test]
    fn test_generate_did() {
        let key = "Account";
        let value = "record { owner : principal; subaccount : opt blob }";
        let did = generate_did_type(key, value);
        assert_snapshot!(did);
    }

    #[test]
    fn test_extract_elements() {
        for (label, s) in TEST_IDENTS {
            let result = extract_elements(s).expect("Failed to parse");
            assert_snapshot!(
                format!("extract_elements__{}", label.to_string()),
                format!("{:#?}", result)
            );
        }
    }

    #[test]
    fn test_extract_elements_multi_nested_record() {
        let s =
            "icrc2_allowance : (record { account : record { owner : principal; subaccount : opt blob }; spender : record { owner : principal; subaccount : opt blob } }) -> (record { allowance : nat; expires_at : opt nat64 })";
        let (identifier, args_ty, response_ty) = extract_elements(s).expect("Failed to parse");
        assert_eq!(identifier, "icrc2_allowance");
        assert_eq!(args_ty, "record { account : record { owner : principal; subaccount : opt blob }; spender : record { owner : principal; subaccount : opt blob } }");
        assert_eq!(
            response_ty,
            "record { allowance : nat; expires_at : opt nat64 }"
        );
    }

    #[test]
    fn test_exclude_service_from_did_string() {
        let expected = r#"type List = opt record { head: int; tail: List };
type byte = nat8;".to_string()"#;
        let mut actual = format!("{}\n", expected)
            + r#"service : {
  f : (byte, int, nat, int8) -> (List);
  g : (List) -> (int) query;
}"#;

        assert_eq!(exclude_service_from_did_string(&mut actual), expected);
    }
}
