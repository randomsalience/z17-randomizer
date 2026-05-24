use crate::patch::Patcher;
use crate::Result;
use crate::SeedInfo;
use game::Course::{self, *};
use modinfo::Settings;

/// Add Actors to scenes that don't originally have them
pub fn patch(patcher: &mut Patcher, seed_info: &SeedInfo) -> Result<()> {
    patch_dev_stuff(patcher, seed_info)?;

    patch_letter_in_a_bottle(patcher)?;
    patch_bow_of_light_hint(patcher, &seed_info.settings)?;

    Ok(())
}

#[allow(unused_variables)]
fn patch_dev_stuff(patcher: &mut Patcher, seed_info: &SeedInfo) -> Result<()> {
    if !seed_info.settings.dev_mode {
        return Ok(());
    }

    // Add chest actor to Maiamai cave
    // let chest = patcher.scene(game::Course::DungeonHera, 0)?.actors().get_actor_bch("TreasureBoxS")?;
    // patcher.scene(game::Course::CaveLight, 14)?.actors_mut().add(chest)?;

    Ok(())
}

/// Add Hint Ghost to Hilda's Study to give out Bow of Light Hint
fn patch_bow_of_light_hint(patcher: &mut Patcher, settings: &Settings) -> Result<()> {
    if !settings.progressive_bow_of_light {
        let hint_ghost = patcher.scene(IndoorDark, 15)?.actors().get_actor_bch("HintGhost")?;
        patcher.scene(IndoorDark, 4)?.actors_mut().add(hint_ghost)?;
    }
    Ok(())
}

/// Add Heart Piece actor to vanilla Letter in a Bottle area
fn patch_letter_in_a_bottle(patcher: &mut Patcher) -> Result<()> {
    let heart_piece = patcher.scene(FieldLight, 29)?.actors().get_actor_bch("HeartPiece")?;
    patcher.scene(FieldLight, 35)?.actors_mut().add(heart_piece)?;
    Ok(())
}

pub const HEART_PIECES: [(&str, Course, u16, u16); 21] = [
    ("[TR] Left Balcony", FieldDark, 35, 54),
    ("Fire Cave Pillar", CaveLight, 25, 9),
    ("Floating Island", FieldLight, 4, 25),
    ("Spectacle Rock", FieldLight, 3, 302),
    ("Eastern Ruins Cave", CaveLight, 29, 10),
    ("Eastern Ruins Peg Circle", FieldLight, 30, 41),
    ("Blacksmith Cave", CaveLight, 16, 1),
    ("Blacksmith Ledge", FieldLight, 17, 95),
    ("Hyrule Castle Rocks", FieldLight, 18, 209),
    ("Kakariko Well (Top)", CaveLight, 4, 8),
    ("Lake Hylia Eastern Shore", FieldLight, 36, 38),
    ("Lost Woods Alcove", FieldLight, 1, 46),
    ("Graveyard Ledge Cave", CaveLight, 5, 2),
    ("Waterfall Cave", CaveLight, 13, 103),
    ("[HS] Ledge", CaveLight, 18, 31),
    ("Southern Ruins Pillar Cave", FieldLight, 33, 313),
    ("Dark Maze Ledge", FieldDark, 20, 172),
    ("Swamp Cave (Middle)", CaveDark, 3, 8),
    ("Misery Mire Ledge", FieldDark, 31, 82),
    ("Destroyed House", FieldDark, 2, 144),
    ("n-Shaped House", FieldDark, 16, 124),
];

pub const HEART_CONTAINERS: [(&str, Course, u16, u16); 10] = [
    ("[PD] Gemesaur King", DungeonDark, 1, 119),
    ("[DP] Zaganaga", FieldDark, 31, 83),
    ("[EP] Yuga (2)", DungeonEast, 3, 94),
    ("[HG] Margomill", DungeonWind, 3, 458),
    ("[IR] Dharkstare", DungeonIce, 1, 554),
    ("[SW] Knucklemaster", DungeonDokuro, 2, 404),
    ("[SP] Arrghus", DungeonWater, 1, 129),
    ("[TT] Stalblind", IndoorDark, 15, 12),
    ("[TH] Moldorm", DungeonHera, 1, 772),
    ("[TR] Grinexx", DungeonKame, 3, 6),
];

pub const SMALL_KEYS: [(&str, Course, u16, u16); 15] = [
    ("[PD] (1F) Left Pit", DungeonDark, 2, 25),
    ("[PD] (B1) Fall From 1F", DungeonDark, 1, 26),
    ("[PD] (B1) Helmasaur Room", DungeonDark, 1, 281),
    ("[HG] (2F) Narrow Ledge", DungeonWind, 2, 180),
    ("[HG] (2F) Fire Ring", DungeonWind, 2, 97),
    ("[IR] (B2) Ice Pillar", DungeonIce, 1, 1057),
    ("[IR] (B1) Narrow Ledge", DungeonIce, 1, 98),
    ("[SP] (B1) Raft Room (Pillar)", DungeonWater, 2, 116),
    ("[SP] (B1) Waterfall Room", DungeonWater, 2, 219),
    ("[TH] (3F) Platform", DungeonHera, 1, 244),
    ("[TH] (6F) Left Mole", DungeonHera, 1, 334),
    ("[TR] (1F) Northwest Room", DungeonKame, 1, 153),
    ("[TR] (1F) Northeast Ledge", DungeonKame, 1, 243),
    ("[TR] (B1) Northeast Room", DungeonKame, 2, 53),
    ("[LS] Ledge", AttractionDark, 2, 31),
];

pub const RUPEES: [(&str, Course, u16, u16); 4] = [
    ("[PD] (2F) South Hidden Room", DungeonDark, 3, 166),
    ("[TR] (B1) Under Center", DungeonKame, 2, 211),
    ("[TR] (1F) Under Center", DungeonKame, 1, 114),
    ("Cucco Mini-Dungeon", AttractionLight, 3, 9),
];
