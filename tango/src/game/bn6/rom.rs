use byteorder::ByteOrder;

use crate::{patch, rom};

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
}

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
};

const PRINT_VAR_COMMAND: u8 = 0xfa;
const EREADER_COMMAND: u8 = 0xff;

pub struct Assets {
    element_icons: [image::RgbaImage; 11],
    chips: [rom::Chip; 411],
    navicust_parts: [rom::NavicustPart; 188],
    modcard56s: Option<[rom::Modcard56; 118]>,
}

impl Assets {
    pub fn new(
        offsets: &Offsets,
        rom: &[u8],
        wram: &[u8],
        default_charset: &[&str],
        overrides: &patch::ROMOverrides,
    ) -> Self {
        let override_charset = overrides
            .charset
            .as_ref()
            .map(|charset| charset.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        let text_parse_options = rom::text::ParseOptions {
            charset: if let Some(charset) = override_charset.as_ref() {
                charset
            } else {
                default_charset
            },
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
        };

        let mapper = rom::MemoryMapper::new(rom, wram);

        let chip_icon_palette = rom::read_palette(
            &mapper.get(byteorder::LittleEndian::read_u32(
                &mapper.get(offsets.chip_icon_palette_pointer)[..4],
            ))[..32],
        );

        Self {
            element_icons: {
                let palette = rom::read_palette(
                    &mapper.get(byteorder::LittleEndian::read_u32(
                        &mapper.get(offsets.element_icon_palette_pointer)[..4],
                    ))[..32],
                );
                {
                    let buf = mapper.get(byteorder::LittleEndian::read_u32(
                        &mapper.get(offsets.element_icons_pointer)[..4],
                    ));
                    (0..11)
                        .map(|i| {
                            rom::apply_palette(
                                rom::read_merged_tiles(&buf[i * rom::TILE_BYTES * 4..(i + 1) * rom::TILE_BYTES * 4], 2)
                                    .unwrap(),
                                &palette,
                            )
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap()
                }
            },
            chips: (0..411)
                .map(|i| {
                    let buf = &mapper.get(offsets.chip_data)[i * 0x2c..(i + 1) * 0x2c];
                    rom::Chip {
                        name: if let Some(chips) = overrides.chips.as_ref() {
                            chips.get(i).map(|chip| chip.name.clone()).unwrap_or("???".to_string())
                        } else {
                            let pointer = offsets.chip_names_pointers + ((i / 0x100) * 4) as u32;
                            let i = i % 0x100;

                            if let Ok(parts) = rom::text::parse_entry(
                                &mapper.get(byteorder::LittleEndian::read_u32(&mapper.get(pointer)[..4])),
                                i,
                                &text_parse_options,
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
                        },
                        description: if let Some(chips) = overrides.chips.as_ref() {
                            chips
                                .get(i)
                                .map(|chip| chip.description.clone())
                                .unwrap_or("???".to_string())
                        } else {
                            let pointer = offsets.chip_descriptions_pointers + ((i / 0x100) * 4) as u32;
                            let i = i % 0x100;

                            if let Ok(parts) = rom::text::parse_entry(
                                &mapper.get(byteorder::LittleEndian::read_u32(&mapper.get(pointer)[..4])),
                                i,
                                &text_parse_options,
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
                                                    &mapper.get(0x020007d6 + params[1] as u32 * 100),
                                                    &text_parse_options,
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
                                    .replace("-\n", "-")
                                    .replace("\n", " ")
                            } else {
                                "???".to_string()
                            }
                        },
                        icon: rom::apply_palette(
                            rom::read_merged_tiles(
                                &mapper.get(byteorder::LittleEndian::read_u32(&buf[0x20..0x20 + 4]))
                                    [..rom::TILE_BYTES * 4],
                                2,
                            )
                            .unwrap(),
                            &chip_icon_palette,
                        ),
                        image: rom::apply_palette(
                            rom::read_merged_tiles(
                                &mapper.get(byteorder::LittleEndian::read_u32(&buf[0x24..0x24 + 4]))
                                    [..rom::TILE_BYTES * 6 * 7],
                                6,
                            )
                            .unwrap(),
                            &rom::read_palette(
                                &mapper.get(byteorder::LittleEndian::read_u32(&buf[0x28..0x28 + 4]))[..32],
                            ),
                        ),
                        codes: buf[0x00..0x04].iter().cloned().filter(|code| *code != 0xff).collect(),
                        element: buf[0x06] as usize,
                        class: [
                            rom::ChipClass::Standard,
                            rom::ChipClass::Mega,
                            rom::ChipClass::Giga,
                            rom::ChipClass::None,
                            rom::ChipClass::ProgramAdvance,
                        ][buf[0x07] as usize],
                        dark: false,
                        mb: buf[0x08],
                        damage: {
                            let damage = byteorder::LittleEndian::read_u16(&buf[0x1a..0x1a + 2]) as u32;
                            if damage < 1000 {
                                damage
                            } else {
                                0
                            }
                        },
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            navicust_parts: (0..188)
                .map(|i| {
                    let buf = &mapper.get(offsets.ncp_data)[i * 0x10..(i + 1) * 0x10];
                    rom::NavicustPart {
                        name: if let Some(navicust_parts) = overrides.navicust_parts.as_ref() {
                            navicust_parts
                                .get(i / 4)
                                .map(|ncp| ncp.name.clone())
                                .unwrap_or("???".to_string())
                        } else {
                            if let Ok(parts) = rom::text::parse_entry(
                                &mapper.get(byteorder::LittleEndian::read_u32(
                                    &mapper.get(offsets.ncp_names_pointer)[..4],
                                )),
                                i / 4,
                                &text_parse_options,
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
                        },
                        description: if let Some(navicust_parts) = overrides.navicust_parts.as_ref() {
                            navicust_parts
                                .get(i / 4)
                                .map(|ncp| ncp.description.clone())
                                .unwrap_or("???".to_string())
                        } else {
                            if let Ok(parts) = rom::text::parse_entry(
                                &mapper.get(byteorder::LittleEndian::read_u32(
                                    &mapper.get(offsets.ncp_descriptions_pointer)[..4],
                                )),
                                i / 4,
                                &text_parse_options,
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
                                    .replace("-\n", "-")
                                    .replace("\n", " ")
                            } else {
                                "???".to_string()
                            }
                        },
                        color: [
                            None,
                            Some(rom::NavicustPartColor::White),
                            Some(rom::NavicustPartColor::Yellow),
                            Some(rom::NavicustPartColor::Pink),
                            Some(rom::NavicustPartColor::Red),
                            Some(rom::NavicustPartColor::Blue),
                            Some(rom::NavicustPartColor::Green),
                        ][buf[0x03] as usize]
                            .clone(),
                        is_solid: buf[0x01] == 0,
                        compressed_bitmap: image::ImageBuffer::from_vec(
                            7,
                            7,
                            mapper.get(byteorder::LittleEndian::read_u32(&buf[0x08..0x0c]))[..49].to_vec(),
                        )
                        .unwrap(),
                        uncompressed_bitmap: image::ImageBuffer::from_vec(
                            7,
                            7,
                            mapper.get(byteorder::LittleEndian::read_u32(&buf[0x0c..0x10]))[..49].to_vec(),
                        )
                        .unwrap(),
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            modcard56s: if offsets.modcard_data != 0 {
                Some(
                    [rom::Modcard56 {
                        name: "".to_string(),
                        mb: 0,
                        effects: vec![],
                    }]
                    .into_iter()
                    .chain((1..118).map(|i| {
                        let buf = mapper.get(offsets.modcard_data);
                        let buf = &buf[byteorder::LittleEndian::read_u16(&buf[i * 2..(i + 1) * 2]) as usize
                            ..byteorder::LittleEndian::read_u16(&buf[(i + 1) * 2..(i + 2) * 2]) as usize];
                        rom::Modcard56 {
                            name: if let Some(modcard56s) = overrides.modcard56s.as_ref() {
                                modcard56s
                                    .get(i)
                                    .map(|modcard| modcard.name.clone())
                                    .unwrap_or("???".to_string())
                            } else {
                                if let Ok(parts) = rom::text::parse_entry(
                                    &mapper.get(byteorder::LittleEndian::read_u32(
                                        &mapper.get(offsets.modcard_names_pointer)[..4],
                                    )),
                                    i,
                                    &text_parse_options,
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
                                        .replace("-\n", "-")
                                        .replace("\n", " ")
                                } else {
                                    "???".to_string()
                                }
                            },
                            mb: buf[1],
                            effects: buf[3..]
                                .chunks(3)
                                .map(|chunk| {
                                    let id = chunk[0];
                                    let parameter = chunk[1];
                                    rom::Modcard56Effect {
                                        id,
                                        name: if let Some(modcard56_effects) = overrides.modcard56_effects.as_ref() {
                                            modcard56_effects
                                                .get(id as usize)
                                                .map(|effect| effect.name_template.clone())
                                                .unwrap_or_else(|| {
                                                    vec![rom::Modcard56EffectTemplatePart::String("???".to_string())]
                                                })
                                        } else {
                                            if let Ok(parts) = rom::text::parse_entry(
                                                &mapper.get(byteorder::LittleEndian::read_u32(
                                                    &mapper.get(offsets.modcard_details_names_pointer)[..4],
                                                )),
                                                id as usize,
                                                &text_parse_options,
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
                                .collect::<Vec<_>>(),
                        }
                    }))
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
                )
            } else {
                None
            },
        }
    }
}

impl rom::Assets for Assets {
    fn chip(&self, id: usize) -> Option<&rom::Chip> {
        self.chips.get(id)
    }

    fn num_chips(&self) -> usize {
        self.chips.len()
    }

    fn element_icon(&self, id: usize) -> Option<&image::RgbaImage> {
        self.element_icons.get(id)
    }

    fn navicust_part(&self, id: usize, variant: usize) -> Option<&rom::NavicustPart> {
        self.navicust_parts.get(id * 4 + variant)
    }

    fn num_navicust_parts(&self) -> (usize, usize) {
        (self.navicust_parts.len() / 4, 4)
    }

    fn modcard56(&self, id: usize) -> Option<&rom::Modcard56> {
        self.modcard56s.as_ref().and_then(|modcard56s| modcard56s.get(id))
    }

    fn num_modcard56s(&self) -> usize {
        self.modcard56s.as_ref().map(|modcards| modcards.len()).unwrap_or(0)
    }
}

#[rustfmt::skip]
pub const EN_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "*", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "RV", "BX", "EX", "SP", "FZ", "ウ", "ア", "イ", "オ", "エ", "ケ", "コ", "カ", "ク", "キ", "セ", "サ", "ソ", "シ", "ス", "テ", "ト", "ツ", "タ", "チ", "ネ", "ノ", "ヌ", "ナ", "ニ", "ヒ", "ヘ", "ホ", "ハ", "フ", "ミ", "マ", "メ", "ム", "モ", "ヤ", "ヨ", "ユ", "ロ", "ル", "リ", "レ", "ラ", "ン", "熱", "斗", "ワ", "ヲ", "ギ", "ガ", "ゲ", "ゴ", "グ", "ゾ", "ジ", "ゼ", "ズ", "ザ", "デ", "ド", "ヅ", "ダ", "ヂ", "ベ", "ビ", "ボ", "バ", "ブ", "ピ", "パ", "ペ", "プ", "ポ", "ゥ", "ァ", "ィ", "ォ", "ェ", "ュ", "ヴ", "ッ", "ョ", "ャ", "-", "×", "=", ":", "%", "?", "+", "█", "[bat]", "ー", "!", "&", ",", "゜", ".", "・", ";", "'", "\"", "~", "/", "(", ")", "「", "」", "�", "_", "ƶ", "[L]", "[B]", "[R]", "[A]", "あ", "い", "け", "く", "き", "こ", "か", "せ", "そ", "す", "さ", "し", "つ", "と", "て", "た", "ち", "ね", "の", "な", "ぬ", "に", "へ", "ふ", "ほ", "は", "ひ", "め", "む", "み", "も", "ま", "ゆ", "よ", "や", "る", "ら", "り", "ろ", "れ", "[END]", "ん", "を", "わ", "研", "げ", "ぐ", "ご", "が", "ぎ", "ぜ", "ず", "じ", "ぞ", "ざ", "で", "ど", "づ", "だ", "ぢ", "べ", "ば", "び", "ぼ", "ぶ", "ぽ", "ぷ", "ぴ", "ぺ", "ぱ", "ぅ", "ぁ", "ぃ", "ぉ", "ぇ", "ゅ", "ょ", "っ", "ゃ", "容", "量", "全", "木", "[MB]", "無", "現", "実", "[circle]", "×", "緑", "道", "不", "止", "彩", "起", "父", "集", "院", "一", "二", "三", "四", "五", "六", "七", "八", "陽", "十", "百", "千", "万", "脳", "上", "下", "左", "右", "手", "来", "日", "目", "月", "獣", "各", "人", "入", "出", "山", "口", "光", "電", "気", "綾", "科", "次", "名", "前", "学", "校", "省", "祐", "室", "世", "界", "高", "朗", "枚", "野", "悪", "路", "闇", "大", "小", "中", "自", "分", "間", "系", "花", "問", "究", "門", "城", "王", "兄", "化", "葉", "行", "街", "屋", "水", "見", "終", "新", "桜", "先", "生", "長", "今", "了", "点", "井", "子", "言", "太", "属", "風", "会", "性", "持", "時", "勝", "赤", "代", "年", "火", "改", "計", "画", "職", "体", "波", "回", "外", "地", "員", "正", "造", "値", "合", "戦", "川", "秋", "原", "町", "晴", "用", "金", "郎", "作", "数", "方", "社", "攻", "撃", "力", "同", "武", "何", "発", "少", "教", "以", "白", "早", "暮", "面", "組", "後", "文", "字", "本", "階", "明", "才", "者", "向", "犬", "々", "ヶ", "連", "射", "舟", "戸", "切", "土", "炎", "伊", "夫", "鉄", "国", "男", "天", "老", "師", "堀", "杉", "士", "悟", "森", "霧", "麻", "剛", "垣", "★", "[bracket1]", "[bracket2]", "[.]"];

#[rustfmt::skip]
pub const JA_CHARSET: &[&str] = &[" ", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "ウ", "ア", "イ", "オ", "エ", "ケ", "コ", "カ", "ク", "キ", "セ", "サ", "ソ", "シ", "ス", "テ", "ト", "ツ", "タ", "チ", "ネ", "ノ", "ヌ", "ナ", "ニ", "ヒ", "ヘ", "ホ", "ハ", "フ", "ミ", "マ", "メ", "ム", "モ", "ヤ", "ヨ", "ユ", "ロ", "ル", "リ", "レ", "ラ", "ン", "熱", "斗", "ワ", "ヲ", "ギ", "ガ", "ゲ", "ゴ", "グ", "ゾ", "ジ", "ゼ", "ズ", "ザ", "デ", "ド", "ヅ", "ダ", "ヂ", "ベ", "ビ", "ボ", "バ", "ブ", "ピ", "パ", "ペ", "プ", "ポ", "ゥ", "ァ", "ィ", "ォ", "ェ", "ュ", "ヴ", "ッ", "ョ", "ャ", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "*", "-", "×", "=", ":", "%", "?", "+", "■", "[bat]", "ー", "!", "RV", "BX", "&", "、", "゜", ".", "・", ";", "’", "\"", "~", "/", "(", ")", "「", "」", "EX", "SP", "FZ", "�", "_", "ƶ", "周", "え", "お", "う", "あ", "い", "け", "く", "き", "こ", "か", "せ", "そ", "す", "さ", "し", "つ", "と", "て", "た", "ち", "ね", "の", "な", "ぬ", "に", "へ", "ふ", "ほ", "は", "ひ", "め", "む", "み", "も", "ま", "ゆ", "よ", "や", "る", "ら", "り", "ろ", "れ", "[END]", "ん", "を", "わ", "研", "げ", "ぐ", "ご", "が", "ぎ", "ぜ", "ず", "じ", "ぞ", "ざ", "で", "ど", "づ", "だ", "ぢ", "べ", "ば", "び", "ぼ", "ぶ", "ぽ", "ぷ", "ぴ", "ぺ", "ぱ", "ぅ", "ぁ", "ぃ", "ぉ", "ぇ", "ゅ", "ょ", "っ", "ゃ", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "容", "量", "全", "木", "[MB]", "無", "現", "実", "[circle]", "×", "緑", "道", "不", "止", "彩", "起", "父", "集", "院", "一", "二", "三", "四", "五", "六", "七", "八", "陽", "十", "百", "千", "万", "脳", "上", "下", "左", "右", "手", "来", "日", "目", "月", "獣", "各", "人", "入", "出", "山", "口", "光", "電", "気", "綾", "科", "次", "名", "前", "学", "校", "省", "祐", "室", "世", "界", "高", "朗", "枚", "野", "悪", "路", "闇", "大", "小", "中", "自", "分", "間", "系", "花", "問", "究", "門", "城", "王", "兄", "化", "葉", "行", "街", "屋", "水", "見", "終", "新", "桜", "先", "生", "長", "今", "了", "点", "井", "子", "言", "太", "属", "風", "会", "性", "持", "時", "勝", "赤", "代", "年", "火", "改", "計", "画", "職", "体", "波", "回", "外", "地", "員", "正", "造", "値", "合", "戦", "川", "秋", "原", "町", "晴", "用", "金", "郎", "作", "数", "方", "社", "攻", "撃", "力", "同", "武", "何", "発", "少", "教", "以", "白", "早", "暮", "面", "組", "後", "文", "字", "本", "階", "明", "才", "者", "向", "犬", "々", "ヶ", "連", "射", "舟", "戸", "切", "土", "炎", "伊", "夫", "鉄", "国", "男", "天", "老", "師", "堀", "杉", "士", "悟", "森", "霧", "麻", "剛", "垣"];
