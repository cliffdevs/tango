use byteorder::ByteOrder;

use crate::save;

const SRAM_SIZE: usize = 0x2308;
const GAME_NAME_OFFSET: usize = 0x03fc;
const CHECKSUM_OFFSET: usize = 0x03f0;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Region {
    US,
    JP,
}

#[derive(PartialEq, Debug)]
pub struct GameInfo {
    pub region: Region,
}

pub struct Save {
    buf: Vec<u8>,
}

impl Save {
    pub fn new(buf: &[u8]) -> Result<Self, anyhow::Error> {
        let buf = buf
            .get(..SRAM_SIZE)
            .map(|buf| buf.to_vec())
            .ok_or(anyhow::anyhow!("save is wrong size"))?;

        let save = Self { buf };
        save.game_info()?;

        let computed_checksum = save.compute_checksum();
        if save.checksum() != computed_checksum {
            anyhow::bail!(
                "checksum mismatch: expected {:08x}, got {:08x}",
                save.checksum(),
                computed_checksum
            );
        }

        Ok(save)
    }

    pub fn game_info(&self) -> Result<GameInfo, anyhow::Error> {
        Ok(match &self.buf[GAME_NAME_OFFSET..GAME_NAME_OFFSET + 20] {
            b"ROCKMAN EXE 20010120" => GameInfo { region: Region::JP },
            b"ROCKMAN EXE 20010727" => GameInfo { region: Region::US },
            n => {
                anyhow::bail!("unknown game name: {:02x?}", n);
            }
        })
    }

    pub fn checksum(&self) -> u32 {
        byteorder::LittleEndian::read_u32(&self.buf[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4])
    }

    pub fn compute_checksum(&self) -> u32 {
        save::compute_save_raw_checksum(&self.buf, CHECKSUM_OFFSET) + 0x16
    }
}

impl save::Save for Save {
    fn view_chips<'a>(&'a self) -> Option<Box<dyn save::ChipsView<'a> + 'a>> {
        Some(Box::new(ChipsView { save: self }))
    }
}

pub struct ChipsView<'a> {
    save: &'a Save,
}

impl<'a> save::ChipsView<'a> for ChipsView<'a> {
    fn chip_codes(&self) -> &'static [u8] {
        &b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"[..]
    }

    fn num_folders(&self) -> usize {
        1
    }

    fn equipped_folder_index(&self) -> usize {
        0
    }

    fn regular_chip_is_in_place(&self) -> bool {
        false
    }

    fn regular_chip_index(&self, _folder_index: usize) -> Option<usize> {
        None
    }

    fn tag_chip_indexes(&self, _folder_index: usize) -> Option<(usize, usize)> {
        None
    }

    fn chip(&self, folder_index: usize, chip_index: usize) -> Option<save::Chip> {
        if folder_index > 0 || chip_index > 30 {
            return None;
        }

        Some(save::Chip {
            id: self.save.buf[0x01c0 + chip_index * 2] as usize,
            variant: self.save.buf[0x01c0 + chip_index * 2 + 1] as usize,
        })
    }
}
