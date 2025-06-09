use std::fs::File;
use std::io::Write;



pub fn save_key(agent_id: u64, encrypted_key: &[u8]) -> std::io::Result<()> {
    let filename = format!("keys/agent_{agent_id}_key.bin");
    let mut file = File::create(filename)?;
    file.write_all(encrypted_key)?;
    Ok(())
}