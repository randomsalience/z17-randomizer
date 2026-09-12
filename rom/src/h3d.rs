use crate::files::align;
use crate::{Error, IntoBytes, Result};
use bytey::*;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum Section {
    Contents,
    Strings,
    Commands,
    Data,
    ExtData,
    Relocation,
}

#[derive(Clone)]
pub struct Resource {
    sections: Vec<RefCell<Vec<u8>>>,
    relocations: Vec<HashMap<usize, Section>>,
    header: Vec<u8>,
}

impl Resource {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        typedef! { struct Header: TryFromBytes<'_> [0x44] {
            #b"BCH\0",
            [4] min_version: u8 where min_version == 0x21,
            [5] max_version: u8 where max_version == 0x21,
            [6] converter_version: u16,
            [8] section_offsets: [u8; 0x18],
            [0x20] section_sizes: [u8; 0x20],
            [0x40] flags: u16,
            [0x42] address_count: u16,
        }}

        let (header, _) = Header::try_from_slice(bytes)?;
        let mut sections = Vec::new();
        for i in 0..6 {
            let (start, _) = u32::try_from_slice(&header.section_offsets[4 * i..])?;
            let start = start as usize;
            let end = if i == 5 { bytes.len() } else {
                let (end, _) = u32::try_from_slice(&header.section_offsets[4 * i + 4..])?;
                end as usize
            };
            sections.push(RefCell::new(bytes[start..end].to_vec()));
        }
        
        let mut relocations = vec![HashMap::new(); 6];
        {
            let reloc_section = sections[Section::Relocation as usize].borrow();
            let relocation_count = reloc_section.len() >> 2;
            for i in 0..relocation_count {
                let (reloc, _) = u32::try_from_slice(&reloc_section[4 * i..])?;
                let section = ((reloc & 0xe0000000) >> 29) as usize;
                let reloc_section = match (reloc & 0x1e000000) >> 25 {
                    1 => Section::Strings,
                    2..4 => Section::Commands,
                    4..9 => Section::Data,
                    9..14 => Section::ExtData,
                    _ => Section::Contents,
                };
                let mut offset = (reloc & 0x01ffffff) as usize;
                if reloc_section != Section::Strings {
                    offset <<= 2;
                }
                relocations[section].insert(offset, reloc_section);
            }
        }

        let header = bytes[0..0x44].to_vec();
        Ok(Self {sections, relocations, header})
    }

    fn section(&self, section: Section) -> Ref<'_, Vec<u8>> {
        self.sections[section as usize].borrow()
    }

    fn section_mut(&self, section: Section) -> RefMut<'_, Vec<u8>> {
        self.sections[section as usize].borrow_mut()
    }
}

impl IntoBytes for Resource {
    fn into_bytes(self) -> Box<[u8]> {
        let mut buf = vec![];
        buf.extend(&self.header);
        for (i, section) in self.sections.iter().enumerate() {
            match i {
                2 => { buf.resize(align::<0x10>(buf.len() as u32) as usize, 0); },
                3 | 4 => { buf.resize(align::<0x80>(buf.len() as u32) as usize, 0); },
                _ => {},
            }
            buf.extend(section.borrow().iter());
        }
        buf.into()
    }
}

struct Pointer<'a> {
    resource: &'a Resource,
    section: Section,
    offset: usize,
}

impl<'a> Pointer<'a> {
    fn new(resource: &'a Resource, section: Section, offset: usize) -> Pointer<'a> {
        Pointer {resource, section, offset}
    }

    fn add_offset(&self, offset: usize) -> Pointer<'a> {
        Pointer::new(self.resource, self.section, self.offset + offset)
    }

    fn follow(&self, offset: usize) -> Result<Pointer<'a>> {
        let offset = self.offset + offset;
        let (ptr_offset, _) = u32::try_from_slice(&self.resource.section(self.section)[offset..])?;
        let section = self.resource.relocations[self.section as usize]
            .get(&offset)
            .ok_or(Error::new(format!("data in bch file at section {} offset {:X} is not a pointer",
                self.section as usize, offset)))?;
        Ok(Pointer {resource: self.resource, section: *section, offset: ptr_offset as usize})
    }

    fn write(&self, offset: usize, data: &[u8]) {
        let mut section = self.resource.section_mut(self.section);
        let offset = self.offset + offset;
        section[offset..offset + data.len()].copy_from_slice(data);
    }
}

pub struct Model<'a>(Pointer<'a>);
pub struct Material<'a>(Pointer<'a>);
pub struct MaterialParam<'a>(Pointer<'a>);

impl<'a> Resource {
    pub fn get_model(&'a self, index: usize) -> Result<Model<'a>> {
        let models_pointer = Pointer::new(self, Section::Contents, 0).follow(0)?;
        Ok(Model(models_pointer.follow(4 * index)?))
    }
}

impl<'a> Model<'a> {
    pub fn get_material(&self, index: usize) -> Result<Material<'a>> {
        Ok(Material(self.0.follow(0x34)?.add_offset(0x2c * index)))
    }
}

impl<'a> Material<'a> {
    pub fn get_param(&self) -> Result<MaterialParam<'a>> {
        Ok(MaterialParam(self.0.follow(0)?))
    }
}

impl<'a> MaterialParam<'a> {
    pub fn set_color(&self, color: MaterialColor, value: [u8; 4]) {
        self.0.write(0x58 + 4 * (color as usize), &value);
    }

    pub fn set_tev_color(&self, index: TevStage, value: [u8; 4]) -> Result<()> {
        let commands = self.0.follow(0xc8)?;
        commands.write(0xd0 + 0x18 * (index as usize), &value);
        Ok(())
    }
}

pub enum MaterialColor {
    Emission,
    Ambient,
    Diffuse,
    Specular0,
    Specular1,
    Constant0,
    Constant1,
    Constant2,
    Constant3,
    Constant4,
    Constant5,
    Blend,
}

pub enum TevStage {
    Stage0,
    Stage1,
    Stage2,
    Stage3,
    Stage4,
    Stage5,
}
