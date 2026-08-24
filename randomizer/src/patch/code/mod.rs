use super::Patcher;
use crate::filler::filler_item::Item::*;
use crate::filler::filler_item::Randomizable;
use crate::patch::actors::{HEART_PIECES, HEART_CONTAINERS, SMALL_KEYS, RUPEES};
use crate::patch::code::arm::Register::*;
use crate::patch::code::arm::data::{add, sub, cmp, mov, mul, orr, and, tst};
use crate::patch::code::arm::ls::{ldr, ldrb, ldrh, str_, strb};
use crate::patch::code::arm::lsm::{pop, push};
use crate::patch::code::arm::{Instruction, LR, PC, SP, b, bl, bx, blx};
use crate::{Layout, Result, SeedInfo, patch::util::prize_flag, regions};
use game::{Course, Item, Item::*};
use modinfo::settings::{Settings, Cracks, HintGhosts, PedestalSetting::*};
use rom::ExHeader;
use rom::flag::Flag;
use rom::scene::SpawnPoint;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::Path,
};

mod arm;

#[derive(Debug)]
pub struct Code {
    text: u32,
    text_end: u32,
    rodata: u32,
    rodata_end: u32,
    freespace: u32,
    freespace_end: u32,
    ips: Ips,
}

impl Code {
    pub fn new(exheader: &ExHeader) -> Self {
        let entry = exheader.get_text_address();
        let text = entry + exheader.get_text_size();
        let text_end = exheader.get_rodata_address();
        let rodata = exheader.get_rodata_address() + exheader.get_rodata_size();
        let rodata_end = exheader.get_data_address();
        let freespace = 0x3b7200; // Code at this location is related to Shadow Link, so it can be freely overwritten
        let freespace_end = 0x3bb680;
        let ips = Ips::new(entry);
        Self { text, text_end, rodata, rodata_end, freespace, freespace_end, ips }
    }

    pub fn text(&mut self) -> Segment<'_> {
        Segment { name: "text", address: &mut self.text, ips: &mut self.ips, end_address: &mut self.text_end }
    }

    pub fn rodata(&mut self) -> Segment<'_> {
        Segment { name: "rodata", address: &mut self.rodata, ips: &mut self.ips, end_address: &mut self.rodata_end }
    }

    pub fn freespace(&mut self) -> Segment<'_> {
        Segment { name: "freespace", address: &mut self.freespace, ips: &mut self.ips, end_address: &mut self.freespace_end }
    }

    pub fn patch<const N: usize>(&mut self, addr: u32, instructions: [Instruction; N]) -> u32 {
        let code = arm::assemble(addr, instructions);
        let len = code.len() as u32;
        self.overwrite(addr, code);
        len
    }

    #[allow(unused)]
    pub fn addr(&mut self, addr: u32, data: u32) {
        self.overwrite(addr, u32::to_le_bytes(data));
    }

    pub fn overwrite<T>(&mut self, addr: u32, data: T)
    where
        T: Into<Box<[u8]>>,
    {
        self.ips.append(addr, data.into());
    }

    pub fn dump<P>(self, path: P, exheader: &ExHeader) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let mut exheader = exheader.clone();
        exheader.set_text_size(self.text - exheader.get_text_address());
        exheader.set_rodata_size(self.rodata - exheader.get_rodata_address());
        self.ips.write(File::create(path.join("code.ips"))?)?;
        fs::write(path.join("exheader.bin"), exheader.as_ref())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Segment<'a> {
    name: &'static str,
    address: &'a mut u32,
    end_address: &'a mut u32,
    ips: &'a mut Ips,
}

impl<'a> Segment<'a> {
    pub fn declare<T>(&mut self, data: T) -> u32
    where
        T: Into<Vec<u8>>,
    {
        let mut data = data.into();
        let addr = *self.address;
        let len = data.len() as u32;
        let padded = (len + 3) & 0xFFFFFFFC;
        data.resize(padded as usize, 0);
        self.ips.append(addr, data);
        *self.address += padded;
        assert!(self.address <= self.end_address, "{} segment overflow", self.name);
        addr
    }

    pub fn define<const N: usize>(&mut self, instructions: [Instruction; N]) -> u32 {
        let addr = *self.address;
        let len = self.patch(addr, instructions);
        *self.address += len;
        assert!(self.address <= self.end_address, "{} segment overflow", self.name);
        addr
    }

    pub fn patch<const N: usize>(&mut self, addr: u32, instructions: [Instruction; N]) -> u32 {
        let code = arm::assemble(addr, instructions);
        let len = code.len() as u32;
        self.write(addr, code);
        len
    }

    pub fn write<T>(&mut self, addr: u32, data: T)
    where
        T: Into<Box<[u8]>>,
    {
        self.ips.append(addr, data.into());
    }
}

#[derive(Debug)]
pub struct Ips {
    buf: Vec<u8>,
    offset: u32,
}

impl Ips {
    pub fn new(offset: u32) -> Self {
        Self { buf: vec![], offset }
    }

    pub fn append<T>(&mut self, offset: u32, data: T)
    where
        T: AsRef<[u8]>,
    {
        let offset = offset - self.offset;
        let data = data.as_ref();
        self.buf.extend_from_slice(&offset.to_be_bytes()[1..4]);
        self.buf.extend_from_slice(&data.len().to_be_bytes()[6..8]);
        self.buf.extend(data);
    }

    pub fn write<W>(self, mut writer: W) -> Result<()>
    where
        W: Write,
    {
        writer.write_all(b"PATCH")?;
        writer.write_all(&self.buf)?;
        writer.write_all(b"EOF")?;
        Ok(())
    }
}

pub fn create(patcher: &Patcher, seed_info: &SeedInfo) -> Code {
    let mut code = Code::new(patcher.game.exheader());
    
    // This must be called first so the Archipelago header goes in the correct location
    if let Some(info) = &seed_info.archipelago_info {
        patch_archipelago(&mut code, seed_info.seed, &info.name);
    }
    
    let actor_names = actor_names(&mut code);
    let item_names = item_names(&mut code);

    do_dev_stuff(&mut code, seed_info);

    // warp(&mut code);
    // shield_without_sword(&mut code);
    // swordless_beams(&mut code);
    quake(&mut code);

    // Start with Pouch
    if seed_info.settings.start_with_pouch {
        code.text().patch(0x47b28c, [mov(R0, 1)]);
    }

    // Enable Y Button
    code.text().patch(0x47B2C8, [mov(R0, 1)]);

    // instant text
    code.overwrite(0x17A430, [0xFF]);

    new_items(&mut code);
    starting_gear(&mut code, &seed_info.settings);
    remove_charm_from_gear_menu(&mut code);
    fix_joystick_rotation(&mut code);
    rental_items(&mut code);
    progressive_items(&mut code, &seed_info.settings);
    bracelet(&mut code, &seed_info.settings);
    ore_progress(&mut code);
    merchant(&mut code);
    configure_pedestal_requirements(&mut code, &seed_info.settings);
    night_mode(&mut code, &seed_info.settings);
    show_hint_ghosts(&mut code, &seed_info.settings);
    mother_maiamai(&mut code, &seed_info.layout, &item_names);
    if seed_info.is_archipelago() && seed_info.settings.shuffle_maiamai_rewards {
        archipelago_mother_maiamai(&mut code, &seed_info.mother_maiamai_costs);
    }
    if seed_info.settings.change_freestanding_models {
        item_models(&mut code, &seed_info.layout, &actor_names);
    }
    pause_menu_warp(&mut code);
    purple_potion_bottles(&mut code, &seed_info.settings);
    // golden_bees(&mut code);
    // file_select_screen_background(&mut code);

    // Show Maiamai on Gear Screen even when you have zero
    code.patch(0x426490, [b(0x4264a0)]);

    // Correct Master Ore display count
    code.patch(0x4637b4, [b(0x463800)]);

    // Tear down Barrier automatically when obtaining Tempered Sword
    let set_barrier_flag = code.text().define([
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R1, Flag::HC_BARRIER.get_value()),
        mov(R2, 1),
        ldr(R0, (R0, 0)),
        bl(0x4CDF40),
        mov(R0, 1),
        pop([R4, R5, R6, PC]),
    ]);
    code.patch(0x344E7C, [b(set_barrier_flag)]);

    // don't lose Bow of Light on defeat
    code.patch(0x502DD8, [mov(R0, R0)]);

    // Infinite Scoot Fruit
    code.patch(0x38D59C, [mov(R2, 0x2)]);

    // Infinite Foul Fruit
    code.patch(0x38D728, [mov(R0, R0)]); // Don't clear equipped slot
    code.patch(0x38D734, [mov(R2, 0x2)]); // Keep fruit

    // blacksmith
    let get_sword_flag1 = code.text().define([
        push([LR]),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, 0x375),
        bl(0x584A80),
        add(R0, R0, 3),
        pop([PC]),
    ]);
    let get_sword_flag2 = code.text().define([
        push([LR]),
        ldr(R0, 0x70C8E0),
        ldr(R0, (R0, 0)),
        mov(R1, 0xCE),
        mov(R2, 3),
        bl(0x5822A0),
        add(R0, R0, 4),
        pop([PC]),
    ]);
    code.patch(0x243DE8, [bl(get_sword_flag1)]);
    code.patch(0x30E160, [bl(get_sword_flag2)]);

    let overwrite_rentals = code.text;
    let mut actor_offset = 0;
    for rental in patcher.rentals.iter() {
        let actor = actor_names
            .get(&rental.normalize())
            .copied()
            .unwrap_or_else(|| panic!("Could not find actor name for {}", rental.as_str()));
        code.text().define([
            ldr(R1, actor),
            str_(R4, (R0, actor_offset)),
            str_(R1, (R0, actor_offset + 4)),
            add(PC, PC, 0), // bad hack
        ]);
        actor_offset += 8;
    }
    code.text().define([b(0x5D68F4)]);
    code.patch(0x5D688C, [b(overwrite_rentals)]);
    let rental_data = patcher.rentals.iter()
        .flat_map(|item| item.as_item_index().to_le_bytes())
        .collect::<Vec<_>>();
    let rentals = code.rodata().declare(rental_data);
    let patch_rentals = code.text().define([
        ldr(R12, rentals),
        ldr(R2, (R12, R2, 2)),
        b(0x312734),
    ]);
    code.patch(0x312724, [b(patch_rentals)]);
    let sold_out = 0x5D6B84u32;
    let merchant_left = patcher.merchant[0];
    let merchant_left_actor = code.rodata().declare(VTABLE_STRING.to_le_bytes());
    code.rodata().declare(actor_names.get(&merchant_left.normalize()).unwrap().to_le_bytes());
    code.rodata().declare(VTABLE_STRING.to_le_bytes());
    code.rodata().declare(sold_out.to_le_bytes());
    code.overwrite(0x707DD4, merchant_left_actor.to_le_bytes());
    code.overwrite(0x6A03E0, (merchant_left.as_item_index() as u16).to_le_bytes());
    let merchant_right = patcher.merchant[2];
    let merchant_right_actor = code.rodata().declare(VTABLE_STRING.to_le_bytes());
    code.rodata().declare(actor_names.get(&merchant_right.normalize()).unwrap().to_le_bytes());
    code.rodata().declare(VTABLE_STRING.to_le_bytes());
    code.rodata().declare(sold_out.to_le_bytes());
    code.overwrite(0x707DE0, merchant_right_actor.to_le_bytes());
    code.overwrite(0x6A03E8, (merchant_right.as_item_index() as u16).to_le_bytes());

    // Hearts
    code.patch(0x33497C, [ldrh(R1, (R4, 0x2E)), mov(R0, R0)]);

    // Keys
    code.patch(0x192E58, [ldrh(R1, (R4, 0x2E))]);

    // Maiamai
    code.patch(0x514254, [ldrh(R1, (R4, 0x30))]);

    // Silver and Gold Rupees
    code.patch(0x1D6DBC, [ldrh(R1, (R4, 0x2E)), mov(R0, R0)]);

    // Premium Milk
    if seed_info.is_archipelago() || seed_info.layout.find_single(LetterInABottle).is_none() {
        // This code makes the Premium Milk work correctly when picked up without having first picked up the Letter.
        // This patch is only applied when the Milk is shuffled in the rando instead of the Letter.
        // If it's desired to have both shuffled at once then this code needs to be re-written.

        code.patch(0x3455C0, [bl(0x2558DC)]); // Repurpose Letter In a Bottle code
        code.patch(0x255930, [mov(R0, 0xD)]); // Give Milk instead of Letter
    }
    code.patch(0x345588, [b(0x34559C)]); // Skip setting Flag 916

    // Pendant Redirection - Get destination coordinates from Byaml
    let redirect_pendants = code.text().define([
        mov(R7, 0x1), // ???
        strb(R7, (SP, 0xA)),
        ldrb(R0, (R4, 0x42)), // scene = arg10
        str_(R0, (SP, 0x0)),
        ldrb(R0, (R4, 0x44)), // scene index = arg11
        str_(R0, (SP, 0x4)),
        ldrb(R0, (R4, 0x2C)), // spawn point = arg0
        strb(R0, (SP, 0x8)),
        b(0x143a78),
    ]);
    code.patch(0x143a3c, [b(redirect_pendants)]);

    // Pendant of Courage - Set Flag 251 when picked up
    let set_courage_flag = code.text().define([
        ldr(R0, EVENT_FLAG_PTR),
        mov(R2, 1),
        ldr(R1, Flag::EASTERN_COMPLETE.get_value()),
        ldr(R0, (R0, 0)),
        bl(FN_SET_EVENT_FLAG),
        b(0x344F00),
    ]);
    code.patch(0x344d9c, [b(set_courage_flag)]);

    // Great Spin Fix to work with and not disappear when obtaining Forgotten Sword
    let great_spin_fix =
        code.text().define([ldr(R0, (R4, 0x4E4)), cmp(R0, 0x3), mov(R2, 0x2).ne(), mov(R2, 0x3).eq(), b(0x344df0)]);
    code.patch(0x344dec, [b(great_spin_fix)]);

    // Increase number of text colors from 12 to 14
    code.patch(0x2cb258, [cmp(R4, 0xE)]);

    code
}

fn patch_archipelago(code: &mut Code, seed: u32, name: &str) {
    let ap_data_ptr = 0x6e9170u32;
    let archipelago_header = code.rodata().declare([0x41, 0x52, 0x43, 0x48]); // magic number
    code.rodata().declare([3, 0, 0, 0]); // data version
    code.rodata().declare(seed.to_le_bytes()); // seed
    code.rodata().declare(ap_data_ptr.to_le_bytes()); // pointer to writeable data
    let mut name_bytes = name.as_bytes().to_vec();
    name_bytes.resize(0x40, 0);
    code.rodata().declare(name_bytes); // username padded to 0x40 bytes

    let get_item = ap_data_ptr;
    code.overwrite(get_item, 0xffffffffu32.to_le_bytes());
    let received_items_counter = ap_data_ptr + 4;
    code.overwrite(received_items_counter, 0u32.to_le_bytes());
    let death_link_flag = ap_data_ptr + 8;
    code.overwrite(death_link_flag, 0u32.to_le_bytes());
    let in_game_flag = ap_data_ptr + 0xc;
    code.overwrite(in_game_flag, 0u32.to_le_bytes());

    let handle_death_link = code.text().define([
        push([R0, R1, R2, R3, R4, LR]),

        // Get DeathLink flag
        ldr(R4, death_link_flag),
        ldr(R0, (R4, 0)),
        cmp(R0, 0),         //if DeathLink flag not set
        pop([R0, R1, R2, R3, R4, PC]).eq(), //leave
        // Clear flag
        mov(R0, 0),
        str_(R0, (R4, 0)),

        // Get PlayerObject
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        cmp(R0, 0),
        pop([R0, R1, R2, R3, R4, PC]).eq(), //if error, leave

        ldr(R4, (R0, 0x10)),   // pPlayerObject
        cmp(R4, 0),
        pop([R0, R1, R2, R3, R4, PC]).eq(), //if error, leave

        // applyHeartDelta(pPlayerObject, -80, 1)
        // applyHeartDelta(pointer, damage (in quarter heart), some flag?)
        mov(R0, R4),
        mov(R1, 0),      
        sub(R1, R1, 80), //- 20 hearts
        mov(R2, 1),
        bl(0x001DD48C),

        // Get PlayerController
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        ldr(R0, (R0, 0x14)),
        ldr(R0, (R0, 0x48)),
        cmp(R0, 0), 
        pop([R0, R1, R2, R3, R4, PC]).eq(), //if error, leave

        // Build minimal damage struct
        mov(R1, 0x00110000),    
        // damage[0]=0, damage[1]=0x11
        // 0x11 = fall damage, doesn't use other values in struct, not affected by progressive mail level,
        // and makes a funny noise
        
        // call onDamage(controller, &damageStruct) to actually apply the damage done with applyHeartDelta and handle death
        bl(0x00363A0C),

        pop([R0, R1, R2, R3, R4, PC]),
    ]);

    // Save Archipelago information
    let patch_create_save = code.text().define([
        bl(0x320d78),
        ldr(R0, archipelago_header),
        ldr(R0, (R0, 0x0)),
        str_(R0, (R4, 0xde0)), // magic number
        ldr(R0, archipelago_header),
        ldr(R0, (R0, 0x4)),
        str_(R0, (R4, 0xde4)), // data version
        ldr(R0, archipelago_header),
        ldr(R0, (R0, 0x8)),
        str_(R0, (R4, 0xde8)), // seed
        mov(R0, 0x0),
        str_(R0, (R4, 0xdec)), // received items count
        b(0x293ad0),
    ]);
    code.patch(0x293acc, [b(patch_create_save)]);

    // Receive items from the server
    let receive_items_timer = code.rodata().declare([0, 0, 0, 0]);
    let receive_items_skip = code.text().define([
        pop([R0, R1, R4, R5, R6, LR]),
        b(0x349214),
    ]);
    let receive_items_quick = code.text().define([
        // Get item
        mov(R0, R4),
        ldr(R1, PLAYER_OBJECT_SINGLETON),
        ldr(R1, (R1, 0x0)),
        ldr(R1, (R1, 0x10)),
        bl(0x344834),
        // Set timer
        mov(R0, 0x1e),
        ldr(R1, receive_items_timer),
        str_(R0, (R1, 0x0)),
        // Increment received items counter
        ldr(R4, received_items_counter),
        ldr(R5, (R4, 0)),
        add(R5, R5, 1),
        str_(R5, (R4, 0)),
        // Set received item to -1
        ldr(R4, get_item),
        ldr(R5, -0x1),
        str_(R5, (R4, 0)),
        // Return to main Link procedure
        pop([R0, R1, R4, R5, R6, LR]),
        b(0x349214),
    ]);
    let receive_items = code.text().define([
        push(&[R0, R1, R4, R5, R6, LR]),
        bl(handle_death_link),
        // Return to normal function if player state is not 0 (standing) or 1 (walking)
        cmp(R1, 0x0),
        cmp(R1, 0x1).ne(),
        b(receive_items_skip).ne(),
        // Return to normal function if received item is -1
        ldr(R4, get_item),
        ldr(R4, (R4, 0)),
        add(R5, R4, 0x1),
        cmp(R5, 0x0),
        b(receive_items_skip).eq(),
        // Return to normal function if timer is still going
        ldr(R6, receive_items_timer),
        ldr(R5, (R6, 0x0)),
        cmp(R5, 0x0),
        sub(R5, R5, 0x1).ne(),
        str_(R5, (R6, 0x0)),
        b(receive_items_skip).ne(),
        // Check if we should use quick get item
        cmp(R4, 0x04),
        cmp(R4, 0x05).ne(),
        cmp(R4, 0x06).ne(),
        cmp(R4, 0x07).ne(),
        cmp(R4, 0x08).ne(),
        cmp(R4, 0x2d).ne(),
        cmp(R4, 0x2e).ne(),
        cmp(R4, 0x32).ne(),
        cmp(R4, 0x3a).ne(),
        cmp(R4, 0x3b).ne(),
        cmp(R4, 0x3c).ne(),
        cmp(R4, 0x5b).ne(),
        cmp(R4, CompassEastern as u32).ne(),
        cmp(R4, CompassGales as u32).ne(),
        cmp(R4, CompassHera as u32).ne(),
        cmp(R4, CompassDark as u32).ne(),
        cmp(R4, CompassSwamp as u32).ne(),
        cmp(R4, CompassSkull as u32).ne(),
        cmp(R4, CompassThieves as u32).ne(),
        cmp(R4, CompassIce as u32).ne(),
        cmp(R4, CompassDesert as u32).ne(),
        cmp(R4, CompassTurtle as u32).ne(),
        cmp(R4, CompassCastle as u32).ne(),
        cmp(R4, Item::BeeTrap as u32).ne(),
        b(receive_items_quick).eq(),
        // Call get item routine
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0x0)),
        mov(R1, R4),
        mov(R2, 0x0),
        bl(0x36174c),
        // Return to normal function if item get failed
        cmp(R0, 0x0),
        b(receive_items_skip).eq(),
        // Increment received items counter
        ldr(R4, received_items_counter),
        ldr(R5, (R4, 0)),
        add(R5, R5, 1),
        str_(R5, (R4, 0)),
        // Set received item to -1
        ldr(R4, get_item),
        ldr(R5, -0x1),
        str_(R5, (R4, 0)),
        pop([R0, R1, R4, R5, R6, LR]),
        bx(LR),
    ]);
    code.addr(0x6e30ac, receive_items);

    // Save the received items counter when saving the game
    let save_received_items_counter = code.text().define([
        ldr(R1, received_items_counter),
        ldr(R1, (R1, 0)),
        str_(R1, (R0, 0xdec)),
        b(0x320b74),
    ]);
    code.patch(0x4c3d60, [bl(save_received_items_counter)]);

    // Load the received items counter when loading the game
    let load_received_items_counter = code.text().define([
        ldr(R1, received_items_counter),
        ldr(R2, (R4, 0x14)),
        ldr(R2, (R2, 0xdec)),
        str_(R2, (R1, 0)),
        // Clear received item
        ldr(R1, get_item),
        ldr(R2, -1),
        str_(R2, (R1, 0)),
        // Set received item timer
        ldr(R1, receive_items_timer),
        mov(R2, 0x1e),
        str_(R2, (R1, 0)),
        b(0x4ad758),
    ]);
    code.patch(0x4c3b68, [bl(load_received_items_counter)]);

    // Allow getting items outside of an event
    let get_item_patch = code.text().define([
        push(&[R4, R5, R6]),
        ldr(R4, EVENT_FLAG_PTR),
        ldr(R4, (R4, 0)),
        ldr(R5, (R4, 0x10)),
        mov(R6, 0x1),
        str_(R6, (R4, 0x10)),
        blx(R2),
        str_(R5, (R4, 0x10)),
        pop(&[R4, R5, R6]),
        b(0x2922fc),
    ]);
    code.patch(0x2922f8, [b(get_item_patch)]);

    // Prevent getting item during stamina scroll animation
    let fix_get_stamina_scroll = code.text().define([
        cmp(R0, 0x45),
        b(0x344834).ne(),
        mov(R0, 0xf0),
        ldr(R1, receive_items_timer),
        str_(R0, (R1, 0x0)),
        mov(R0, 0x1),
        bx(LR),
    ]);
    code.patch(0x028edf0, [bl(fix_get_stamina_scroll)]);

    // Repurpose Letter in a Bottle as Archipelago Item
    code.patch(0x345578, [b(0x344f00)]);
    code.addr(0x3447c4, 0x34482c);

    // Allow Ravio items to have arbitrary names
    let change_ravio_text = code.text().define([
        ldr(R1, 0x714694),
        ldr(R0, (R1, R0, 2)),
        bx(LR),
    ]);
    code.patch(0x55af28, [b(change_ravio_text)]);

    // Change get item text for Archipelago items
    let ap_item_format_string = code.rodata().declare("ap_item_%d\0".to_string().into_bytes());
    let normal_get_item_message = code.text().define([
        pop([R0, R1, R2, R3]),
        bl(0x51bb74),
        add(SP, SP, 0x1c),
        b(0x28ea14),
    ]);
    let patch_get_item_message = code.text().define([
        sub(SP, SP, 0x1c),
        push([R0, R1, R2, R3]),
        // if bit 15 of item ID is not set (not an AP item), proceed as usual
        ldr(R1, (R4, 0x74)),
        tst(R1, 0x8000),
        b(normal_get_item_message).eq(),
        // strip bit 15 from item ID
        ldr(R3, 0x7fff),
        and(R2, R1, R3),
        // create a FixedSafeString<0x10> on the stack
        ldr(R1, VTABLE_FIXED_STRING_10),
        str_(R1, (SP, 0x10)),
        add(R1, SP, 0x1c),
        str_(R1, (SP, 0x14)),
        mov(R1, 0x10),
        str_(R1, (SP, 0x18)),
        // apply the format string
        add(R0, SP, 0x10),
        ldr(R1, ap_item_format_string),
        bl(FN_STRING_FORMAT),
        // set the get item message and return
        pop([R0, R1, R2, R3]),
        mov(R1, SP),
        bl(0x51bb74),
        add(SP, SP, 0x1c),
        b(0x28ea14),
    ]);
    code.patch(0x28ea10, [b(patch_get_item_message)]);

    // Since we're using bit 15 of the item ID to represent an Archipelago item,
    // we have to convert to the actual item ID when loading the get item info
    let get_correct_get_item = code.text().define([
        ldr(R2, (R4, 0x74)),
        tst(R2, 0x8000),
        mov(R2, 0x49).ne(),
        b(0x28e544),
    ]);
    code.patch(0x28e540, [b(get_correct_get_item)]);

    // Set in-game flag when the game starts and clear it when it stops
    let start_game = code.text().define([
        mov(R2, 1),
        ldr(R1, in_game_flag),
        str_(R2, (R1, 0)),
        b(0x1e3134),
    ]);
    code.overwrite(0x6d1e14, start_game.to_le_bytes());
    let end_game = code.text().define([
        mov(R2, 0),
        ldr(R1, in_game_flag),
        str_(R2, (R1, 0)),
        b(0x1e30e0),
    ]);
    code.overwrite(0x6d1e1c, end_game.to_le_bytes());
}

#[allow(unused_variables)]
fn do_dev_stuff(code: &mut Code, seed_info: &SeedInfo) {
    if !seed_info.settings.dev_mode {
        return;
    }

    // Make each Maiamai worth more (for testing only)
    let amount = 25;
    code.patch(0x2559bc, [add(R1, R1, amount)]);
    code.patch(0x2559c0, [add(R2, R2, amount)]);
}

/// Add new get items
fn new_items(code: &mut Code) {
    // Adjust size of get item list
    let item_count = Item::iter().count() as u32;
    code.addr(0x346b18, 0xec * item_count);
    code.patch(0x3a56b4, [mov(R3, item_count)]);
    code.patch(0x3a56dc, [cmp(R0, item_count)]);

    // Add messages for new get items
    let mut message_name_pointer_data = Vec::new();
    for item in Item::new_items() {
        let message_name = code.rodata().declare(item.get_item_message_name());
        code.rodata().declare([0]);
        message_name_pointer_data.append(&mut message_name.to_le_bytes().into());
    }
    let get_item_message_name_pointers = code.rodata().declare(message_name_pointer_data);

    let old_get_item_message_name = code.text().define([
        add(R1, R3, (R1, 3)),
        ldr(R1, (R1, 4)),
        b(0x4b9b08),
    ]);
    let fn_get_item_message_name = code.text().define([
        str_(R2, (R0, 0)),
        cmp(R1, 0x61),
        b(old_get_item_message_name).lt(),
        sub(R1, R1, 0x61),
        ldr(R3, get_item_message_name_pointers),
        add(R1, R3, (R1, 2)),
        ldr(R1, (R1, 0)),
        b(0x4b9b08),
    ]);
    code.patch(0x4b9afc, [b(fn_get_item_message_name)]);

    // Courses in the order of the get item keys
    let course_table = code.rodata().declare([
        Course::DungeonEast as u8,
        Course::DungeonWind as u8,
        Course::DungeonHera as u8,
        Course::DungeonDark as u8,
        Course::DungeonWater as u8,
        Course::DungeonDokuro as u8,
        Course::DungeonHagure as u8,
        Course::DungeonIce as u8,
        Course::DungeonSand as u8,
        Course::DungeonKame as u8,
        Course::DungeonGanon as u8,
        Course::CaveLight as u8,
        Course::AttractionDark as u8,
    ]);

    // Number of keys in each course
    let key_count_table = code.rodata().declare([2, 4, 2, 4, 4, 3, 1, 3, 5, 3, 5, 1, 1]);

    let not_new_item = code.text().define([
        mov(R0, 0),
        b(0x3459c4),
    ]);

    // Code for gaining a small key
    let fn_add_small_key = code.text().define([
        // Check if item is small key
        cmp(R0, Item::SMALL_KEY_START),
        b(not_new_item).lt(),
        cmp(R0, Item::SMALL_KEY_END),
        b(not_new_item).gt(),

        // Get course associated to key
        sub(R0, R0, Item::SMALL_KEY_START),
        ldr(R1, course_table),
        ldrb(R1, (R1, R0)),

        // Get current course
        ldr(R2, GAME_MANAGER),
        ldr(R2, (R2, 0)),
        ldrb(R2, (R2, 0x18)),

        // Load player inventory
        ldr(R3, PLAYER_OBJECT_SINGLETON),
        ldr(R3, (R3, 0)),
        ldr(R3, (R3, 0x10)),
        add(R3, R3, 0x400),
        add(R3, R3, 0xC),

        // Increment key count
        cmp(R1, R2),
        add(R2, R1, 0x10C),
        ldrb(R0, (R3, 0x24)).eq(),
        ldrb(R0, (R3, R2)).ne(),
        add(R0, R0, 1),
        strb(R0, (R3, 0x24)).eq(),
        strb(R0, (R3, R2)).ne(),

        b(0x344f00),
    ]);

    // Code for gaining a big key
    let fn_add_big_key = code.text().define([
        // Check if item is big key
        cmp(R0, Item::BIG_KEY_START),
        b(fn_add_small_key).lt(),
        cmp(R0, Item::BIG_KEY_END),
        b(fn_add_small_key).gt(),

        // Get course associated to key
        sub(R0, R0, Item::BIG_KEY_START),
        ldr(R1, course_table),
        ldrb(R1, (R1, R0)),

        // Get current course
        ldr(R2, GAME_MANAGER),
        ldr(R2, (R2, 0)),
        ldrb(R2, (R2, 0x18)),

        // Load player inventory
        ldr(R3, PLAYER_OBJECT_SINGLETON),
        ldr(R3, (R3, 0)),
        ldr(R3, (R3, 0x10)),
        add(R3, R3, 0x400),
        add(R3, R3, 0xC),

        // Increment key count
        cmp(R1, R2),
        add(R2, R1, 0x14C),
        ldrb(R0, (R3, 0x25)).eq(),
        ldrb(R0, (R3, R2)).ne(),
        add(R0, R0, 1),
        strb(R0, (R3, 0x25)).eq(),
        strb(R0, (R3, R2)).ne(),

        b(0x344f00),
    ]);

    // Code for gaining a compass
    let fn_add_compass = code.text().define([
        // Check if item is compass
        cmp(R0, Item::COMPASS_START),
        b(fn_add_big_key).lt(),
        cmp(R0, Item::COMPASS_END),
        b(fn_add_big_key).gt(),

        // Get course associated to compass
        sub(R0, R0, Item::COMPASS_START),
        ldr(R1, course_table),
        ldrb(R5, (R1, R0)),

        // Get course data
        ldr(R2, MAP_MANAGER),
        ldr(R2, (R2, 0)),
        mov(R3, 0x16c),
        mul(R1, R5, R3),
        add(R1, R1, 0x44),
        add(R0, R2, R1),

        // Set flag 0 for the course
        mov(R1, 0),
        mov(R2, 1),
        bl(0x1bb724),

        // Set flag 0 in the save data
        ldr(R0, SAVE_MANAGER),
        ldr(R0, (R0, 0)),
        add(R0, R0, 0x18),
        mov(R3, 0x40),
        mul(R2, R5, R3),
        add(R0, R0, R2),
        ldr(R1, (R0, 0)),
        orr(R1, R1, 1),
        str_(R1, (R0, 0)),

        b(0x344f00),
    ]);

    // Code for gaining an item upgrade
    let fn_add_upgrade = code.text().define([
        // Check if item is upgrade
        cmp(R0, Item::UPGRADE_START),
        b(fn_add_compass).lt(),
        cmp(R0, Item::UPGRADE_END),
        b(fn_add_compass).gt(),

        // Set event flag
        sub(R0, R0, Item::UPGRADE_START),
        ldr(R1, Flag::UPGRADE_START.get_value()),
        add(R1, R0, R1),
        mov(R2, 1),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        bl(FN_SET_EVENT_FLAG),

        b(0x344f00),
    ]);

    // Code for gaining a key ring
    let fn_add_key_ring = code.text().define([
        // Check if item is key ring
        cmp(R0, Item::KEY_RING_START),
        b(fn_add_upgrade).lt(),
        cmp(R0, Item::KEY_RING_END),
        b(fn_add_upgrade).gt(),

        // Get course associated to key ring
        sub(R0, R0, Item::KEY_RING_START),
        ldr(R1, course_table),
        ldrb(R1, (R1, R0)),

        // Get current course
        ldr(R2, GAME_MANAGER),
        ldr(R2, (R2, 0)),
        ldrb(R2, (R2, 0x18)),

        // Load player inventory
        ldr(R3, PLAYER_OBJECT_SINGLETON),
        ldr(R3, (R3, 0)),
        ldr(R3, (R3, 0x10)),
        add(R3, R3, 0x400),
        add(R3, R3, 0xC),

        // Increment key count
        cmp(R1, R2),
        add(R2, R1, 0x10C),
        ldr(R1, key_count_table),
        ldrb(R1, (R1, R0)),
        ldrb(R0, (R3, 0x24)).eq(),
        ldrb(R0, (R3, R2)).ne(),
        add(R0, R0, R1),
        strb(R0, (R3, 0x24)).eq(),
        strb(R0, (R3, R2)).ne(),

        b(0x344f00),
    ]);

    let fn_add_bee_trap = code.text().define([
        cmp(R0, Item::BeeTrap as u32),
        b(fn_add_key_ring).ne(),

        sub(SP, SP, 0xc),
        mov(R5, 0),
    ]);
    let lbl_spawn_bee = code.text;
    code.text().define([
        // load vector (0.0, 0.5, 0.0)
        mov(R0, 0),
        ldr(R1, 0x3f000000), // 0.5
        str_(R0, (SP, 0)),
        str_(R1, (SP, 4)),
        str_(R0, (SP, 8)),

        // load player position
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        ldr(R1, (R0, 0x1c)),

        // add (0.0, 0.5, 0.0) to player position
        mov(R0, SP),
        mov(R2, SP),
        bl(FN_VECTOR_ADD),

        // call createBee
        mov(R0, SP),
        mov(R1, 0),
        mov(R2, 0),
        mov(R3, 0),
        bl(0x4cbb0c),

        // loop 8 times
        add(R5, R5, 1),
        cmp(R5, 8),
        b(lbl_spawn_bee).lt(),

        add(SP, SP, 0xc),
        b(0x344f00),
    ]);

    code.patch(0x3459c0, [b(fn_add_bee_trap)]);
}

fn starting_gear(code: &mut Code, settings: &Settings) {
    if !settings.start_with_compasses {
        return;
    }

    let start_compasses = code.text().define([
        mov(R2, 1),
        add(R3, R0, 0x560),
        str_(R2, (R3, 0x240)),
        str_(R2, (R3, 0x280)),
        str_(R2, (R3, 0x2c0)),
        str_(R2, (R3, 0x300)),
        str_(R2, (R3, 0x340)),
        str_(R2, (R3, 0x380)),
        str_(R2, (R3, 0x3c0)),
        str_(R2, (R3, 0x400)),
        str_(R2, (R3, 0x440)),
        str_(R2, (R3, 0x480)),
        str_(R2, (R3, 0x4c0)),
        str_(R2, (R3, 0x500)),
        bl(0x4a143c),
        b(0x1df5b4),
    ]);
    code.patch(0x1df5b0, [b(start_compasses)]);
}

/// The game will show a green orb on the gear menu whether you have the Charm or the full Pendant
/// of Courage. This gets confusing for players who don't understand that the Charm is a junk item
/// that does nothing. This code prevents the green orb from appearing with the Charm, and will
/// make it only appear when the player has the actual Pendant of Courage.
fn remove_charm_from_gear_menu(code: &mut Code) {
    code.text().patch(0x42644c, [b(0x4264a8).ne()]);
}

/// For what are certainly reasons, Nintendo decided to rotate all of Link's movements ever so
/// slightly (about 5 degrees) counterclockwise in the vanilla game. This isn't often complained
/// about by people who play on physical 3DS hardware because reasons, but is very jarring to folks
/// who play on Emulators, and makes navigating the Ice Cave much more difficult than intended.
///
/// This code sets the rotation angle for each direction to zero, eliminating the issue.
fn fix_joystick_rotation(code: &mut Code) {
    code.overwrite(0x6c3ae8, [0x0; 8]); // Fix Down
    code.overwrite(0x6c3ef0, [0x0; 8]); // Fix Right
    code.overwrite(0x6c42e8, [0x0; 8]); // Fix Up
    code.overwrite(0x6c46f0, [0x0; 8]); // Fix Left
}

/// File Select Screen Background
/// TODO Figure out how to make background permanent
#[allow(unused)]
fn file_select_screen_background(code: &mut Code) {
    let sp = SpawnPoint::new(game::Course::Demo, 5, 0); // TODO use argument

    //code.text().patch(0x29d2c0, [mov(R10, 0x0)]); // Standstill camera on FSS over spawn point (ehh)
    code.text().patch(0x29d258, [mov(R0, 0x1)]); // Force always show same scene on FSS
    code.text().patch(0x29d270, [mov(R1, sp.course as u32)]);
    code.text().patch(0x29d278, [mov(R2, sp.scene as u32 - 1)]);
    code.text().patch(0x29d260, [mov(R6, sp.spawn as u32)]);
    let reset_r6 = code.text().define([mov(R6, 0x0), b(0x29d2a4)]);
    code.text().patch(0x29d284, [b(reset_r6)]);
}

#[allow(unused)]
fn warp(code: &mut Code) {
    // code.text().patch(0x441ec0, [b(0x442044)]); // Makes quit identical to cancel!
    // code.text().patch(0x4424b0, [mov(R0, R0)]); // Remove SE from "Quit" button!

    // code.text().patch(0x442498, [b(0x4424e0)]); // Quit button acts like Continue!

    let kill_player = code.text().define([
        // todo kill player here
        // no workie yet
        // ldr(R0, PLAYER_OBJECT_SINGLETON),
        // ldr(R0, (R0, 0x0)),
        // mov(R2, 0x0),
        // strb(R2, (R0, 0x598)),

        // bl(0x1973e4),

        // ldr(R0, 0x197440),
        // mov(R2, 0x1),
        // mov(R3, 0x0),
        //
        // mov(R1, SP),
        // ldr(R0, (R0, 0x0)),
        // bl(0x004ef418), // LoadScene
        b(0x4424e0), // Acts like player hit the "Continue" button
    ]);
    code.text().patch(0x442498, [b(kill_player)]);

    // code.text().patch(0x12cc7c, [mov(R0, R0)]); // Don't play SE_S_SELECT sound effect when hit continue button
    // Continue button sets: FUN_002317c0(0x3f800000,param_1 + 0x50);
}

/// Create new item that sets Flag 510
fn quake(code: &mut Code) {
    let earthquake = code.text().define([
        // TODO Play Earthquake Noise:
        // 0x6f8f24 - SE_EVENT_EARTQAUAKE
        // 0x587f74 - PlaySoundEffect Function

        // Set Flag
        ldr(R0, EVENT_FLAG_PTR),
        mov(R2, 0x1),
        ldr(R1, Flag::QUAKE.get_value()), // Quake
        ldr(R0, (R0, 0x0)),
        bl(FN_SET_EVENT_FLAG),
        b(0x344f00),
    ]);
    code.addr(0x344848, earthquake); // Empty -> Quake
}

/// Mother Maiamai Stuff
fn mother_maiamai(code: &mut Code, layout: &Layout, item_names: &HashMap<Item, u32>) {
    /// Use event flags 863-872 (not 866) to record whether we've picked up that item's upgrade.
    /// The "inventory index" (see table: 0x6a6170) of each item gets added to this:
    /// * 0x4 = Bow
    /// * 0x3 = Boomerang
    /// * 0xB = Hookshot
    /// * 0x6 = Hammer
    /// * 0x2 = Bombs
    /// * 0x8 = Fire Rod
    /// * 0x9 = Ice Rod
    /// * 0xA = Tornado Rod
    /// * 0x7 = Sand Rod
    const NEW_EVENT_FLAGS_START_IDX: u32 = 861;

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Great Spin final Nice Item check (?)
    // Accept Nice Items in addition to their regular counterparts for the check to see if we own anything upgradable.
    // code.patch(0x30fdcc, [b(0x30fef0).ge()]);
    let fn_get_maiamai_flag = code.text().define([
        ldr(R1, NEW_EVENT_FLAGS_START_IDX),
        add(R1, R1, R4),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0x0)),
        bl(FN_GET_EVENT_FLAG),
        b(0x30fdc4),
    ]);
    code.patch(0x30fdb8, [b(fn_get_maiamai_flag)]);
    code.patch(0x30fdc4, [cmp(R0, 0x0)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Rewrite ::::caseD_6 to check event flags for (I think?) 100 Maiamai item giveout
    let fn_get_maiamai_flag = code.text().define([
        ldr(R1, NEW_EVENT_FLAGS_START_IDX),
        add(R1, R1, R4),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0x0)),
        bl(FN_GET_EVENT_FLAG),
        b(0x30fee4),
    ]);
    code.patch(0x30fed8, [b(fn_get_maiamai_flag)]);
    code.patch(0x30fee4, [cmp(R0, 0x1)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    /*
     * Determines if the player has enough Maiamai compared with how many have been spent to show the upgrade dialog,
     * using the following formula:
     *
     * 0 < floor((total_maiamai_obtained - total_maiamai_on_hand) / (10 - num_nice_items_obtained))
     *
     * For randomizer, we need to replace `num_nice_items_obtained` with a count of our current flags that have been set.
     */
    code.patch(0x30fdf8, [b(0x30fe0c)]); // Skip Great Spin item check
    let fn_get_maiamai_flag = code.text().define([
        ldr(R1, NEW_EVENT_FLAGS_START_IDX),
        add(R1, R1, R4),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0x0)),
        bl(FN_GET_EVENT_FLAG),
        b(0x30fe48),
    ]);
    code.patch(0x30fe3c, [b(fn_get_maiamai_flag)]);
    code.patch(0x30fe48, [cmp(R0, 0x1)]);
    code.patch(0x30fe4c, [add(R5, R5, 0x1).eq()]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Skip FUN_00583b1c - Suck/Spit Old/New Item Animation
    code.patch(0x30ffe4, [mov(R0, 1)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Check our newly created event flags to determine if items can appear on MM's list of items to upgrade
    let fn_get_maiamai_flag = code.text().define([
        ldr(R2, NEW_EVENT_FLAGS_START_IDX),
        add(R1, R2, R1),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0x0)),
        bl(FN_GET_EVENT_FLAG),
        cmp(R0, 0x0),
        b(0x46d848).eq(),
        b(0x46d888),
    ]);
    code.patch(0x46d840, [b(fn_get_maiamai_flag).ge()]);
    code.patch(0x46d844, [b(0x46d888)]);

    // Allow getting upgrades if you already have the Nice Item for this slot
    code.patch(0x30feb4, [mov(R0, 0).eq()]);

    // Skip Sound: SE_ShopManKinSta_VACUUM
    code.patch(0x3105c8, [mov(R0, 0x0)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Ingoing Item Model
    // code.patch(0x30FF8C, [mov(R1, 0x11)]);
    // code.patch(0x30FF94, [mov(R1, 0x11)]);
    // code.patch(0x30FF9C, [mov(R1, 0x11)]);
    // code.patch(0x30FFA4, [mov(R1, 0x11)]);
    // code.patch(0x30FFAC, [mov(R1, 0x11)]);
    // code.patch(0x30FFB4, [mov(R1, 0x11)]);
    // code.patch(0x30FFBC, [mov(R1, 0x11)]);
    // code.patch(0x30FFC4, [mov(R1, 0x11)]);
    // code.patch(0x30FFCC, [mov(R1, 0x11)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Record upgrades received with a new event flag for each one.
    let bow = layout.get_unsafe("Maiamai Bow Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let boomerang = layout.get_unsafe("Maiamai Boomerang Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let hookshot = layout.get_unsafe("Maiamai Hookshot Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let hammer = layout.get_unsafe("Maiamai Hammer Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let bombs = layout.get_unsafe("Maiamai Bombs Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let fire_rod = layout.get_unsafe("Maiamai Fire Rod Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let ice_rod = layout.get_unsafe("Maiamai Ice Rod Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let tornado_rod = layout.get_unsafe("Maiamai Tornado Rod Upgrade", regions::hyrule::lake::cave::SUBREGION);
    let sand_rod = layout.get_unsafe("Maiamai Sand Rod Upgrade", regions::hyrule::lake::cave::SUBREGION);

    for (offset, addr, item) in [
        (4, 0x3100f8, bow),
        (3, 0x3100f0, boomerang),
        (11, 0x310128, hookshot),
        (6, 0x310100, hammer),
        (2, 0x310130, bombs),
        (8, 0x310110, fire_rod),
        (9, 0x310118, ice_rod),
        (10, 0x310120, tornado_rod),
        (7, 0x310108, sand_rod),
    ] {
        let fn_set_event_flag_for_this_upgrade = code.text().define([
            ldr(R0, EVENT_FLAG_PTR),
            ldr(R0, (R0, 0x0)),
            ldr(R1, offset + NEW_EVENT_FLAGS_START_IDX),
            mov(R2, 0x1),
            bl(FN_SET_EVENT_FLAG),
            ldr(R0, item.as_item_index()),
            b(0x310134),
        ]);
        code.patch(addr, [b(fn_set_event_flag_for_this_upgrade)]);
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    // Item Names
    let maiamai_item_name_table = code.rodata().declare(
        [ice_rod, sand_rod, tornado_rod, bombs, fire_rod, hookshot, boomerang, hammer, bow]
            .iter()
            .flat_map(|item| {
                u32::to_le_bytes(
                    *item_names
                        .get(&Randomizable::normalize(*item))
                        .unwrap_or_else(|| panic!("No item_name for: {item:?}")),
                )
            })
            .collect::<Vec<_>>(),
    );

    let fn_get_maiamai_item_name = code.text().define([
        push([R1, LR]),
        // R0 *= 4
        add(R0, R0, R0),
        add(R0, R0, R0),
        // R0 = maiamai_item_name_table[R0 * 4]
        ldr(R1, maiamai_item_name_table),
        ldr(R0, (R1, R0)),
        pop([R1, PC]),
    ]);

    code.patch(0x46d858, [bl(fn_get_maiamai_item_name)]);

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////
}

/// Patches for Mother Maiamai in Archipelago with randomized items
fn archipelago_mother_maiamai(code: &mut Code, mother_maiamai_costs: &[u8]) {
    // Alwas show upgrade dialog if you have items that can be upgraded
    code.patch(0x30fe64, [b(0x30fef0)]);

    // Skip maiamai walk animation and reduction of maiamai count
    code.patch(0x30fb00, [mov(R1, 0xc)]);

    // Skip Great Spin
    code.patch(0x30fda4, [b(0x30fef0)]);
    code.patch(0x30fec4, [b(0x30fef0)]);

    // Show correct item names
    let maiamai_item_names = code.rodata().declare(
        (0..9)
            .flat_map(|i| { format!("item_name_mm{}\0", i).into_bytes() })
            .collect::<Vec<_>>()
    );

    let get_maiamai_item_name = code.text().define([
        push([R1, LR]),
        mov(R0, 14),
        mul(R0, R4, R0),
        ldr(R1, maiamai_item_names),
        add(R0, R1, R0),
        pop([R1, PC]),
    ]);
    code.patch(0x46d858, [bl(get_maiamai_item_name)]);

    let maiamai_cost_table = code.rodata().declare(mother_maiamai_costs);
    let reverse_slot_table = code.rodata().declare([0xff, 0xff, 4, 1, 0, 0xff, 3, 8, 5, 6, 7, 2]);

    // Prevent getting items if you don't have enough maiamai
    // Adds a case 7 to the MSBF branch function
    let check_maiamai = code.text().define([
        cmp(R3, 7),
        b(0x30fd50).ne(),
        // R0 = total maiamai count
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        bl(0x556a2c),
        // R2 = item slot
        ldr(R1, reverse_slot_table),
        ldr(R2, (R4, 0xad8)),
        ldrb(R2, (R1, R2)),
        // R1 = maiamai cost
        ldr(R1, maiamai_cost_table),
        ldrb(R1, (R1, R2)),
        // compare count to cost
        cmp(R0, R1),
        b(0x30ff04).ge(),
        b(0x30fef0),
    ]);
    code.patch(0x30fd4c, [b(check_maiamai)]);

    // Gray out upgrades that cannot be purchased
    let set_alpha = code.text().define([
        push([R0, R2]),
        // R0 = total maiamai count
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        bl(0x556a2c),
        // R1 = maiamai cost
        ldr(R2, reverse_slot_table),
        ldrb(R2, (R2, R7)),
        ldr(R1, maiamai_cost_table),
        ldrb(R1, (R1, R2)),
        // compare count to cost
        cmp(R0, R1),
        // select alpha value
        mov(R0, 0x80).lt(),
        mov(R0, 0xff).ge(),
        // set alpha
        strb(R0, (R5, 0x45)),
        strb(R0, (R5, 0x46)),
        // return
        pop([R0, R2]),
        mov(R8, 0),
        b(0x46da18),
    ]);
    code.patch(0x46da14, [b(set_alpha)]);

    let set_cancel_alpha = code.text().define([
        mov(R1, 0xff),
        strb(R1, (R5, 0x45)),
        strb(R1, (R5, 0x46)),
        ldr(R0, (R5, 0)),
        b(0x46d9b4),
    ]);
    code.patch(0x46d9b0, [b(set_cancel_alpha)]);
}

/// Replace freestanding item models
fn item_models(code: &mut Code, layout: &Layout, actor_names: &HashMap<Item, u32>) {
    // Load heart piece / container BCH index from arg 0
    code.patch(0x334cb4, [ldrh(R2, (R4, 0x2C))]);
    // Load small key BCH index from arg 2
    code.patch(0x193758, [ldrh(R2, (R4, 0x30))]);
    // Load rupee BCH index from arg 2
    code.patch(0x1d7c2c, [ldrh(R2, (R4, 0x30))]);

    // Create new BCH lists
    for (data, offset, orig_bch) in [
        (HEART_PIECES.iter(), 0x707f34, 0x5d7b94),
        (HEART_CONTAINERS.iter(), 0x707f30, 0x5d7b7c),
        (SMALL_KEYS.iter(), 0x707d24, 0x5d6580),
        (RUPEES.iter(), 0x707e2c, 0x5d639c),
    ] {
        let bch_list = code.freespace().declare(VTABLE_STRING.to_le_bytes());
        code.freespace().declare((orig_bch as u32).to_le_bytes());
        code.freespace().declare(
            data
                .flat_map(|(name, _, _, _)|
                    VTABLE_STRING.to_le_bytes()
                        .into_iter()
                        .chain(actor_names
                            .get(&layout.get_by_name(name).normalize())
                            .unwrap()
                            .to_le_bytes()
                            .into_iter()))
                .collect::<Vec<_>>()
        );
        code.overwrite(offset, bch_list.to_le_bytes());
    }

    // Set new BCH counts
    code.overwrite(0x693d8d, [HEART_PIECES.len() as u8 + 1]);
    code.overwrite(0x693d8c, [HEART_CONTAINERS.len() as u8 + 1]);
    code.overwrite(0x693d09, [SMALL_KEYS.len() as u8 + 1]);
    code.overwrite(0x693d4b, [RUPEES.len() as u8 + 1]);

    // Get models from ActorCommon instead of stage archive
    code.overwrite(0x693b0c, [0]);
    code.overwrite(0x693b0d, [0]);
}

fn pause_menu_warp(code: &mut Code) {
    // Pause Menu...?
    // code.patch(0x441ee8, [mov(R0, R0)]); // Don't call function to return to FSS?
    // code.patch(0x441eec, [mov(R0, R0)]); // Don't call function to return to FSS?

    let _fn_load_scene_links_house = code.text().define([
        mov(R0, 0x0),
        bl(0x4eefa0),
        mov(R0, 0x1), // ???
        strb(R0, (SP, 0xA)),
        mov(R0, 0x2), // scene = IndoorLight
        str_(R0, (SP, 0x0)),
        mov(R0, 0x0), // index = 1
        str_(R0, (SP, 0x4)),
        mov(R0, 0x1), // spawn = 0
        strb(R0, (SP, 0x8)),
        mov(R3, 0x0),
        mov(R2, 0x0),
        mov(R1, SP),
        ldr(R0, 0x709df8),
        // ldr(R0, (R0, 0x0)),
        bl(0x4ef418), // Load Scene Function
        b(0x441eec),
    ]);

    // code.patch(0x441ee8, [b(fn_load_scene_links_house)]);

    // different attempt...
    // let fn_death_warp = code.text().define([
    //     ldr(R0, (R4, 0x0)), // ???
    //     bl(0x502d24),
    // ]);
    //
    // code.patch(0x441edc, [b(fn_death_warp)]);
    //
    // code.patch(0x0, 0x0);
}

fn purple_potion_bottles(code: &mut Code, settings: &Settings) {
    if settings.purple_potion_bottles {
        code.patch(0x255210, [mov(R1, 0x3)]);
    }
}

/// Golden Bee stuff
#[allow(unused)]
fn golden_bees(code: &mut Code) {
    // Alter Odds of a bee being a golden bee
    let golden_bee_chance = 3; // Choose percentage chance 0-100 (in vanilla it's 3)
    code.patch(0x4cbb8c, [cmp(R0, golden_bee_chance)]);
}

/// Show Hint Ghosts always, without the need for the Hint Glasses
fn show_hint_ghosts(code: &mut Code, settings: &Settings) {
    match settings.hint_ghosts {
        HintGhosts::Off => {
            // Prevent talking to Hint Ghosts
            code.patch(0x1cb3c8, [mov(R0, 0x0)]);

            // Skip checking if Hint Glasses are put on
            // Do not change state to "cState_Appear" (7)
            code.patch(0x1cb8cc, [b(0x1cb918)]);

            // Set initial state to "cState_DisappearWait" (6) instead of "cState_Wait" (0)
            code.patch(0x1cbf9c, [mov(R2, 0x0), b(0x1cc014)]);
        },
        HintGhosts::Glasses => {
            // Skip checking if Hint Glasses are taken off
            // Do not change state to "cState_Disappear" (5) or "cState_DisappearWait" (6)
            code.patch(0x1cb70c, [b(0x1cb74c)]);

            // Check if Hint Glasses in inventory instead of checking if they are worn
            let fn_check_glasses = code.text().define([
                push([R1, LR]),
                mov(R1, 0xe),
                bl(FN_GET_ITEM_LEVEL),
                pop([R1, LR]),
                bx(LR),
            ]);
            code.patch(0x1cbf9c, [bl(fn_check_glasses)]);
            code.patch(0x1cb8cc, [bl(fn_check_glasses)]);
            code.patch(0x1cb3c8, [bl(fn_check_glasses)]);
        },
        HintGhosts::Always => {
            // Allow talking to Hint Ghosts without glasses
            code.patch(0x1cb3c8, [mov(R0, 0x1)]);

            // Skip checking if Hint Glasses are taken off
            // Do not change state to "cState_Disappear" (5) or "cState_DisappearWait" (6)
            code.patch(0x1cb70c, [b(0x1cb74c)]);

            // Set initial state to "cState_Wait" (0) instead of "cState_DisappearWait" (6)
            code.patch(0x1cbf9c, [mov(R2, 0x0), b(0x1cbfac)]);
        }
    }
}

fn night_mode(code: &mut Code, settings: &Settings) {
    if settings.night_mode {
        // Keeps Flag 964 from being unset
        code.patch(0x3a8624, [mov(R2, 0x1)]);
    }
}

fn configure_pedestal_requirements(code: &mut Code, settings: &Settings) {
    const FLAG_PEDESTAL: u32 = 375;
    const RETURN_LABEL: u32 = 0x1439c8;

    let ped_instructions = match settings.ped_requirement {
        Vanilla => {
            code.text().define([
                // Power
                ldr(R0, EVENT_FLAG_PTR),
                ldr(R0, (R0, 0x0)),
                ldr(R1, prize_flag(PendantOfPower.into()).get_value() as u32),
                bl(FN_GET_EVENT_FLAG),
                cmp(R0, 0x0),
                b(RETURN_LABEL).eq(),
                // Wisdom
                ldr(R0, EVENT_FLAG_PTR),
                ldr(R0, (R0, 0x0)),
                ldr(R1, prize_flag(PendantOfWisdom.into()).get_value() as u32),
                bl(FN_GET_EVENT_FLAG),
                cmp(R0, 0x0),
                b(RETURN_LABEL).eq(),
                // Set Flag
                ldr(R0, EVENT_FLAG_PTR),
                mov(R2, 0x1),
                ldr(R1, FLAG_PEDESTAL),
                ldr(R0, (R0, 0x0)),
                bl(FN_SET_EVENT_FLAG),
                b(RETURN_LABEL),
            ])
        },
        Standard => {
            code.text().define([
                // Power
                ldr(R0, EVENT_FLAG_PTR),
                ldr(R0, (R0, 0x0)),
                ldr(R1, prize_flag(Randomizable::Item(PendantOfPower)).get_value() as u32),
                bl(FN_GET_EVENT_FLAG),
                cmp(R0, 0x0),
                b(RETURN_LABEL).eq(),
                // Wisdom
                ldr(R0, EVENT_FLAG_PTR),
                ldr(R0, (R0, 0x0)),
                ldr(R1, prize_flag(Randomizable::Item(PendantOfWisdom)).get_value() as u32),
                bl(FN_GET_EVENT_FLAG),
                cmp(R0, 0x0),
                b(RETURN_LABEL).eq(),
                // Courage
                ldr(R0, EVENT_FLAG_PTR),
                ldr(R0, (R0, 0x0)),
                ldr(R1, prize_flag(Randomizable::Item(PendantOfCourage)).get_value() as u32),
                bl(FN_GET_EVENT_FLAG),
                cmp(R0, 0x0),
                b(RETURN_LABEL).eq(),
                // Set Flag
                ldr(R0, EVENT_FLAG_PTR),
                mov(R2, 0x1),
                ldr(R1, FLAG_PEDESTAL),
                ldr(R0, (R0, 0x0)),
                bl(FN_SET_EVENT_FLAG),
                b(RETURN_LABEL),
            ])
        },
    };
    code.patch(0x143968, [b(ped_instructions)]);
}

fn merchant(code: &mut Code) {
    let get_merchant_event_flag =
        code.text().define([ldr(R0, EVENT_FLAG_PTR), ldr(R0, (R0, 0)), ldr(R1, 0x143), b(FN_GET_EVENT_FLAG)]);
    code.patch(0x19487C, [bl(get_merchant_event_flag)]);
}

#[allow(unused)]
fn shield_without_sword(code: &mut Code) {
    let enable_shield_fn = code.text().define([
        mov(R3, 0),
        add(R0, R4, 0x400),
        add(R0, R0, 0x294),
        mov(R2, R3),
        mov(R1, R3),
        bl(0x2d455c), // Enables Shield ???
        b(0x344f00),
    ]);
    code.text().patch(0x3450fc, [b(enable_shield_fn)]);
}

#[allow(unused)]
fn swordless_beams(code: &mut Code) {
    let _ = code.text().define([
        mov(R3, 0),
        add(R0, R4, 0x400),
        add(R0, R0, 0x294),
        mov(R2, R3),
        mov(R1, R3),
        bl(0x2d455c), // Enables Shield ???
        b(0x344f00),
    ]);
}

fn rental_items(code: &mut Code) {
    let map_rental_item = 0x194BFC;
    let flag_offset = 0xF0;
    let getter = code.text().define([
        push([LR]),
        ldr(R0, 0x70C8E0),
        ldr(R0, (R0, 0)),
        add(R1, R1, flag_offset),
        mov(R2, 3),
        bl(0x5822A0),
        cmp(R0, 1),
        mov(R0, 2).eq(),
        pop([PC]),
    ]);
    code.patch(0x194728, [bl(getter)]);
    code.patch(0x311CE4, [bl(getter)]);
    code.patch(0x311EAC, [bl(getter)]);
    code.patch(0x31261C, [bl(getter)]);
    code.patch(0x312660, [bl(getter)]);
    let setter = code.text().define([
        ldrb(R0, (R4, 0x9D0)),
        bl(map_rental_item),
        mov(R6, R1),
        mov(R1, R0),
        add(R1, R1, flag_offset),
        ldr(R0, 0x70C8E0),
        ldr(R0, (R0, 0)),
        mov(R2, 3),
        mov(R3, 1),
        bl(0x4AD9E8),
        b(0x652E70),
    ]);
    code.patch(0x652E34, [b(setter).eq()]);
}

fn progressive_items(code: &mut Code, settings: &Settings) {
    let return_label = code.freespace().define([
        pop([R1, R2, R4]),
        b(0x2922C4),
    ]);
    /*let first_sword = code.freespace().define([
        ldr(R0, (R0, 0x4C4)),
        cmp(R0, 0),
        mov(R5, 0x3D).eq(),
        mov(R5, 0x1B).ne(),
        b(return_label),
    ]);*/
    let progressive_sword = //if settings.items.captains_sword.is_skipped() {
        code.freespace().define([
            cmp(R5, 0x1B),
            cmp(R5, 0x1C).ne(),
            b(return_label).ne(),
            ldr(R3, (R0, 0x434)),
            cmp(R3, 0),
            mov(R3, 1).eq(),
            cmp(R3, 5),
            mov(R3, 4).eq(),
            add(R5, R3, 0x1A),
            b(return_label)
        ]);
    /*} else {
        code.freespace().define([
            cmp(R5, 0x1B),
            cmp(R5, 0x1C).ne(),
            cmp(R5, 0x3D).ne(),
            b(return_label).ne(),
            ldr(R3, (R0, 0x434)),
            cmp(R3, 0),
            b(first_sword).eq(),
            add(R5, R3, 0x1A),
            b(return_label),
        ])
    };*/
    let progressive_bracelet = match settings.cracks {
        Cracks::Open | Cracks::Closed => {
            code.freespace().define([
                cmp(R5, 0x2A),
                b(progressive_sword).ne(),
                ldr(R0, (R0, 0x490)),
                cmp(R0, 0),
                mov(R5, 0x2A).eq(),
                mov(R5, 0x2B).ne(),
                b(return_label),
            ])
        },
        Cracks::Progressive => {
            code.freespace().define([
                cmp(R5, 0x2A),
                b(progressive_sword).ne(),
                ldr(R0, (R0, 0x490)),
                cmp(R0, 0),
                mov(R5, 0x2B).eq(),
                mov(R5, 0).ne(),
                b(return_label),
            ])
        }
    };
    let progressive_glove = code.freespace().define([
        cmp(R5, 0x2F),
        b(progressive_bracelet).ne(),
        ldr(R0, (R0, 0x4AC)),
        cmp(R0, 0),
        mov(R5, 0x2F).eq(),
        mov(R5, 0x31).ne(),
        b(return_label),
    ]);
    let progressive_mail = code.freespace().define([
        cmp(R5, 0x3F),
        b(progressive_glove).ne(),
        ldr(R0, (R0, 0x4B0)),
        cmp(R0, 2),
        mov(R5, 0x3F).eq(),
        mov(R5, 0x40).ne(),
        b(return_label),
    ]);
    let progressive_lamp = code.freespace().define([
        cmp(R5, 0x1A),
        b(progressive_mail).ne(),
        ldr(R4, (R0, 0x464)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_LAMP.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0x1A).eq(),
        mov(R5, 0x58).ne(),
        b(return_label),
    ]);
    let progressive_bow = code.freespace().define([
        cmp(R5, 0x11),
        b(progressive_lamp).ne(),
        ldr(R4, (R0, 0x444)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_BOW.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0x11).eq(),
        mov(R5, 0x55).ne(),
        b(return_label),
    ]);
    let progressive_boomerang = code.freespace().define([
        cmp(R5, 0xF),
        b(progressive_bow).ne(),
        ldr(R4, (R0, 0x440)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_BOOMERANG.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xF).eq(),
        mov(R5, 0x53).ne(),
        b(return_label),
    ]);
    let progressive_hookshot = code.freespace().define([
        cmp(R5, 0xE),
        b(progressive_boomerang).ne(),
        ldr(R4, (R0, 0x460)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_HOOKSHOT.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xE).eq(),
        mov(R5, 0x52).ne(),
        b(return_label),
    ]);
    let progressive_hammer = code.freespace().define([
        cmp(R5, 0x10),
        b(progressive_hookshot).ne(),
        ldr(R4, (R0, 0x44C)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_HAMMER.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0x10).eq(),
        mov(R5, 0x54).ne(),
        b(return_label),
    ]);
    let progressive_bombs = code.freespace().define([
        cmp(R5, 0xC),
        b(progressive_hammer).ne(),
        ldr(R4, (R0, 0x43C)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_BOMBS.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xC).eq(),
        mov(R5, 0x50).ne(),
        b(return_label),
    ]);
    let progressive_fire_rod = code.freespace().define([
        cmp(R5, 0xD),
        b(progressive_bombs).ne(),
        ldr(R4, (R0, 0x454)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_FIRE_ROD.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xD).eq(),
        mov(R5, 0x51).ne(),
        b(return_label),
    ]);
    let progressive_ice_rod = code.freespace().define([
        cmp(R5, 0x9),
        b(progressive_fire_rod).ne(),
        ldr(R4, (R0, 0x458)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_ICE_ROD.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0x9).eq(),
        mov(R5, 0x4D).ne(),
        b(return_label),
    ]);
    let progressive_tornado_rod = code.freespace().define([
        cmp(R5, 0xB),
        b(progressive_ice_rod).ne(),
        ldr(R4, (R0, 0x45C)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_TORNADO_ROD.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xB).eq(),
        mov(R5, 0x4F).ne(),
        b(return_label),
    ]);
    let progressive_sand_rod = code.freespace().define([
        cmp(R5, 0xA),
        b(progressive_tornado_rod).ne(),
        ldr(R4, (R0, 0x450)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_SAND_ROD.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0xA).eq(),
        mov(R5, 0x4E).ne(),
        b(return_label),
    ]);
    let progressive_net = code.freespace().define([
        cmp(R5, 0x30),
        b(progressive_sand_rod).ne(),
        ldr(R4, (R0, 0x468)),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, Flag::UPGRADE_NET.get_value()),
        bl(FN_GET_EVENT_FLAG),
        orr(R0, R0, R4),
        cmp(R0, 0),
        mov(R5, 0x30).eq(),
        mov(R5, 0x59).ne(),
        b(return_label),
    ]);
    let lamp_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeLamp as u32),
        b(progressive_net).ne(),
        ldr(R0, (R0, 0x464)),
        cmp(R0, 0),
        mov(R5, 0x58).ne(),
        b(return_label),
    ]);
    let bow_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeBow as u32),
        b(lamp_upgrade).ne(),
        ldr(R0, (R0, 0x444)),
        cmp(R0, 0),
        mov(R5, 0x55).ne(),
        b(return_label),
    ]);
    let boomerang_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeBoomerang as u32),
        b(bow_upgrade).ne(),
        ldr(R0, (R0, 0x440)),
        cmp(R0, 0),
        mov(R5, 0x53).ne(),
        b(return_label),
    ]);
    let hookshot_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeHookshot as u32),
        b(boomerang_upgrade).ne(),
        ldr(R0, (R0, 0x460)),
        cmp(R0, 0),
        mov(R5, 0x52).ne(),
        b(return_label),
    ]);
    let hammer_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeHammer as u32),
        b(hookshot_upgrade).ne(),
        ldr(R0, (R0, 0x44C)),
        cmp(R0, 0),
        mov(R5, 0x54).ne(),
        b(return_label),
    ]);
    let bomb_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeBombs as u32),
        b(hammer_upgrade).ne(),
        ldr(R0, (R0, 0x43C)),
        cmp(R0, 0),
        mov(R5, 0x50).ne(),
        b(return_label),
    ]);
    let fire_rod_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeFireRod as u32),
        b(bomb_upgrade).ne(),
        ldr(R0, (R0, 0x454)),
        cmp(R0, 0),
        mov(R5, 0x51).ne(),
        b(return_label),
    ]);
    let ice_rod_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeIceRod as u32),
        b(fire_rod_upgrade).ne(),
        ldr(R0, (R0, 0x458)),
        cmp(R0, 0),
        mov(R5, 0x4D).ne(),
        b(return_label),
    ]);
    let tornado_rod_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeTornadoRod as u32),
        b(ice_rod_upgrade).ne(),
        ldr(R0, (R0, 0x45C)),
        cmp(R0, 0),
        mov(R5, 0x4F).ne(),
        b(return_label),
    ]);
    let sand_rod_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeSandRod as u32),
        b(tornado_rod_upgrade).ne(),
        ldr(R0, (R0, 0x450)),
        cmp(R0, 0),
        mov(R5, 0x4E).ne(),
        b(return_label),
    ]);
    let net_upgrade = code.freespace().define([
        cmp(R5, Item::UpgradeNet as u32),
        b(sand_rod_upgrade).ne(),
        ldr(R0, (R0, 0x468)),
        cmp(R0, 0),
        mov(R5, 0x59).ne(),
        b(return_label),
    ]);
    let progressive_charm = code.freespace().define([
        cmp(R5, 0x3E),
        b(net_upgrade).ne(),
        ldr(R0, (R0, 0x4A0)),
        cmp(R0, 0),
        mov(R5, 0x3E).eq(),
        mov(R5, 0x19).ne(),
        b(return_label),
    ]);
    let progressive_ore = code.freespace().define([
        cmp(R5, 0x42),
        cmp(R5, 0x43).ne(),
        cmp(R5, 0x44).ne(),
        cmp(R5, 0x48).ne(),
        b(progressive_charm).ne(),
        ldr(R3, (R0, 0x4D8)),
        cmp(R3, 0),
        mov(R5, 0x42).eq(),
        b(return_label).eq(),
        ldr(R3, (R0, 0x4D4)),
        cmp(R3, 0),
        mov(R5, 0x43).eq(),
        b(return_label).eq(),
        ldr(R3, (R0, 0x4E0)),
        cmp(R3, 0),
        mov(R5, 0x44).eq(),
        mov(R5, 0x48).ne(),
        b(return_label),
    ]);
    let start_progressive = code.freespace().define([
        push([R1, R2, R4]),
        b(progressive_ore),
    ]);

    code.patch(0x2922A0, [b(start_progressive)]);
}

fn bracelet(code: &mut Code, settings: &Settings) {
    if settings.start_with_merge {
        // Check Flag 1 (always set) instead of Flag 250 to see if we can merge.
        code.patch(0x4266c8, [mov(R1, 0x1)]);
        code.patch(0x537c40, [mov(R1, 0x1)]);
        return;
    }

    /*
     * FIXME
     *
     * In vanilla, the game determines whether you can merge not by checking your inventory but by
     * checking Flag 250 (Yuga 1 defeated). This patch adds an entry in the Add() function for
     * RingHekiga (full bracelet) and raises the Bracelet level in the inventory to 3 (impossible in
     * vanilla, where it's either 0 if you don't have it or 2 if you have RingRental). The patch
     * then changes all the locations where Flag 250 is checked and instead has them check the
     * player inventory for Bracelet level 3.
     *
     * This works in rando, but breaks the ability to Merge if a vanilla file is loaded as the
     * player inventory for the Bracelet can never be set to 3. Low priority, but change this.
     */

    let item_set_value = 0x255494;
    let add_ring_hekiga = code.text().define([
        add(R0, R4, 0x400),
        mov(R2, 3),
        mov(R1, 0x17),
        add(R0, R0, 0xC),
        bl(item_set_value),
        b(0x344F00),
    ]);
    code.overwrite(0x3448F4, add_ring_hekiga.to_le_bytes());
    let can_merge = code.text().define([
        push([LR]),
        ldr(R0, PLAYER_OBJECT_SINGLETON),
        ldr(R0, (R0, 0)),
        mov(R1, 0x17),
        bl(FN_GET_ITEM_LEVEL),
        cmp(R0, 3),
        mov(R0, 1).eq(),
        mov(R0, 0).ne(),
        pop([PC]),
    ]);
    code.patch(0x1DCA8C, [bl(can_merge)]);
    code.patch(0x4266D0, [bl(can_merge)]);
    code.patch(0x52E654, [bl(can_merge)]);
}

fn ore_progress(code: &mut Code) {
    let get_sword_fake = code.text().define([
        push([R4, LR]),
        mov(R4, 1),
        ldr(R0, 0x70C8E0),
        ldr(R0, (R0, 0)),
        add(R0, R0, 0x400),
        add(R0, R0, 0x88),
        mov(R1, 0xCE),
        bl(0x52A05C),
        add(R4, R4, R0),
        ldr(R0, EVENT_FLAG_PTR),
        ldr(R0, (R0, 0)),
        ldr(R1, 0x375),
        bl(0x584A80),
        add(R4, R4, R0),
        mov(R0, R4),
        pop([R4, PC]),
    ]);
    code.patch(0x4637B8, [bl(get_sword_fake)]);
}

fn actor_names(code: &mut Code) -> HashMap<Item, u32> {
    let mut map = IntoIterator::into_iter(ACTOR_NAME_OFFSETS).collect::<HashMap<_, _>>();
    map.extend(IntoIterator::into_iter(ACTOR_NAMES).map(|(item, name)| {
        let name = format!("{}\0", name);
        (item, code.freespace().declare(name.as_bytes()))
    }));
    map
}

fn item_names(code: &mut Code) -> HashMap<Item, u32> {
    let mut map = IntoIterator::into_iter(ITEM_NAME_OFFSETS).collect::<HashMap<_, _>>();
    map.extend(IntoIterator::into_iter(ITEM_NAMES).map(|(item, name)| {
        let name = format!("item_name_{}\0", name);
        (item, code.freespace().declare(name.as_bytes()))
    }));
    // log::info!("{map:?}");
    map
}

const ACTOR_NAME_OFFSETS: [(Item, u32); 28] = [
    (ItemStoneBeauty, 0x5D2060),
    (RupeeSilver, 0x5D63A4),
    (KeySmall, 0x5D6580),
    (ItemIceRod, 0x5D6AFC),
    (ItemSandRod, 0x5D6B08),
    (ItemTornadeRod, 0x5D6B18),
    (ItemBomb, 0x5D6B28),
    (ItemFireRod, 0x5D6B30),
    (ItemHookShot, 0x5D6B40),
    (ItemBoomerang, 0x5D6B50),
    (ItemHammer, 0x5D6B60),
    (ItemBow, 0x5D6B6C),
    (ItemShield, 0x5D6B78),
    (ItemBottle, 0x5D7048),
    (Item::HintGlasses, 0x5D70AC),
    (RupeeGold, 0x5D7144),
    (ItemSwordLv1, 0x5D7178),
    (ItemSwordLv2, 0x5D7178),
    (ItemSwordLv3, 0x5D7178),
    (ItemSwordLv4, 0x5D7178),
    (LiverPurple, 0x5D762C),
    (LiverYellow, 0x5D7640),
    (LiverBlue, 0x5D7654),
    (MessageBottle, 0x5D76A0),
    (Item::Pouch, 0x5D7734),
    (ItemBowLight, 0x5D776C),
    (HeartContainer, 0x5D7B7C),
    (HeartPiece, 0x5D7B94),
];

const ACTOR_NAMES: [(Item, &str); 108] = [
    (RupeeG, "RupeeG"),
    (RupeeB, "RupeeB"),
    (RupeeR, "RupeeR"),
    (RupeePurple, "RupeeP"),
    (KeyBoss, "KeyBoss"),
    (TriforceCourage, "BadgeBee"),
    (Compass, "Compass"),
    (ItemKandelaar, "GtEvKandelaar"),
    (ItemKandelaarLv2, "GtEvKandelaar"),
    (ItemMizukaki, "GtEvFin"),
    (RingRental, "RingRental"),
    (RingHekiga, "RingRental"),
    (ItemBell, "GtEvBell"),
    (PowerGlove, "GtEvGloveA"),
    (ItemInsectNet, "GtEvNet"),
    (ItemInsectNetLv2, "GtEvNet"),
    (BadgeBee, "BadgeBee"),
    (ClothesBlue, "GtEvCloth"),
    (Heart, "Heart"),
    (HyruleShield, "GtEvShieldB"),
    (Item::OreYellow, "OreSword"),
    (Item::OreGreen, "OreSword"),
    (Item::OreBlue, "OreSword"),
    (GanbariPowerUp, "PowerUp"),
    (DashBoots, "GtEvBoots"),
    (Item::OreRed, "OreSword"),
    (ItemIceRodLv2, "GtEvRodIceB"),
    (ItemSandRodLv2, "GtEvRodSandB"),
    (ItemTornadeRodLv2, "GtEvTornadoB"),
    (ItemBombLv2, "BombM"),
    (ItemFireRodLv2, "GtEvRodFireB"),
    (ItemHookShotLv2, "GtEvHookshotB"),
    (ItemBoomerangLv2, "GtEvBoomerangB"),
    (ItemHammerLv2, "GtEvHammerB"),
    (ItemBowLv2, "GtEvBowB"),
    (Milk, "GtEvBottleMedicine"),
    (MilkMatured, "GtEvBottleMedicine"),
    (Kinsta, "KinSta"),
    (PendantPower, "Pendant"),
    (PendantWisdom, "Pendant"),
    (PendantCourage, "Pendant"),
    (ZeldaAmulet, "Pendant"),
    (Item::Empty, "KeyBoss"),
    (EscapeFruit, "FruitEscape"),
    (StopFruit, "FruitStop"),
    (SpecialMove, "SwordD"),
    (Fairy, "GtEvBottleFairy"),
    (Bee, "GtEvBottleBee"),
    (GoldenBee, "GtEvBottleBee"),
    (SmallKeyHyrule, "KeySmall"),
    (SmallKeyEastern, "KeySmall"),
    (SmallKeyGales, "KeySmall"),
    (SmallKeyHera, "KeySmall"),
    (SmallKeyLorule, "KeySmall"),
    (SmallKeyDark, "KeySmall"),
    (SmallKeySwamp, "KeySmall"),
    (SmallKeySkull, "KeySmall"),
    (SmallKeyThieves, "KeySmall"),
    (SmallKeyIce, "KeySmall"),
    (SmallKeyDesert, "KeySmall"),
    (SmallKeyTurtle, "KeySmall"),
    (SmallKeyCastle, "KeySmall"),
    (BigKeyEastern, "KeyBoss"),
    (BigKeyGales, "KeyBoss"),
    (BigKeyHera, "KeyBoss"),
    (BigKeyDark, "KeyBoss"),
    (BigKeySwamp, "KeyBoss"),
    (BigKeySkull, "KeyBoss"),
    (BigKeyThieves, "KeyBoss"),
    (BigKeyIce, "KeyBoss"),
    (BigKeyDesert, "KeyBoss"),
    (BigKeyTurtle, "KeyBoss"),
    (CompassEastern, "Compass"),
    (CompassGales, "Compass"),
    (CompassHera, "Compass"),
    (CompassDark, "Compass"),
    (CompassSwamp, "Compass"),
    (CompassSkull, "Compass"),
    (CompassThieves, "Compass"),
    (CompassIce, "Compass"),
    (CompassDesert, "Compass"),
    (CompassTurtle, "Compass"),
    (CompassCastle, "Compass"),
    (Item::UpgradeIceRod, "GtEvRodIceB"),
    (Item::UpgradeSandRod, "GtEvRodSandB"),
    (Item::UpgradeTornadoRod, "GtEvTornadoB"),
    (Item::UpgradeBombs, "BombM"),
    (Item::UpgradeFireRod, "GtEvRodFireB"),
    (Item::UpgradeHookshot, "GtEvHookshotB"),
    (Item::UpgradeBoomerang, "GtEvBoomerangB"),
    (Item::UpgradeHammer, "GtEvHammerB"),
    (Item::UpgradeBow, "GtEvBowB"),
    (Item::UpgradeLamp, "GtEvKandelaar"),
    (Item::UpgradeNet, "GtEvNet"),
    (KeyRingHyrule, "KeySmall"),
    (KeyRingEastern, "KeySmall"),
    (KeyRingGales, "KeySmall"),
    (KeyRingHera, "KeySmall"),
    (KeyRingLorule, "KeySmall"),
    (KeyRingDark, "KeySmall"),
    (KeyRingSwamp, "KeySmall"),
    (KeyRingSkull, "KeySmall"),
    (KeyRingThieves, "KeySmall"),
    (KeyRingIce, "KeySmall"),
    (KeyRingDesert, "KeySmall"),
    (KeyRingTurtle, "KeySmall"),
    (KeyRingCastle, "KeySmall"),
    (Item::BeeTrap, "GtEvBottleBee"),
];

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

const ITEM_NAME_OFFSETS: [(Item, u32); 20] = [
    (ItemBomb, 0x6F9A9A),
    (ItemSandRod, 0x6F9AD0),
    (ItemIceRod, 0x6F9AE2),
    (ItemTornadeRod, 0x6F9AF3),
    (ItemFireRod, 0x6F9B08),
    (LiverPurple, 0x6F9B55),
    (ItemBottle, 0x6F9B6C),
    (LiverBlue, 0x6F9B94),
    (ItemBoomerang, 0x6F9BA9),
    (ItemHammer, 0x6F9CCC),
    (ItemHookShot, 0x6F9CDD),
    (ItemBow, 0x6F9D08),         // item_name_bow
    (LiverYellow, 0x6F9D2F),     // item_name_liver_yellow
    (ItemStoneBeauty, 0x6F9D56), // item_name_stonebeauty
    (RupeeR, 0x6f9c2f),          // item_name_bfirerod_rental
    (RupeeG, 0x6f9c13),          // item_name_tornaderod_rental
    (RupeeB, 0x6f9bfb),          // item_name_icerod_rental
    (RupeePurple, 0x6f9c49),     // item_name_boomerang_rental
    (RupeeSilver, 0x6f9c7c),     // item_name_hookshot_rental
    (RupeeGold, 0x6f9be2),       // item_name_sandrod_rental
];

const ITEM_NAMES: [(Item, &str); 116] = [
    (BadgeBee, "beebadge"),
    (Compass, "compass"),
    (ItemBell, "bell"),
    (ItemBowLight, "bow_light"),
    (RingRental, "bracelet"),
    (RingHekiga, "bracelet"),
    (ClothesBlue, "clothes_blue"),
    (EscapeFruit, "doron"),
    (StopFruit, "durian"),
    (GanbariPowerUp, "ganbari_power_up"),
    (HeartContainer, "heartcontioner"),
    (HeartPiece, "heartpiece"),
    (Item::HintGlasses, "hintglass"),
    (HyruleShield, "hyrule_shield"),
    (KeyBoss, "keyboss"),
    (TriforceCourage, "triforce_courage"),
    (KeySmall, "keysmall"),
    (Kinsta, "kinsta"),
    (ItemKandelaar, "lantern"),
    (ItemKandelaarLv2, "lantern_lv2"),
    (ItemSwordLv1, "mastersword"),
    (ItemSwordLv2, "mastersword"),
    (ItemSwordLv3, "mastersword"),
    (ItemSwordLv4, "mastersword"),
    (Item::Empty, "gamecoin"),
    (MessageBottle, "messagebottle"),
    (Milk, "milk"),
    (MilkMatured, "milk_matured"),
    (ItemInsectNet, "net"),
    (ItemInsectNetLv2, "net_lv2"),
    (Item::OreYellow, "ore"),
    (Item::OreGreen, "ore"),
    (Item::OreBlue, "ore"),
    (Item::OreRed, "ore"),
    (DashBoots, "pegasus"),
    (Heart, "potshop_heart"),
    (PendantCourage, "courage"),
    (PendantPower, "power"),
    (PendantWisdom, "wisdom"),
    (Item::Pouch, "pouch"),
    (PowerGlove, "powergloves"),
    (ItemShield, "shield"),
    (SpecialMove, "special_move"),
    (ItemBombLv2, "bomb_LV2"),
    (ItemBoomerangLv2, "boomerang_LV2"),
    (ItemBowLv2, "bow_LV2"),
    (ItemFireRodLv2, "firerod_LV2"),
    (ItemHammerLv2, "hammer_LV2"),
    (ItemHookShotLv2, "hookshot_LV2"),
    (ItemIceRodLv2, "icerod_LV2"),
    (ItemSandRodLv2, "sandrod_LV2"),
    (ItemTornadeRodLv2, "tornaderod_LV2"),
    (ItemMizukaki, "web"),
    (ZeldaAmulet, "charm"),
    (Fairy, "fairy"),
    (Bee, "bee"),
    (GoldenBee, "goldenbee"),
    (SmallKeyHyrule, "small_key_hyrule"),
    (SmallKeyEastern, "small_key_eastern"),
    (SmallKeyGales, "small_key_gales"),
    (SmallKeyHera, "small_key_hera"),
    (SmallKeyLorule, "small_key_lorule"),
    (SmallKeyDark, "small_key_dark"),
    (SmallKeySwamp, "small_key_swamp"),
    (SmallKeySkull, "small_key_skull"),
    (SmallKeyThieves, "small_key_thieves"),
    (SmallKeyIce, "small_key_ice"),
    (SmallKeyDesert, "small_key_desert"),
    (SmallKeyTurtle, "small_key_turtle"),
    (SmallKeyCastle, "small_key_castle"),
    (BigKeyEastern, "big_key_eastern"),
    (BigKeyGales, "big_key_gales"),
    (BigKeyHera, "big_key_hera"),
    (BigKeyDark, "big_key_dark"),
    (BigKeySwamp, "big_key_swamp"),
    (BigKeySkull, "big_key_skull"),
    (BigKeyThieves, "big_key_thieves"),
    (BigKeyIce, "big_key_ice"),
    (BigKeyDesert, "big_key_desert"),
    (BigKeyTurtle, "big_key_turtle"),
    (CompassEastern, "compass_eastern"),
    (CompassGales, "compass_gales"),
    (CompassHera, "compass_hera"),
    (CompassDark, "compass_dark"),
    (CompassSwamp, "compass_swamp"),
    (CompassSkull, "compass_skull"),
    (CompassThieves, "compass_thieves"),
    (CompassIce, "compass_ice"),
    (CompassDesert, "compass_desert"),
    (CompassTurtle, "compass_turtle"),
    (CompassCastle, "compass_castle"),
    (Item::UpgradeIceRod, "upgrade_ice_rod"),
    (Item::UpgradeSandRod, "upgrade_sand_rod"),
    (Item::UpgradeTornadoRod, "upgrade_tornado_rod"),
    (Item::UpgradeBombs, "upgrade_bombs"),
    (Item::UpgradeFireRod, "upgrade_fire_rod"),
    (Item::UpgradeHookshot, "upgrade_hookshot"),
    (Item::UpgradeBoomerang, "upgrade_boomerang"),
    (Item::UpgradeHammer, "upgrade_hammer"),
    (Item::UpgradeBow, "upgrade_bow"),
    (Item::UpgradeLamp, "upgrade_lamp"),
    (Item::UpgradeNet, "upgrade_net"),
    (KeyRingHyrule, "key_ring_hyrule"),
    (KeyRingEastern, "key_ring_eastern"),
    (KeyRingGales, "key_ring_gales"),
    (KeyRingHera, "key_ring_hera"),
    (KeyRingLorule, "key_ring_lorule"),
    (KeyRingDark, "key_ring_dark"),
    (KeyRingSwamp, "key_ring_swamp"),
    (KeyRingSkull, "key_ring_skull"),
    (KeyRingThieves, "key_ring_thieves"),
    (KeyRingIce, "key_ring_ice"),
    (KeyRingDesert, "key_ring_desert"),
    (KeyRingTurtle, "key_ring_turtle"),
    (KeyRingCastle, "key_ring_castle"),
    (Item::BeeTrap, "bee_trap"),
];

const EVENT_FLAG_PTR: u32 = 0x70B728;
const FN_GET_ITEM_LEVEL: u32 = 0x55696C;
const FN_GET_EVENT_FLAG: u32 = 0x584B94;
const FN_SET_EVENT_FLAG: u32 = 0x4CDF40;

/// r0: PlayerObjectSingleton <br />
/// r1: flag index
// const FN_GET_LOCAL_FLAG_3: u32 = 0x52a05c;

/// r0: PlayerObjectSingleton <br />
/// r1: flag index <br />
/// r2: new flag value (0 or 1)
// const FN_SET_LOCAL_FLAG_3: u32 = 0x1bb724;

const MAP_MANAGER: u32 = 0x70c8e0;
// const PTR_MAP_MANAGER_INSTANCE: u32 = 0x27320c;
const PLAYER_OBJECT_SINGLETON: u32 = 0x70FB60;
const GAME_MANAGER: u32 = 0x709DF8;
const SAVE_MANAGER: u32 = 0x711de8;
const VTABLE_STRING: u32 = 0x6F5988;
const VTABLE_FIXED_STRING_10: u32 = 0x6f5b94;
const FN_STRING_FORMAT: u32 = 0x4994b0;
const FN_VECTOR_ADD: u32 = 0x115c54;
