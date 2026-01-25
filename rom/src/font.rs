use std::collections::HashMap;
use bytey::*;
use crate::{Error, Result};

pub struct Font {
    glyph_widths: HashMap<u16, u8>,
    code_map: HashMap<u16, u16>,
}

impl Font {
    pub fn new(file: &[u8]) -> Result<Font> {
        typedef! { struct Header<'h>: TryFromBytes<'h> [0x14] {
            [0] magic: &'h [u8; 4],
            [4] bom: u16 where bom == 0xfeff,
            [0xc] size: u32,
            [0x10] sections: u16,
        }}
        let (header, rest) = Header::try_from_slice(file)?;
        if header.magic != &[0x46, 0x46, 0x4e, 0x54] { // FFNT
            return Err(Error::new("Magic number did not match."));
        }
        if header.size as usize != file.len() {
            return Err(Error::new("Size did not match."));
        }
        let mut rest = rest;
        let mut glyph_widths = HashMap::new();
        let mut code_map = HashMap::new();
        let mut offset = 0x14;
        for _ in 0..header.sections {
            typedef! { struct SectionHeader: FromBytes<'_> [8] {
                [0] magic: [u8; 4],
                [4] size: u32,
            }}
            let (section_header, section) = SectionHeader::try_from_slice(rest)?;
            offset += section_header.size;
            if offset > header.size {
                return Err(Error::new("Ran out of data."));
            }
            rest = &rest[section_header.size as usize..];
            
            if section_header.magic == [0x43, 0x57, 0x44, 0x48] { // CWDH
                typedef! { struct WidthHeader: FromBytes<'_> [8] {
                    [0] first_index: u16,
                    [2] last_index: u16,
                }}
                let (width_header, width_data) = WidthHeader::try_from_slice(section)?;
                let mut data_offset = 1;
                for index in width_header.first_index..=width_header.last_index {
                    glyph_widths.insert(index, width_data[data_offset]);
                    data_offset += 3;
                }
            }

            if section_header.magic == [0x43, 0x4d, 0x41, 0x50] { // CMAP
                typedef! { struct CodeMapHeader: FromBytes<'_> [0xc] {
                    [0] first_code: u16,
                    [2] last_code: u16,
                    [4] map_type: u16,
                }}
                let (code_map_header, code_map_data) = CodeMapHeader::try_from_slice(section)?;
                match code_map_header.map_type {
                    0 => {
                        let (base_index, _) = u16::try_from_slice(code_map_data)?;
                        let mut index = base_index;
                        for code in code_map_header.first_code..=code_map_header.last_code {
                            code_map.insert(code, index);
                            index += 1;
                        }
                    },
                    1 => {
                        let mut data = code_map_data;
                        for code in code_map_header.first_code..=code_map_header.last_code {
                            let (index, rest) = u16::try_from_slice(data)?;
                            data = rest;
                            if index != 0xffff {
                                code_map.insert(code, index);
                            }
                        }
                    },
                    2 => {
                        let (count, data) = u16::try_from_slice(code_map_data)?;
                        let mut data = data;
                        for _ in 0..count {
                            let (code, rest) = u16::try_from_slice(data)?;
                            let (index, rest) = u16::try_from_slice(rest)?;
                            data = rest;
                            code_map.insert(code, index);
                        }
                    },
                    _ => {
                        return Err(Error::new("Invalid code map type."))
                    }
                }
            }
        }
        Ok(Font { glyph_widths, code_map })
    }

    pub fn wrap(&self, text: &str) -> Result<String> {
        const WRAP_WIDTH: usize = 360;
        let mut result = Vec::new();
        let mut word = Vec::new();
        let mut width = 0;
        let mut word_width = 0;
        for code in text.encode_utf16() {
            word.push(code);
            if code < 0x20 || code == 0xffff { // control characters
                continue;
            }

            let glyph = self.code_map.get(&code).ok_or(Error::new(format!("Invalid character code 0x{:X}.", code)))?;
            word_width += *self.glyph_widths.get(glyph).ok_or(Error::new(format!("Invalid glyph 0x{:X}.", glyph)))? as usize;

            if code == 0x20 || code == 0x2d || code == 0x5f { // space, hyphen, underscore
                if width + word_width < WRAP_WIDTH {
                    result.append(&mut word);
                    width += word_width;
                } else {
                    result.push(0xa); // newline
                    result.append(&mut word);
                    width = word_width;
                }
                word_width = 0;
            }
        }
        
        if width + word_width < WRAP_WIDTH {
            result.append(&mut word);
        } else {
            result.push(0xa); // newline
            result.append(&mut word);
        }

        Ok(String::from_utf16(&result).unwrap())
    }
}
