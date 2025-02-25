use byteorder::ByteOrder;

use crate::rom;

pub struct Offsets {
    chip_data: u32,
    chip_names_pointers: u32,
    chip_descriptions_pointers: u32,
    chip_icon_palette_pointer: u32,
    ncp_data: u32,
    ncp_names_pointer: u32,
    ncp_descriptions_pointer: u32,
    element_icon_palette_pointer: u32,
    element_icons_pointer: u32,
    modcard_data: u32,
    modcard_names_pointer: u32,
    modcard_details_names_pointer: u32,
    navicust_bg: image::Rgba<u8>,
}

const NAVICUST_BG_G: image::Rgba<u8> = image::Rgba([0x08, 0xbd, 0x73, 0xff]);
const NAVICUST_BG_F: image::Rgba<u8> = image::Rgba([0xe7, 0x8c, 0x39, 0xff]);

#[rustfmt::skip]
pub static BR5J_00: Offsets = Offsets {
    chip_data:                      0x080221bc,
    chip_names_pointers:            0x08043274,
    chip_descriptions_pointers:     0x08028164,
    chip_icon_palette_pointer:      0x0801f144,
    ncp_data:                       0x081460cc,
    ncp_names_pointer:              0x08043284,
    ncp_descriptions_pointer:       0x08139240,
    element_icon_palette_pointer:   0x081226e4,
    element_icons_pointer:          0x081226dc,
    modcard_data:                   0x08144778,
    modcard_names_pointer:          0x08130fe0,
    modcard_details_names_pointer:  0x08130fec,

    navicust_bg: NAVICUST_BG_G,
};

#[rustfmt::skip]
pub static BR6J_00: Offsets = Offsets {
    chip_data:                      0x080221bc,
    chip_names_pointers:            0x080432a4,
    chip_descriptions_pointers:     0x08028164,
    chip_icon_palette_pointer:      0x0801f144,
    ncp_data:                       0x08144300,
    ncp_names_pointer:              0x080432b4,
    ncp_descriptions_pointer:       0x08137478,
    element_icon_palette_pointer:   0x081213c4,
    element_icons_pointer:          0x081213bc,
    modcard_data:                   0x081429b0,
    modcard_names_pointer:          0x0812f218,
    modcard_details_names_pointer:  0x0812f224,

    navicust_bg: NAVICUST_BG_F,
};

#[rustfmt::skip]
pub static BR5E_00: Offsets = Offsets {
    chip_data:                      0x08021da8,
    chip_names_pointers:            0x08042038,
    chip_descriptions_pointers:     0x08027d50,
    chip_icon_palette_pointer:      0x0801ed20,
    ncp_data:                       0x0813b22c,
    ncp_names_pointer:              0x08042048,
    ncp_descriptions_pointer:       0x08130878,
    element_icon_palette_pointer:   0x0811a9a4,
    element_icons_pointer:          0x0811a99c,
    modcard_data:                   0,
    modcard_names_pointer:          0,
    modcard_details_names_pointer:  0,

    navicust_bg: NAVICUST_BG_G,
};

#[rustfmt::skip]
pub static BR6E_00: Offsets = Offsets {
    chip_data:                      0x08021da8,
    chip_names_pointers:            0x08042068,
    chip_descriptions_pointers:     0x08027d50,
    chip_icon_palette_pointer:      0x0801ed20,
    ncp_data:                       0x0813944c,
    ncp_names_pointer:              0x08042078,
    ncp_descriptions_pointer:       0x0812ea9c,
    element_icon_palette_pointer:   0x08119674,
    element_icons_pointer:          0x0811966c,
    modcard_data:                   0,
    modcard_names_pointer:          0,
    modcard_details_names_pointer:  0,

    navicust_bg: NAVICUST_BG_F,
};

const PRINT_VAR_COMMAND: u8 = 0xfa;
const EREADER_COMMAND: u8 = 0xff;

pub struct Assets {
    offsets: &'static Offsets,
    overrides: rom::Overrides,
    text_parse_options: rom::text::ParseOptions,
    mapper: rom::MemoryMapper,
    chip_icon_palette: [image::Rgba<u8>; 16],
    element_icon_palette: [image::Rgba<u8>; 16],
}

struct Chip<'a> {
    id: usize,
    assets: &'a Assets,
}

impl<'a> Chip<'a> {
    fn raw_info(&'a self) -> [u8; 0x2c] {
        self.assets.mapper.get(self.assets.offsets.chip_data)[self.id * 0x2c..(self.id + 1) * 0x2c]
            .try_into()
            .unwrap()
    }
}

impl<'a> rom::Chip for Chip<'a> {
    fn name(&self) -> String {
        if let Some(chips) = self.assets.overrides.chips.as_ref() {
            return chips
                .get(self.id)
                .and_then(|chip| chip.name.clone())
                .unwrap_or_else(|| "???".to_string());
        }

        let pointer = self.assets.offsets.chip_names_pointers + ((self.id / 0x100) * 4) as u32;
        let id = self.id % 0x100;

        if let Ok(parts) = rom::text::parse_entry(
            &self
                .assets
                .mapper
                .get(byteorder::LittleEndian::read_u32(&self.assets.mapper.get(pointer)[..4])),
            id,
            &self.assets.text_parse_options,
        ) {
            parts
                .into_iter()
                .flat_map(|part| {
                    match part {
                        rom::text::Part::String(s) => s,
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>()
        } else {
            "???".to_string()
        }
    }

    fn description(&self) -> String {
        if let Some(chips) = self.assets.overrides.chips.as_ref() {
            return chips
                .get(self.id)
                .and_then(|chip| chip.description.clone())
                .unwrap_or_else(|| "???".to_string());
        }

        let pointer = self.assets.offsets.chip_descriptions_pointers + ((self.id / 0x100) * 4) as u32;
        let id = self.id % 0x100;

        if let Ok(parts) = rom::text::parse_entry(
            &self
                .assets
                .mapper
                .get(byteorder::LittleEndian::read_u32(&self.assets.mapper.get(pointer)[..4])),
            id,
            &self.assets.text_parse_options,
        ) {
            parts
                .into_iter()
                .flat_map(|part| {
                    match part {
                        rom::text::Part::String(s) => s,
                        rom::text::Part::Command {
                            op: EREADER_COMMAND,
                            params,
                        } => {
                            if let Ok(parts) = rom::text::parse(
                                &self.assets.mapper.get(0x020007d6 + params[1] as u32 * 100),
                                &self.assets.text_parse_options,
                            ) {
                                parts
                                    .into_iter()
                                    .flat_map(|part| {
                                        match part {
                                            rom::text::Part::String(s) => s,
                                            _ => "".to_string(),
                                        }
                                        .chars()
                                        .collect::<Vec<_>>()
                                    })
                                    .collect::<String>()
                            } else {
                                "???".to_string()
                            }
                        }
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>()
        } else {
            "???".to_string()
        }
    }

    fn icon(&self) -> image::RgbaImage {
        let raw = self.raw_info();
        rom::apply_palette(
            rom::read_merged_tiles(
                &self
                    .assets
                    .mapper
                    .get(byteorder::LittleEndian::read_u32(&raw[0x20..0x20 + 4]))[..rom::TILE_BYTES * 4],
                2,
            )
            .unwrap(),
            &self.assets.chip_icon_palette,
        )
    }

    fn image(&self) -> image::RgbaImage {
        let raw = self.raw_info();
        rom::apply_palette(
            rom::read_merged_tiles(
                &self
                    .assets
                    .mapper
                    .get(byteorder::LittleEndian::read_u32(&raw[0x24..0x24 + 4]))[..rom::TILE_BYTES * 7 * 6],
                7,
            )
            .unwrap(),
            &rom::read_palette(
                &self
                    .assets
                    .mapper
                    .get(byteorder::LittleEndian::read_u32(&raw[0x28..0x28 + 4]))[..32],
            ),
        )
    }

    fn codes(&self) -> Vec<char> {
        let raw = self.raw_info();
        raw[0x00..0x04]
            .iter()
            .cloned()
            .filter(|code| *code != 0xff)
            .map(|code| b"ABCDEFGHIJKLMNOPQRSTUVWXYZ*"[code as usize] as char)
            .collect()
    }

    fn element(&self) -> usize {
        let raw = self.raw_info();
        raw[0x06] as usize
    }

    fn class(&self) -> rom::ChipClass {
        let raw = self.raw_info();
        [
            rom::ChipClass::Standard,
            rom::ChipClass::Mega,
            rom::ChipClass::Giga,
            rom::ChipClass::None,
            rom::ChipClass::ProgramAdvance,
        ][raw[0x07] as usize]
    }

    fn dark(&self) -> bool {
        false
    }

    fn mb(&self) -> u8 {
        let raw = self.raw_info();
        raw[0x08]
    }

    fn damage(&self) -> u32 {
        let raw = self.raw_info();
        let damage = byteorder::LittleEndian::read_u16(&raw[0x1a..0x1a + 2]) as u32;
        if damage < 1000 {
            damage
        } else {
            0
        }
    }
}

struct NavicustPart<'a> {
    id: usize,
    variant: usize,
    assets: &'a Assets,
}

impl<'a> NavicustPart<'a> {
    fn raw_info(&'a self) -> [u8; 0x10] {
        let i = self.id * 4 + self.variant;
        self.assets.mapper.get(self.assets.offsets.ncp_data)[i * 0x10..(i + 1) * 0x10]
            .try_into()
            .unwrap()
    }
}

impl<'a> rom::NavicustPart for NavicustPart<'a> {
    fn name(&self) -> String {
        if let Some(navicust_parts) = self.assets.overrides.navicust_parts.as_ref() {
            return navicust_parts
                .get(self.id)
                .and_then(|ncp| ncp.name.clone())
                .unwrap_or_else(|| "???".to_string());
        }

        if let Ok(parts) = rom::text::parse_entry(
            &self.assets.mapper.get(byteorder::LittleEndian::read_u32(
                &self.assets.mapper.get(self.assets.offsets.ncp_names_pointer)[..4],
            )),
            self.id,
            &self.assets.text_parse_options,
        ) {
            parts
                .into_iter()
                .flat_map(|part| {
                    match &part {
                        rom::text::Part::String(s) => s,
                        _ => "",
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>()
        } else {
            "???".to_string()
        }
    }

    fn description(&self) -> String {
        if let Some(navicust_parts) = self.assets.overrides.navicust_parts.as_ref() {
            return navicust_parts
                .get(self.id)
                .and_then(|ncp| ncp.description.clone())
                .unwrap_or_else(|| "???".to_string());
        }

        if let Ok(parts) = rom::text::parse_entry(
            &self.assets.mapper.get(byteorder::LittleEndian::read_u32(
                &self.assets.mapper.get(self.assets.offsets.ncp_descriptions_pointer)[..4],
            )),
            self.id,
            &self.assets.text_parse_options,
        ) {
            parts
                .into_iter()
                .flat_map(|part| {
                    match part {
                        rom::text::Part::String(s) => s,
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>()
        } else {
            "???".to_string()
        }
    }

    fn color(&self) -> Option<rom::NavicustPartColor> {
        let raw = self.raw_info();
        Some(match raw[0x03] {
            1 => rom::NavicustPartColor::White,
            2 => rom::NavicustPartColor::Yellow,
            3 => rom::NavicustPartColor::Pink,
            4 => rom::NavicustPartColor::Red,
            5 => rom::NavicustPartColor::Blue,
            6 => rom::NavicustPartColor::Green,
            _ => {
                return None;
            }
        })
    }

    fn is_solid(&self) -> bool {
        let raw = self.raw_info();
        raw[0x01] == 0
    }

    fn uncompressed_bitmap(&self) -> rom::NavicustBitmap {
        let raw = self.raw_info();
        image::ImageBuffer::from_vec(
            7,
            7,
            self.assets
                .mapper
                .get(byteorder::LittleEndian::read_u32(&raw[0x08..0x0c]))[..49]
                .to_vec(),
        )
        .unwrap()
    }

    fn compressed_bitmap(&self) -> rom::NavicustBitmap {
        let raw = self.raw_info();
        image::ImageBuffer::from_vec(
            7,
            7,
            self.assets
                .mapper
                .get(byteorder::LittleEndian::read_u32(&raw[0x0c..0x10]))[..49]
                .to_vec(),
        )
        .unwrap()
    }
}

impl Assets {
    pub fn new(
        offsets: &'static Offsets,
        rom: Vec<u8>,
        wram: Vec<u8>,
        default_charset: &[&str],
        overrides: &crate::rom::Overrides,
    ) -> Self {
        let mapper = rom::MemoryMapper::new(rom, wram);

        let chip_icon_palette = rom::read_palette(
            &mapper.get(byteorder::LittleEndian::read_u32(
                &mapper.get(offsets.chip_icon_palette_pointer)[..4],
            ))[..32],
        );

        let element_icon_palette = rom::read_palette(
            &mapper.get(byteorder::LittleEndian::read_u32(
                &mapper.get(offsets.element_icon_palette_pointer)[..4],
            ))[..32],
        );

        Self {
            offsets,
            overrides: overrides.clone(),
            text_parse_options: rom::text::ParseOptions {
                charset: overrides
                    .charset
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| default_charset.iter().map(|s| s.to_string()).collect()),
                extension_ops: 0xe4..=0xe4,
                eof_op: 0xe6,
                newline_op: 0xe9,
                commands: std::collections::HashMap::from([
                    (PRINT_VAR_COMMAND, 3),
                    (EREADER_COMMAND, 2),
                    (0xe7, 1),
                    (0xe8, 3),
                    (0xee, 3),
                    (0xf1, 2),
                ]),
            },
            mapper,
            chip_icon_palette,
            element_icon_palette,
        }
    }
}

struct Modcard56<'a> {
    id: usize,
    assets: &'a Assets,
}

impl<'a> Modcard56<'a> {
    pub fn raw_info(&self) -> Vec<u8> {
        let buf = self.assets.mapper.get(self.assets.offsets.modcard_data);
        buf[byteorder::LittleEndian::read_u16(&buf[self.id * 2..(self.id + 1) * 2]) as usize
            ..byteorder::LittleEndian::read_u16(&buf[(self.id + 1) * 2..(self.id + 2) * 2]) as usize]
            .to_vec()
    }
}

impl<'a> rom::Modcard56 for Modcard56<'a> {
    fn name(&self) -> String {
        if self.id == 0 {
            return "".to_string();
        }

        if let Some(modcard56s) = self.assets.overrides.modcard56s.as_ref() {
            return modcard56s
                .get(self.id)
                .and_then(|modcard56| modcard56.name.clone())
                .unwrap_or_else(|| "???".to_string());
        }

        if let Ok(parts) = rom::text::parse_entry(
            &self.assets.mapper.get(byteorder::LittleEndian::read_u32(
                &self.assets.mapper.get(self.assets.offsets.modcard_names_pointer)[..4],
            )),
            self.id,
            &self.assets.text_parse_options,
        ) {
            parts
                .into_iter()
                .flat_map(|part| {
                    match part {
                        rom::text::Part::String(s) => s,
                        _ => "".to_string(),
                    }
                    .chars()
                    .collect::<Vec<_>>()
                })
                .collect::<String>()
        } else {
            "???".to_string()
        }
    }

    fn mb(&self) -> u8 {
        if self.id == 0 {
            return 0;
        }

        let raw = self.raw_info();
        raw[1]
    }

    fn effects(&self) -> Vec<rom::Modcard56Effect> {
        if self.id == 0 {
            return vec![];
        }

        let raw = self.raw_info();
        raw[3..]
            .chunks(3)
            .map(|chunk| {
                let id = chunk[0];
                let parameter = chunk[1];
                rom::Modcard56Effect {
                    id,
                    name: if let Some(modcard56_effects) = self.assets.overrides.modcard56_effects.as_ref() {
                        modcard56_effects
                            .get(id as usize)
                            .and_then(|effect| effect.name_template.clone())
                            .unwrap_or_else(|| vec![rom::Modcard56EffectTemplatePart::String("???".to_string())])
                    } else {
                        if let Ok(parts) = rom::text::parse_entry(
                            &self.assets.mapper.get(byteorder::LittleEndian::read_u32(
                                &self
                                    .assets
                                    .mapper
                                    .get(self.assets.offsets.modcard_details_names_pointer)[..4],
                            )),
                            id as usize,
                            &self.assets.text_parse_options,
                        ) {
                            rom::text::parse_modcard56_effect(parts, PRINT_VAR_COMMAND)
                        } else {
                            vec![rom::Modcard56EffectTemplatePart::String("???".to_string())]
                        }
                    }
                    .into_iter()
                    .flat_map(|p| {
                        match p {
                            rom::Modcard56EffectTemplatePart::String(s) => s,
                            rom::Modcard56EffectTemplatePart::PrintVar(v) => {
                                if v == 1 {
                                    let mut parameter = parameter as u32;
                                    if id == 0x00 || id == 0x02 {
                                        parameter = parameter * 10;
                                    }
                                    format!("{}", parameter)
                                } else {
                                    "".to_string()
                                }
                            }
                        }
                        .chars()
                        .collect::<Vec<_>>()
                    })
                    .collect(),
                    parameter,
                    is_debuff: chunk[2] == 1,
                    is_ability: id > 0x15,
                }
            })
            .collect::<Vec<_>>()
    }
}

impl rom::Assets for Assets {
    fn chip<'a>(&'a self, id: usize) -> Option<Box<dyn rom::Chip + 'a>> {
        if id >= self.num_chips() {
            return None;
        }
        Some(Box::new(Chip { id, assets: self }))
    }

    fn num_chips(&self) -> usize {
        411
    }

    fn element_icon(&self, id: usize) -> Option<image::RgbaImage> {
        if id >= 11 {
            return None;
        }

        let buf = self.mapper.get(byteorder::LittleEndian::read_u32(
            &self.mapper.get(self.offsets.element_icons_pointer)[..4],
        ));
        Some(rom::apply_palette(
            rom::read_merged_tiles(&buf[id * rom::TILE_BYTES * 4..(id + 1) * rom::TILE_BYTES * 4], 2).unwrap(),
            &self.element_icon_palette,
        ))
    }

    fn navicust_part<'a>(&'a self, id: usize, variant: usize) -> Option<Box<dyn rom::NavicustPart + 'a>> {
        let (max_id, max_variant) = self.num_navicust_parts();
        if id >= max_id || variant >= max_variant {
            return None;
        }
        Some(Box::new(NavicustPart {
            id,
            variant,
            assets: self,
        }))
    }

    fn num_navicust_parts(&self) -> (usize, usize) {
        (47, 4)
    }

    fn navicust_bg(&self) -> Option<image::Rgba<u8>> {
        Some(self.offsets.navicust_bg)
    }

    fn modcard56<'a>(&'a self, id: usize) -> Option<Box<dyn rom::Modcard56 + 'a>> {
        if id >= self.num_modcard56s() {
            return None;
        }
        Some(Box::new(Modcard56 { id, assets: self }))
    }

    fn num_modcard56s(&self) -> usize {
        118
    }
}

#[rustfmt::skip]
pub const EN_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "*", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "RV", "BX", "EX", "SP", "FZ", "ウ", "ア", "イ", "オ", "エ", "ケ", "コ", "カ", "ク", "キ", "セ", "サ", "ソ", "シ", "ス", "テ", "ト", "ツ", "タ", "チ", "ネ", "ノ", "ヌ", "ナ", "ニ", "ヒ", "ヘ", "ホ", "ハ", "フ", "ミ", "マ", "メ", "ム", "モ", "ヤ", "ヨ", "ユ", "ロ", "ル", "リ", "レ", "ラ", "ン", "熱", "斗", "ワ", "ヲ", "ギ", "ガ", "ゲ", "ゴ", "グ", "ゾ", "ジ", "ゼ", "ズ", "ザ", "デ", "ド", "ヅ", "ダ", "ヂ", "ベ", "ビ", "ボ", "バ", "ブ", "ピ", "パ", "ペ", "プ", "ポ", "ゥ", "ァ", "ィ", "ォ", "ェ", "ュ", "ヴ", "ッ", "ョ", "ャ", "-", "×", "=", ":", "%", "?", "+", "█", "[bat]", "ー", "!", "&", ",", "゜", ".", "・", ";", "'", "\"", "~", "/", "(", ")", "「", "」", "�", "_", "ƶ", "[L]", "[B]", "[R]", "[A]", "あ", "い", "け", "く", "き", "こ", "か", "せ", "そ", "す", "さ", "し", "つ", "と", "て", "た", "ち", "ね", "の", "な", "ぬ", "に", "へ", "ふ", "ほ", "は", "ひ", "め", "む", "み", "も", "ま", "ゆ", "よ", "や", "る", "ら", "り", "ろ", "れ", "[END]", "ん", "を", "わ", "研", "げ", "ぐ", "ご", "が", "ぎ", "ぜ", "ず", "じ", "ぞ", "ざ", "で", "ど", "づ", "だ", "ぢ", "べ", "ば", "び", "ぼ", "ぶ", "ぽ", "ぷ", "ぴ", "ぺ", "ぱ", "ぅ", "ぁ", "ぃ", "ぉ", "ぇ", "ゅ", "ょ", "っ", "ゃ", "容", "量", "全", "木", "[MB]", "無", "現", "実", "[circle]", "×", "緑", "道", "不", "止", "彩", "起", "父", "集", "院", "一", "二", "三", "四", "五", "六", "七", "八", "陽", "十", "百", "千", "万", "脳", "上", "下", "左", "右", "手", "来", "日", "目", "月", "獣", "各", "人", "入", "出", "山", "口", "光", "電", "気", "綾", "科", "次", "名", "前", "学", "校", "省", "祐", "室", "世", "界", "高", "朗", "枚", "野", "悪", "路", "闇", "大", "小", "中", "自", "分", "間", "系", "花", "問", "究", "門", "城", "王", "兄", "化", "葉", "行", "街", "屋", "水", "見", "終", "新", "桜", "先", "生", "長", "今", "了", "点", "井", "子", "言", "太", "属", "風", "会", "性", "持", "時", "勝", "赤", "代", "年", "火", "改", "計", "画", "職", "体", "波", "回", "外", "地", "員", "正", "造", "値", "合", "戦", "川", "秋", "原", "町", "晴", "用", "金", "郎", "作", "数", "方", "社", "攻", "撃", "力", "同", "武", "何", "発", "少", "教", "以", "白", "早", "暮", "面", "組", "後", "文", "字", "本", "階", "明", "才", "者", "向", "犬", "々", "ヶ", "連", "射", "舟", "戸", "切", "土", "炎", "伊", "夫", "鉄", "国", "男", "天", "老", "師", "堀", "杉", "士", "悟", "森", "霧", "麻", "剛", "垣", "★", "[bracket1]", "[bracket2]", "[.]"];

#[rustfmt::skip]
pub const JA_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "ウ", "ア", "イ", "オ", "エ", "ケ", "コ", "カ", "ク", "キ", "セ", "サ", "ソ", "シ", "ス", "テ", "ト", "ツ", "タ", "チ", "ネ", "ノ", "ヌ", "ナ", "ニ", "ヒ", "ヘ", "ホ", "ハ", "フ", "ミ", "マ", "メ", "ム", "モ", "ヤ", "ヨ", "ユ", "ロ", "ル", "リ", "レ", "ラ", "ン", "熱", "斗", "ワ", "ヲ", "ギ", "ガ", "ゲ", "ゴ", "グ", "ゾ", "ジ", "ゼ", "ズ", "ザ", "デ", "ド", "ヅ", "ダ", "ヂ", "ベ", "ビ", "ボ", "バ", "ブ", "ピ", "パ", "ペ", "プ", "ポ", "ゥ", "ァ", "ィ", "ォ", "ェ", "ュ", "ヴ", "ッ", "ョ", "ャ", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "*", "-", "×", "=", ":", "%", "?", "+", "■", "[bat]", "ー", "!", "RV", "BX", "&", "、", "゜", ".", "・", ";", "’", "\"", "~", "/", "(", ")", "「", "」", "EX", "SP", "FZ", "�", "_", "ƶ", "周", "え", "お", "う", "あ", "い", "け", "く", "き", "こ", "か", "せ", "そ", "す", "さ", "し", "つ", "と", "て", "た", "ち", "ね", "の", "な", "ぬ", "に", "へ", "ふ", "ほ", "は", "ひ", "め", "む", "み", "も", "ま", "ゆ", "よ", "や", "る", "ら", "り", "ろ", "れ", "[END]", "ん", "を", "わ", "研", "げ", "ぐ", "ご", "が", "ぎ", "ぜ", "ず", "じ", "ぞ", "ざ", "で", "ど", "づ", "だ", "ぢ", "べ", "ば", "び", "ぼ", "ぶ", "ぽ", "ぷ", "ぴ", "ぺ", "ぱ", "ぅ", "ぁ", "ぃ", "ぉ", "ぇ", "ゅ", "ょ", "っ", "ゃ", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "容", "量", "全", "木", "[MB]", "無", "現", "実", "[circle]", "×", "緑", "道", "不", "止", "彩", "起", "父", "集", "院", "一", "二", "三", "四", "五", "六", "七", "八", "陽", "十", "百", "千", "万", "脳", "上", "下", "左", "右", "手", "来", "日", "目", "月", "獣", "各", "人", "入", "出", "山", "口", "光", "電", "気", "綾", "科", "次", "名", "前", "学", "校", "省", "祐", "室", "世", "界", "高", "朗", "枚", "野", "悪", "路", "闇", "大", "小", "中", "自", "分", "間", "系", "花", "問", "究", "門", "城", "王", "兄", "化", "葉", "行", "街", "屋", "水", "見", "終", "新", "桜", "先", "生", "長", "今", "了", "点", "井", "子", "言", "太", "属", "風", "会", "性", "持", "時", "勝", "赤", "代", "年", "火", "改", "計", "画", "職", "体", "波", "回", "外", "地", "員", "正", "造", "値", "合", "戦", "川", "秋", "原", "町", "晴", "用", "金", "郎", "作", "数", "方", "社", "攻", "撃", "力", "同", "武", "何", "発", "少", "教", "以", "白", "早", "暮", "面", "組", "後", "文", "字", "本", "階", "明", "才", "者", "向", "犬", "々", "ヶ", "連", "射", "舟", "戸", "切", "土", "炎", "伊", "夫", "鉄", "国", "男", "天", "老", "師", "堀", "杉", "士", "悟", "森", "霧", "麻", "剛", "垣"];
