// Derived from the szs crate released by RiiStudio under the MIT license

use {
    crate::{Error, Result},
    bytey::*,
};

#[derive (Clone, Copy)]
struct Match {
    pos: usize,
    len: usize,
}

impl Match {
    fn new(pos: usize, len: usize) -> Match {
        Match { pos, len }
    }
}

struct Hasher {
    value: usize,
    hash_table: Vec<Option<usize>>,
    search_table: Vec<Option<usize>>,
}

impl Hasher {
    fn new() -> Hasher {
        Hasher {
            value: 0,
            hash_table: vec![None; 0x8000],
            search_table: vec![None; 0x1000],
        }
    }

    fn init(&mut self, value0: u8, value1: u8) {
        self.value = ((value0 as usize) << 5) ^ (value1 as usize);
    }

    fn push(&mut self, value: u8, pos: usize) -> Option<usize> {
        self.value = ((self.value << 5) & 0x7fff) ^ (value as usize);
        let prev_pos = self.hash_table[self.value];
        self.hash_table[self.value] = Some(pos);
        self.search_table[pos & 0xfff] = prev_pos;
        prev_pos
    }

    fn search(&self, data: &[u8], search_pos: usize, data_pos: usize) -> Option<Match> {
        if search_pos > data_pos || data_pos > search_pos + 0x1000 {
            return None;
        }

        let mut search_pos = search_pos;
        let mut best_match_len = 2;
        let mut best_match = None;

        for _ in 0..0x1000 {
            let cmp = |i: usize| data_pos + i < data.len() && data[data_pos + i] == data[search_pos + i];
            let mut match_len = 2;

            if cmp(0) && cmp(1) && cmp(best_match_len) {
                while match_len < 0x111 && cmp(match_len) {
                    match_len += 1;
                }
            }

            if match_len > best_match_len {
                best_match_len = match_len;
                best_match = Some(Match::new(data_pos - search_pos, match_len));
            }

            if match_len >= 0x111 {
                break;
            }

            let next_search_pos = self.search_table[search_pos & 0xfff];
            if let Some(next_search_pos) = next_search_pos {
                search_pos = next_search_pos;
                if search_pos <= data_pos - 0x1000 {
                    break;
                }
            } else {
                break;
            }
        }

        best_match
    }
}

pub fn decompress(file: &[u8]) -> Result<Vec<u8>> {
    typedef! { struct Header: TryFromBytes<'_> [0x10] {
        #b"Yaz0",
        [4] decompressed_size: [u8; 4],
        [8] padding: [u8; 8],
    }}
    let (header, data) = Header::try_from_slice(file)?;
    let decompressed_size = u32::from_be_bytes(header.decompressed_size);

    let mut out = Vec::with_capacity(decompressed_size as usize);
    let mut position = 0;

    while let Some(chunk_header) = data.get(position) {
        position += 1;
        for i in 0..8 {
            if position >= data.len() {
                return Ok(out);
            }

            if chunk_header & (1 << (7 - i)) != 0 {
                out.push(data[position]);
                position += 1;
            } else {
                let byte0 = data[position];
                let byte1 = data.get(position + 1).ok_or(Error::new("End of file reached"))?;

                let mut size = (byte0 >> 4) as usize;
                if size != 0 {
                    size += 2;
                    position += 2;
                } else {
                    let byte2 = data.get(position + 2).ok_or(Error::new("End of file reached"))?;
                    size = *byte2 as usize + 18;
                    position += 3;
                }

                let reverse = ((byte0 as usize & 0xf) << 8) + *byte1 as usize + 1;
                if reverse > out.len() {
                    return Err(Error::new("Invalid data in yaz0 file"));
                }

                let out_position = out.len() - reverse;
                for i in 0..size {
                    out.push(out[out_position + i]);
                }
            }
        }
    }

    Ok(out)
}

pub fn compress(file: &[u8]) -> Vec<u8> {
    let size = file.len() as u32;
    let mut out = Vec::new();
    out.extend_from_slice(b"Yaz0");
    out.extend_from_slice(&size.to_be_bytes());
    out.extend_from_slice(&[0; 8]);

    let mut hasher = Hasher::new();
    hasher.init(file[0], file[1]);
    let mut position = 0;
    let mut search_pos = None;
    let mut found_next_match = false;
    let mut temp_buffer = Vec::new();
    let mut bit = 8;
    let mut flag: u8 = 0;

    while let Some(&value) = file.get(position + 2) {
        if !found_next_match {
            search_pos = hasher.push(value, position);
        }
        found_next_match = false;

        let mut match_ = None;
        if let Some(search_pos_value) = search_pos {
            match_ = hasher.search(file, search_pos_value, position);
            if let Some(match_value) = match_ {
                if match_value.len < 0x111 {
                    position += 1;
                    search_pos = hasher.push(value, position);
                    found_next_match = true;
                    if let Some(search_pos_value) = search_pos {
                        let next_match = hasher.search(file, search_pos_value, position);
                        if let Some(next_match_value) = next_match {
                            if match_value.len < next_match_value.len {
                                match_ = None;
                            }
                        }
                    }
                }
            }
        }

        if let Some(match_value) = match_ {
            flag = flag << 1;
            let low = (match_value.pos - 1) as u8;
            let high = ((match_value.pos - 1) >> 8) as u8;
            if match_value.len < 18 {
                let high = high | ((match_value.len - 2) << 4) as u8;
                temp_buffer.extend(&[high, low]);
            } else {
                let len = (match_value.len - 18) as u8;
                temp_buffer.extend(&[high, low, len]);
            }

            let mut len = match_value.len - 1;
            if found_next_match {
                len -= 1;
            }
            while len > 0 {
                position += 1;
                if position + 2 < file.len() {
                    search_pos = hasher.push(file[position + 2], position);
                }
                len -= 1;
            }

            position += 1;
            found_next_match = false;
        } else {
            flag = (flag << 1) | 1;
            if !found_next_match {
                position += 1;
            }
            temp_buffer.push(file[position - 1]);
        }

        bit -= 1;
        if bit == 0 {
            out.push(flag);
            out.append(&mut temp_buffer);
            flag = 0;
            bit = 8;
        }
    }

    if bit < 8 {
        flag = flag << bit;
        out.push(flag);
        out.append(&mut temp_buffer);
    }

    out
}
