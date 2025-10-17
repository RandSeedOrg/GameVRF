use hex;

pub fn hex_to_seed(s: &str) -> Result<[u8; 32], String> {
  if s.len() != 64 {
    return Err(format!("Invalid seed length: expected 64 characters, got {}", s.len()));
  }
  let bytes = hex::decode(s).map_err(|e| format!("Failed to decode hex string: {}", e))?;
  let mut array = [0u8; 32];
  array.copy_from_slice(&bytes[..32]);
  Ok(array)
}

pub fn seed_to_hex(bytes: &[u8; 32]) -> String {
  hex::encode(bytes)
}


#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_hex_to_seed() {
    let s = "deadbeef01000000000000000000000000000000000000000000000000000050";
    let bytes = hex_to_seed(s).unwrap();
    assert_eq!(bytes, [0xde, 0xad, 0xbe, 0xef, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x50]);
  }


  #[test]
  fn test_seed_to_hex() {
    let bytes: [u8; 32] = [0xde, 0xad, 0xbe, 0xef, 0x1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let s = seed_to_hex(&bytes);
    print!("{}", s);
    assert_eq!(s, "deadbeef01000000000000000000000000000000000000000000000000000000");
  }
}