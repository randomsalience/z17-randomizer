use crate::{Patcher, Result};
use rom::files::msgbn::{MsgBnOwned, add_color};

pub mod msbf;

pub fn patch_msbp(patcher: &mut Patcher) -> Result<()> {
    let boot = patcher.language(None).unwrap();
    let msbp = boot
        .open_raw("World/Message/CTRJack.msbp")?
        .try_map(|file| {
            let mut msbp = MsgBnOwned::from_file(file, b"MsgPrjBn")?;
            add_color(&mut msbp, "Plum", [0xAF, 0x99, 0xEF, 0xFF]);
            add_color(&mut msbp, "Slate", [0x6D, 0x8B, 0xE8, 0xFF]);
            Ok(msbp.to_file())
        })?;
    boot.update(msbp)?;
    Ok(())
}
