use {
    crate::{Error, Result, files::align}, bytey::*, std::{mem::MaybeUninit, ops::Range, str::from_utf8},
};

type Ref<'file> = ::core::cell::Ref<'file, [u8]>;
type RefMut<'file> = &'file mut [u8];

type Section = ([u8; 4], Range<usize>);
type Sections<const COUNT: usize> = [Section; COUNT];

#[derive(Debug)]
pub struct MsgBn<T, const SECTIONS: usize> {
    file: T,
    sections: Sections<SECTIONS>,
}

impl<'file, const SECTIONS: usize> MsgBn<Ref<'file>, SECTIONS> {
    pub fn try_read(file: Ref<'file>, magic: &'static [u8; 8]) -> Result<Self> {
        let sections = sections::<SECTIONS>(&file, magic)?;
        Ok(Self { file, sections })
    }

    pub fn get(&self, magic: &'static [u8; 4]) -> Option<Ref<'file>> {
        self.sections.iter().find_map(|(section_magic, section)| {
            (magic == section_magic)
                .then(|| Ref::map(Ref::clone(&self.file), |file| unsafe { file.get_unchecked(section.clone()) }))
        })
    }
}

impl<'file, const SECTIONS: usize> MsgBn<RefMut<'file>, SECTIONS> {
    pub fn try_read(file: RefMut<'file>, magic: &'static [u8; 8]) -> Result<Self> {
        let sections = sections::<SECTIONS>(&*file, magic)?;
        Ok(Self { file, sections })
    }

    pub fn into_section(self, magic: &'static [u8; 4]) -> Option<RefMut<'file>> {
        if let Some(section) =
            self.sections.iter().find_map(|(section_magic, section)| (magic == section_magic).then_some(section))
        {
            unsafe { Some(self.file.get_unchecked_mut(section.clone())) }
        } else {
            None
        }
    }
}

pub fn sections<const SECTIONS: usize>(file: &[u8], magic: &'static [u8; 8]) -> Result<Sections<SECTIONS>> {
    typedef! { struct Header<'h>: TryFromBytes<'h> [0x20] {
        [0] magic: &'h [u8; 8],
        [8] bom: u16 where bom == 0xFEFF,
        [0x12] size: u32,
    }}
    let mut sections = unsafe { MaybeUninit::<Sections<SECTIONS>>::zeroed().assume_init() };
    let (header, rest) = Header::try_from_slice(file)?;
    if header.magic == magic {
        if header.size as usize == file.len() {
            let mut rest = rest;
            let mut index = HEADER_LEN;
            for section_mut in sections.iter_mut() {
                typedef! { struct SectionHeader: FromBytes<'_> [0x20] {
                    [0] magic: [u8; 4],
                    [4] size: u32,
                }}
                let (header, section) = SectionHeader::try_from_slice(rest)?;
                let start = index + SECTION_HEADER_LEN;
                let aligned = align::<0x10>(header.size) as usize;
                let mid = start + aligned;
                if mid <= file.len() {
                    *section_mut = (header.magic, start..start + header.size as usize);
                    rest = &section[aligned - 0x10..];
                    index = mid;
                } else {
                    return Err(Error::new("Ran out of data."));
                }
            }

            if index != file.len() {
                panic!("Remaining section index {} did not match file size {}", index, file.len());
            }

            Ok(sections)
        } else {
            Err(Error::new("Size did not match."))
        }
    } else {
        Err(Error::new(format!(
            "Error parsing file: expected magic number ({:X?}) did not match file's magic number ({:X?})",
            magic, header.magic,
        )))
    }
}

pub struct SectionOwned {
    magic: [u8; 4],
    data: Vec<u8>,
}

pub struct MsgBnOwned {
    magic: [u8; 8],
    version: u8,
    encoding: u8,
    sections: Vec<SectionOwned>,
}

impl MsgBnOwned {
    pub fn from_file(file: &[u8], magic: &'static [u8; 8]) -> Result<MsgBnOwned> {
        typedef! { struct Header: TryFromBytes<'_> [0x20] {
            [0] magic: [u8; 8],
            [8] bom: u16 where bom == 0xFEFF,
            [0xC] encoding: u8,
            [0xD] version: u8,
            [0xE] section_count: u16,
            [0x12] size: u32,
        }}

        let (header, rest) = Header::try_from_slice(file)?;
        if header.magic != *magic {
            return Err(Error::new(format!(
                "Error parsing file: expected magic number ({:X?}) did not match file's magic number ({:X?})",
                magic, header.magic,
            )));
        }
        if header.size as usize != file.len() {
            return Err(Error::new("Size did not match."));
        }

        let Header {magic, version, encoding, section_count, ..} = header;
        let mut sections = Vec::new();
        let mut rest = rest;

        typedef! { struct SectionHeader: FromBytes<'_> [0x10] {
            [0] magic: [u8; 4],
            [4] size: u32,
        }}

        for _ in 0..(section_count as usize) {
            let (header, section) = SectionHeader::try_from_slice(rest)?;
            let size = header.size as usize;
            let aligned = align::<0x10>(header.size) as usize;
            if size > section.len() {
                return Err(Error::new("Ran out of data."));
            }
            let data = section[..size].to_vec();
            sections.push(SectionOwned { magic: header.magic, data });
            rest = &section[aligned..];
        }

        Ok(MsgBnOwned { magic, version, encoding, sections })
    }

    pub fn to_file(&self) -> Vec<u8> {
        let mut size = 0x20u32;
        for section in &self.sections {
            size += 0x10u32 + align::<0x10>(section.data.len() as u32);
        }

        let mut buf = Vec::new();
        buf.extend(self.magic);
        buf.extend(0xFEFFu16.to_le_bytes());
        buf.extend(0u16.to_le_bytes());
        buf.push(self.encoding);
        buf.push(self.version);
        buf.extend((self.sections.len() as u16).to_le_bytes());
        buf.extend(0u16.to_le_bytes());
        buf.extend(size.to_le_bytes());
        buf.extend([0; 0xA]);

        for section in &self.sections {
            size = section.data.len() as u32;
            let padding = align::<0x10>(size) - size;
            buf.extend(section.magic);
            buf.extend(size.to_le_bytes());
            buf.extend([0; 8]);
            buf.extend(&section.data);
            for _ in 0..padding {
                buf.push(0xAB);
            }
        }

        buf
    }

    pub fn section_mut(&mut self, magic: &'static [u8; 4]) -> Option<&mut SectionOwned> {
        self.sections.iter_mut().find_map(|section| (*magic == section.magic).then_some(section))
    }
}

const HEADER_LEN: usize = 0x20;
const SECTION_HEADER_LEN: usize = 0x10;

pub fn add_color(msbp: &mut MsgBnOwned, name: &str, rgba: [u8; 4]) {
    let clr_section = msbp.section_mut(b"CLR1").expect("MSBP file does not contain a CLR1 block");
    let item_index = clr_section.data[0] as u32;
    clr_section.data[0] += 1;
    clr_section.data.extend(&rgba);

    let clb_section = msbp.section_mut(b"CLB1").expect("MSBP file does not contain a CLB1 block");
    let mut clb_table = HashTable::from_bytes(&clb_section.data);
    clb_table.labels.push(Label {name: name.to_owned(), item_index});
    clb_section.data = clb_table.to_bytes();
}

struct HashTable {
    num_slots: usize,
    labels: Vec<Label>,
}

impl HashTable {
    fn from_bytes(data: &[u8]) -> Self {
        let (num_slots, _) = u32::try_from_slice(&data).unwrap();
        let num_slots = num_slots as usize;
        let mut labels = Vec::new();
        for slot in 0..num_slots {
            let (count, _) = u32::try_from_slice(&data[4 + 8 * slot ..]).unwrap();
            let (offset, _) = u32::try_from_slice(&data[8 + 8 * slot ..]).unwrap();
            let mut offset = offset as usize;
            for _ in 0..count {
                let len = data[offset] as usize;
                let str_bytes = &data[offset + 1 .. offset + len + 1];
                let name = from_utf8(str_bytes).unwrap().to_owned();
                let (item_index, _) = u32::try_from_slice(&data[offset + len + 1 ..]).unwrap();
                offset += len + 5;
                labels.push(Label { name, item_index });
            }
        }
        HashTable { num_slots, labels }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend((self.num_slots as u32).to_le_bytes());

        let mut slots = Vec::new();
        for _ in 0..self.num_slots {
            slots.push(Vec::new());
        }

        for label in &self.labels {
            let hash = calc_hash(&label.name, self.num_slots as u32) as usize;
            slots[hash].push(label);
        }

        let mut offset = 4 + 8 * self.num_slots;
        for i in 0..self.num_slots {
            buf.extend((slots[i].len() as u32).to_le_bytes());
            buf.extend((offset as u32).to_le_bytes());
            for label in &slots[i] {
                offset += label.name.len() + 5;
            }
        }

        for i in 0..self.num_slots {
            for label in &slots[i] {
                buf.push(label.name.len() as u8);
                buf.extend(label.name.as_bytes());
                buf.extend(label.item_index.to_le_bytes());
            }
        }

        buf
    }
}

struct Label {
    name: String,
    item_index: u32,
}

/// Standard Hash Function used by MSBT/MSBP files for lookups
/// https://github.com/Kinnay/Nintendo-File-Formats/wiki/LMS-File-Format#hash-table-slot
fn calc_hash(label: &str, num_slots: u32) -> u32 {
    label.chars().fold(0u32, |hash, char| hash.wrapping_mul(0x492) + (char as u32)) % num_slots
}
