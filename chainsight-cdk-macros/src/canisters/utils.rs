use std::path::PathBuf;

pub fn extract_contract_name_from_path(s: &str) -> String {
    let path = PathBuf::from(s);
    let name = path.file_stem().expect("file_stem failed");
    name.to_str().expect("to_str failed").to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_contract_name_from_path() {
        let path = "__interfaces/Oracle.json";
        assert_eq!(extract_contract_name_from_path(path), "Oracle");
    }
}
