use std::io::{Result, Write};

use blockchain::Block;
use util::write_u64_to;

impl Block {
    /// Serializes the `Block` to a `Write`.
    pub fn write_to<W: Write>(&self, mut w: W) -> Result<()> {
        write_u64_to(self.index, &mut w)?;
        w.write_all(&self.prev_hash.0)?;
        write_u64_to(self.timestamp, &mut w)?;

        let l = self.data.len();
        assert!(l < 256);
        w.write_all(&[l as u8])?;
        w.write_all(&self.data)?;

        w.write_all(&self.hash.0)
    }
}
