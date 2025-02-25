mod hooks;
mod rom;
mod save;

use crate::game;

const MATCH_TYPES: &[usize] = &[2, 2];

struct EXE4RSImpl;
pub const EXE4RS: &'static (dyn game::Game + Send + Sync) = &EXE4RSImpl {};

impl game::Game for EXE4RSImpl {
    fn rom_code_and_revision(&self) -> (&[u8; 4], u8) {
        (b"B4WJ", 0x01)
    }

    fn family_and_variant(&self) -> (&str, u8) {
        ("exe4", 0)
    }

    fn language(&self) -> unic_langid::LanguageIdentifier {
        unic_langid::langid!("ja-JP")
    }

    fn expected_crc32(&self) -> u32 {
        0xcf0e8b05
    }

    fn match_types(&self) -> &[usize] {
        MATCH_TYPES
    }

    fn hooks(&self) -> &'static (dyn game::Hooks + Send + Sync) {
        &hooks::B4WJ_01
    }

    fn parse_save(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        let save = save::Save::new(data)?;
        let game_info = save.game_info();
        if game_info.variant != save::Variant::RedSun
            || (game_info.region != save::Region::JP && game_info.region != save::Region::Any)
        {
            anyhow::bail!("save is not compatible: got {:?}", game_info);
        }
        Ok(Box::new(save))
    }

    fn save_from_wram(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        Ok(Box::new(save::Save::from_wram(
            data,
            save::GameInfo {
                region: save::Region::JP,
                variant: save::Variant::RedSun,
            },
        )?))
    }

    fn load_rom_assets(
        &self,
        rom: &[u8],
        wram: &[u8],
        overrides: &crate::rom::Overrides,
    ) -> Result<Box<dyn crate::rom::Assets + Send + Sync>, anyhow::Error> {
        Ok(Box::new(rom::Assets::new(
            &rom::B4WJ_01,
            overrides
                .language
                .as_ref()
                .and_then(|lang| rom::modcards::for_language(lang))
                .unwrap_or(&rom::modcards::JA_MODCARDS),
            overrides
                .charset
                .as_ref()
                .cloned()
                .unwrap_or_else(|| rom::JA_CHARSET.iter().map(|s| s.to_string()).collect()),
            rom.to_vec(),
            wram.to_vec(),
        )))
    }
}

struct EXE4BMImpl;
pub const EXE4BM: &'static (dyn game::Game + Send + Sync) = &EXE4BMImpl {};

impl game::Game for EXE4BMImpl {
    fn rom_code_and_revision(&self) -> (&[u8; 4], u8) {
        (b"B4BJ", 0x00)
    }

    fn family_and_variant(&self) -> (&str, u8) {
        ("exe4", 1)
    }

    fn language(&self) -> unic_langid::LanguageIdentifier {
        unic_langid::langid!("ja-JP")
    }

    fn expected_crc32(&self) -> u32 {
        0xed7c5b50
    }

    fn match_types(&self) -> &[usize] {
        MATCH_TYPES
    }

    fn hooks(&self) -> &'static (dyn game::Hooks + Send + Sync) {
        &hooks::B4BJ_00
    }

    fn parse_save(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        let save = save::Save::new(data)?;
        let game_info = save.game_info();
        if game_info.variant != save::Variant::BlueMoon
            || (game_info.region != save::Region::JP && game_info.region != save::Region::Any)
        {
            anyhow::bail!("save is not compatible: got {:?}", game_info);
        }
        Ok(Box::new(save))
    }

    fn save_from_wram(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        Ok(Box::new(save::Save::from_wram(
            data,
            save::GameInfo {
                region: save::Region::JP,
                variant: save::Variant::BlueMoon,
            },
        )?))
    }

    fn load_rom_assets(
        &self,
        rom: &[u8],
        wram: &[u8],
        overrides: &crate::rom::Overrides,
    ) -> Result<Box<dyn crate::rom::Assets + Send + Sync>, anyhow::Error> {
        Ok(Box::new(rom::Assets::new(
            &rom::B4BJ_00,
            overrides
                .language
                .as_ref()
                .and_then(|lang| rom::modcards::for_language(lang))
                .unwrap_or(&rom::modcards::JA_MODCARDS),
            overrides
                .charset
                .as_ref()
                .cloned()
                .unwrap_or_else(|| rom::JA_CHARSET.iter().map(|s| s.to_string()).collect()),
            rom.to_vec(),
            wram.to_vec(),
        )))
    }
}

struct BN4RSImpl;
pub const BN4RS: &'static (dyn game::Game + Send + Sync) = &BN4RSImpl {};

impl game::Game for BN4RSImpl {
    fn rom_code_and_revision(&self) -> (&[u8; 4], u8) {
        (b"B4WE", 0x00)
    }

    fn family_and_variant(&self) -> (&str, u8) {
        ("bn4", 0)
    }

    fn language(&self) -> unic_langid::LanguageIdentifier {
        unic_langid::langid!("en-US")
    }

    fn expected_crc32(&self) -> u32 {
        0x2120695c
    }

    fn match_types(&self) -> &[usize] {
        MATCH_TYPES
    }

    fn hooks(&self) -> &'static (dyn game::Hooks + Send + Sync) {
        &hooks::B4WE_00
    }

    fn parse_save(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        let save = save::Save::new(data)?;
        let game_info = save.game_info();
        if game_info.variant != save::Variant::RedSun
            || (game_info.region != save::Region::US && game_info.region != save::Region::Any)
        {
            anyhow::bail!("save is not compatible: got {:?}", game_info);
        }
        Ok(Box::new(save))
    }

    fn save_from_wram(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        Ok(Box::new(save::Save::from_wram(
            data,
            save::GameInfo {
                region: save::Region::US,
                variant: save::Variant::RedSun,
            },
        )?))
    }

    fn load_rom_assets(
        &self,
        rom: &[u8],
        wram: &[u8],
        overrides: &crate::rom::Overrides,
    ) -> Result<Box<dyn crate::rom::Assets + Send + Sync>, anyhow::Error> {
        Ok(Box::new(rom::Assets::new(
            &rom::B4WE_00,
            overrides
                .language
                .as_ref()
                .and_then(|lang| rom::modcards::for_language(lang))
                .unwrap_or(&rom::modcards::EN_MODCARDS),
            overrides
                .charset
                .as_ref()
                .cloned()
                .unwrap_or_else(|| rom::EN_CHARSET.iter().map(|s| s.to_string()).collect()),
            rom.to_vec(),
            wram.to_vec(),
        )))
    }
}

struct BN4BMImpl;
pub const BN4BM: &'static (dyn game::Game + Send + Sync) = &BN4BMImpl {};

impl game::Game for BN4BMImpl {
    fn rom_code_and_revision(&self) -> (&[u8; 4], u8) {
        (b"B4BE", 0x00)
    }

    fn family_and_variant(&self) -> (&str, u8) {
        ("bn4", 1)
    }

    fn language(&self) -> unic_langid::LanguageIdentifier {
        unic_langid::langid!("en-US")
    }

    fn expected_crc32(&self) -> u32 {
        0x758a46e9
    }

    fn match_types(&self) -> &[usize] {
        MATCH_TYPES
    }

    fn hooks(&self) -> &'static (dyn game::Hooks + Send + Sync) {
        &hooks::B4BE_00
    }

    fn parse_save(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        let save = save::Save::new(data)?;
        let game_info = save.game_info();
        if game_info.variant != save::Variant::BlueMoon
            || (game_info.region != save::Region::US && game_info.region != save::Region::Any)
        {
            anyhow::bail!("save is not compatible: got {:?}", game_info);
        }
        Ok(Box::new(save))
    }

    fn save_from_wram(&self, data: &[u8]) -> Result<Box<dyn crate::save::Save + Send + Sync>, anyhow::Error> {
        Ok(Box::new(save::Save::from_wram(
            data,
            save::GameInfo {
                region: save::Region::US,
                variant: save::Variant::BlueMoon,
            },
        )?))
    }

    fn load_rom_assets(
        &self,
        rom: &[u8],
        wram: &[u8],
        overrides: &crate::rom::Overrides,
    ) -> Result<Box<dyn crate::rom::Assets + Send + Sync>, anyhow::Error> {
        Ok(Box::new(rom::Assets::new(
            &rom::B4BE_00,
            overrides
                .language
                .as_ref()
                .and_then(|lang| rom::modcards::for_language(lang))
                .unwrap_or(&rom::modcards::EN_MODCARDS),
            overrides
                .charset
                .as_ref()
                .cloned()
                .unwrap_or_else(|| rom::EN_CHARSET.iter().map(|s| s.to_string()).collect()),
            rom.to_vec(),
            wram.to_vec(),
        )))
    }
}
