use crate::filler::cracks::Crack;
use crate::filler::filler_item::{Randomizable, Vane};
use crate::regions;
use crate::{patch::util::*, Error, Result, SeedInfo};
use crate::patch::actors::{HEART_PIECES, HEART_CONTAINERS, SMALL_KEYS, RUPEES};
use code::Code;
use fs_extra::dir::CopyOptions;
use game::{
    Course::{self as CourseId, *},
    Item, World,
};
use log::{debug, error, info};
use macros::fail;
use modinfo::settings::weather_vanes::WeatherVanes::*;
use path_absolutize::*;
use pyo3::prelude::*;
use rom::byaml::scene_env::SceneEnvFile;
use rom::flag::Flag;
use rom::scene::{Transform, Vec3};
use rom::{
    flow::FlowMut,
    h3d,
    scene::{Arg, Obj, Rail, SceneMeta},
    File, IntoBytes, Language, Rom, Scene,
};
use serde::Serialize;
use std::ops::Add;
use std::{collections::HashMap, ffi::CString, fs, path::Path};
use tempfile::tempdir;
use try_insert_ext::EntryInsertExt;

mod actors;
mod byaml;
mod code;
mod demo;
pub mod lms;
mod messages;
mod prizes;
pub mod util;

#[non_exhaustive]
pub struct DungeonPrizes {
    ep_prize: Randomizable,
    hg_prize: Randomizable,
    th_prize: Randomizable,
    pd_prize: Randomizable,
    sp_prize: Randomizable,
    sw_prize: Randomizable,
    tt_prize: Randomizable,
    tr_prize: Randomizable,
    dp_prize: Randomizable,
    ir_prize: Randomizable,
}

#[derive(Debug)]
pub struct Patcher {
    game: Rom,
    boot: Language,
    rentals: [Item; 9],
    merchant: [Item; 3],
    courses: HashMap<CourseId, Course>,
}

impl Patcher {
    pub fn new(game: Rom) -> Result<Self> {
        let boot = game.boot()?;
        Ok(Self {
            game,
            boot,
            rentals: [Item::KeySmall; 9],
            merchant: [Item::KeySmall; 3],
            courses: Default::default(),
        })
    }

    fn add_obj(&mut self, id: CourseId, stage_index: u16, obj: Obj) {
        self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut().add_obj(obj);
    }

    fn add_rail(&mut self, id: CourseId, stage_index: u16, rail: Rail) {
        self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut().add_rail(rail);
    }

    #[allow(unused)]
    fn add_system(&mut self, id: CourseId, stage_index: u16, obj: Obj) {
        self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut().add_system(obj);
    }

    /// Finds the lowest UNQ and SER
    fn find_objs_unq_ser(&mut self, id: CourseId, stage_index: u16) -> (u16, Option<u16>) {
        let unq_ser = (self.find_objs_unq(id, stage_index), self.find_objs_ser(id, stage_index));
        debug!("Unused {:?}{} Objs UNQ: {}, SER: {:?}", id, stage_index, unq_ser.0, unq_ser.1);
        unq_ser
    }

    /// Finds the lowest currently unused UNQ
    fn find_objs_unq(&mut self, id: CourseId, stage_index: u16) -> u16 {
        self.scene(id, stage_index - 1).unwrap().stage().get().find_objs_unq()
    }

    /// Finds the lowest currently unused SER
    fn find_objs_ser(&mut self, id: CourseId, stage_index: u16) -> Option<u16> {
        Some(self.scene(id, stage_index - 1).unwrap().stage().get().find_objs_ser())
    }

    /// Finds the lowest currently unused Rails UNQ
    #[allow(unused)]
    fn find_rails_unq(&mut self, id: CourseId, stage_index: u16) -> u16 {
        let unq = self.scene(id, stage_index - 1).unwrap().stage().get().find_rails_unq();
        debug!("Unused {:?}{} Rails UNQ: {}", id, stage_index, unq);
        unq
    }

    /// Finds the lowest UNQ and SER
    #[allow(unused)]
    fn find_system_unq_ser(&mut self, id: CourseId, stage_index: u16) -> (u16, Option<u16>) {
        let unq_ser = (self.find_system_unq(id, stage_index), self.find_system_ser(id, stage_index));
        debug!("Unused {:?}{} System UNQ: {}, SER: {:?}", id, stage_index, unq_ser.0, unq_ser.1);
        unq_ser
    }

    /// Finds the lowest currently unused UNQ
    #[allow(unused)]
    fn find_system_unq(&mut self, id: CourseId, stage_index: u16) -> u16 {
        self.scene(id, stage_index - 1).unwrap().stage().get().find_system_unq()
    }

    /// Finds the lowest currently unused SER
    #[allow(unused)]
    fn find_system_ser(&mut self, id: CourseId, stage_index: u16) -> Option<u16> {
        Some(self.scene(id, stage_index - 1).unwrap().stage().get().find_system_ser())
    }

    fn read_obj(&mut self, id: CourseId, stage_index: u16, unq: u16) -> &Obj {
        let stage = self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut();
        stage.get_obj(unq).unwrap_or_else(|| {
            panic!(
                "Failed to read Crack Objs entry with UNQ: {} from World/Byaml/{:?}{}_stage.byaml",
                unq, id, stage_index
            )
        })
    }

    fn modify_objs<A>(&mut self, course_id: CourseId, stage_index: u16, actions: A)
    where
        A: Into<Vec<(u16, Box<dyn Fn(&mut Obj)>)>>,
    {
        let stage = self.scene(course_id, stage_index - 1).unwrap().stage_mut().get_mut();
        for (unq, action) in actions.into() {
            action(
                stage
                    .get_obj_mut(unq)
                    .ok_or_else(|| {
                        Error::game(format!(
                            "Could not find [Objs] UNQ {} in {}{}",
                            unq,
                            course_id.as_str(),
                            stage_index
                        ))
                    })
                    .unwrap(),
            );
        }
    }

    fn modify_rails<A>(&mut self, id: CourseId, stage_index: u16, actions: A)
    where
        A: Into<Vec<(u16, Box<dyn Fn(&mut Rail)>)>>,
    {
        let stage = self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut();
        for (unq, action) in actions.into() {
            action(
                stage
                    .get_rails_mut(unq)
                    .ok_or_else(|| {
                        Error::game(format!("Could not find [Rails] UNQ {} in {}{}", unq, id.as_str(), stage_index))
                    })
                    .unwrap(),
            );
        }
    }

    fn modify_system<A>(&mut self, id: CourseId, stage_index: u16, actions: A)
    where
        A: Into<Vec<(u16, Box<dyn Fn(&mut Obj)>)>>,
    {
        let stage = self.scene(id, stage_index - 1).unwrap().stage_mut().get_mut();
        for (unq, action) in actions.into() {
            action(
                stage
                    .get_system_mut(unq)
                    .ok_or_else(|| {
                        Error::game(format!("Could not find [System] UNQ {} in {}{}", unq, id.as_str(), stage_index))
                    })
                    .unwrap(),
            );
        }
    }

    fn load_course(game: &mut Rom, course: CourseId) -> Course {
        game.course(course)
            .language()
            .map(|load| Course {
                language: load,
                scenes: Default::default(),
                scene_meta: game.course(course).scene_meta(),
            })
            .unwrap()
    }

    fn course(&mut self, course: CourseId) -> Result<&mut Course> {
        let Self { game, courses, .. } = self;
        Ok(courses.entry(course).or_insert(Self::load_course(game, course)))
    }

    /// Subtract 1 from stage
    fn scene(&mut self, course: CourseId, stage: u16) -> Result<&mut Scene> {
        let Self { game, courses, .. } = self;
        courses
            .entry(course)
            .or_insert(Self::load_course(game, course))
            .scenes
            .entry(stage)
            .or_try_insert_with(|| game.course(course).scene(stage))
            .map_err(Into::into)
    }

    fn scene_meta(&mut self, course: CourseId) -> &mut SceneMeta {
        let Self { game, courses, .. } = self;
        let Course { scene_meta, .. } = courses.entry(course).or_insert(Self::load_course(game, course));
        scene_meta.as_mut().unwrap()
    }

    fn scene_env(&mut self) -> rom::Result<SceneEnvFile> {
        self.game.scene_env()
    }

    fn update(&mut self, (course, file): (CourseId, File<Vec<u8>>)) -> Result<()> {
        self.language(course)?.update(file)?;
        Ok(())
    }

    fn inject_msbf(&mut self, course: CourseId, msbf: Option<&(&str, File<Box<[u8]>>)>) -> Result<()> {
        if let Some((msbf_key, msbf_file)) = msbf {
            self.language(course)?.flow_inject(msbf_key, msbf_file.clone())?;
        }

        Ok(())
    }

    fn language<C>(&mut self, course: C) -> Result<&mut Language>
    where
        C: Into<Option<CourseId>>,
    {
        Ok(if let Some(course) = course.into() { &mut self.course(course)?.language } else { &mut self.boot })
    }

    fn flow<C>(&mut self, course: C) -> Result<rom::language::LoadedMut<'_, FlowMut<'_>>>
    where
        C: Into<Option<CourseId>>,
    {
        Ok(self.language(course)?.flow_mut())
    }

    /// Perform patching operations for each patch, depending on what type it is.
    fn apply<F>(&mut self, patch: Patch, seed_info: &SeedInfo, filler_item: F, loc_name: &str) -> Result<()>
    where
        F: Into<Option<Randomizable>> + Clone,
    {
        match patch {
            Patch::Chest { course, stage, unq } => {
                let is_big = seed_info.is_major_location(loc_name, false);
                self.prep_chest(filler_item.into().unwrap(), course, stage, unq, is_big, seed_info)?;
            },
            Patch::BigChest { course, stage, unq } => {
                let is_big = seed_info.is_major_location(loc_name, true);
                self.prep_chest(filler_item.into().unwrap(), course, stage, unq, is_big, seed_info)?;
            },
            Patch::Heart { course, scene, unq }
            | Patch::Key { course, scene, unq }
            | Patch::SilverRupee { course, scene, unq }
            | Patch::GoldRupee { course, scene, unq } => {
                self.parse_args(course, scene, unq).1 = filler_item.into().unwrap().as_item_index() as i32;
            },
            Patch::Crack { course, scene, unq, crack } => {
                self.patch_crack(course, scene + 1, unq, crack, seed_info)?;
            },
            Patch::WeatherVane { course, scene, unq, vane } => {
                self.patch_weather_vane(filler_item.into().unwrap(), course, scene, unq, vane, seed_info)?;
            },
            Patch::Maiamai { course, scene, unq } => {
                self.parse_args(course, scene, unq).2 = filler_item.into().unwrap().as_item_index() as i32;
            },
            Patch::Event { course, name, index } => {
                self.flow(course)?
                    .get_mut(name)
                    .ok_or_else(|| Error::game(format!("File not found: {name}")))??
                    .get_mut()
                    .get_mut(index)
                    .ok_or_else(|| {
                        Error::game(format!(
                            "{}/{} [{}] not found",
                            course.as_ref().map(CourseId::as_str).unwrap_or("Boot"),
                            name,
                            index
                        ))
                    })?
                    .set_value(filler_item.into().unwrap().as_item_index());
            },
            Patch::Shop(Shop::Ravio(index)) => {
                self.rentals[index as usize] = filler_item.into().unwrap().as_item().unwrap().to_game_item();
            },
            Patch::Shop(Shop::Merchant(index)) => {
                self.merchant[index as usize] = filler_item.into().unwrap().as_item().unwrap().to_game_item();
            },
            Patch::Multi(patches) => {
                for patch in patches {
                    self.apply(patch, seed_info, filler_item.clone(), loc_name)?;
                }
            },
            Patch::None => {},
        }
        Ok(())
    }

    fn parse_args(&mut self, course: CourseId, stage: u16, unq: u16) -> &mut Arg {
        self.scene(course, stage)
            .unwrap()
            .stage_mut()
            .get_mut()
            .get_obj_mut(unq)
            .ok_or_else(|| Error::game(format!("{}{} [{}] not found", course.as_str(), stage + 1, unq)))
            .unwrap()
            .arg_mut()
    }

    /// Patch Chest actors and swap their size if needed for CSMC.
    fn prep_chest(
        &mut self, item: Randomizable, course: CourseId, stage: u16, unq: u16, is_big: bool,
        SeedInfo { settings, archipelago_info, .. }: &SeedInfo,
    ) -> Result<()> {
        // Set contents
        self.parse_args(course, stage, unq).0 = item.as_item_index() as i32;

        let small_chest = (35, "TreasureBoxS");
        let large_chest = (34, "TreasureBoxL");

        let chest_data = if settings.chest_size_matches_contents && archipelago_info.is_none() {
            if item.is_major_item() {
                large_chest
            } else {
                small_chest
            }
        } else if is_big {
            large_chest
        } else {
            small_chest
        };

        // Forcibly set ID
        self.scene(course, stage).unwrap().stage_mut().get_mut().get_obj_mut(unq).unwrap().set_id(chest_data.0);

        // Add Actor if scene doesn't already have it
        if !self.scene(course, stage).unwrap().actors().contains(chest_data.1) {
            debug!("Adding {} to {}{}", chest_data.1, course.as_str(), stage + 1);
            let actor = self.scene(DungeonHera, 0)?.actors().get_actor_bch(chest_data.1)?;
            self.scene(course, stage).unwrap().actors_mut().add(actor)?;
        }

        Ok(())
    }

    /// Cracks!
    fn patch_crack(
        &mut self, course: CourseId, scene: u16, unq: u16, here_crack: Crack, seed_info: &SeedInfo,
    ) -> Result<()> {
        // Collect info about this crack's new destination
        let there_crack =
            seed_info.crack_map.get(&here_crack).unwrap_or_else(|| panic!("No crack_map entry for: {:?}", here_crack));
        let there_flag = there_crack.get_flag();
        let there_sp = there_crack.get_spawn_point();

        // Crack type
        let crack_type = if here_crack.get_world() == there_crack.get_world() {
            here_crack.get_reverse_type()
        } else {
            here_crack.get_type()
        };

        // Enable Flag - if it's the HC Crack or its pair leave it open, else use the Quake Flag.
        let enable_flag = if here_crack == Crack::HyruleCastle
            || here_crack == *seed_info.crack_map.get(&Crack::HyruleCastle).unwrap()
        {
            Flag::ZERO_ZERO
        } else {
            Flag::QUAKE
        };

        // Apply the patch
        self.modify_objs(
            course,
            scene,
            [call(unq, move |obj| {
                obj.redirect(there_sp);
                obj.arg.2 = crack_type;
                obj.set_active_flag(there_flag);
                obj.set_inactive_flag(here_crack.get_flag());
                obj.set_enable_flag(enable_flag);
            })],
        );

        Ok(())
    }

    /// For cracks that now lead to blocked cracks, add matching blockages on that side of the crack
    #[allow(unused)]
    fn block_cracks(
        &mut self, course: CourseId, scene: u16, unq: u16, here_crack: Crack, there_crack: Crack,
    ) -> Result<()> {
        // Already blocked Cracks can be left alone
        match here_crack {
            Crack::DesertNorth
            | Crack::EasternRuinsSE
            | Crack::DarkRuinsSE
            | Crack::GraveyardLedgeLorule
            | Crack::HyruleCastle => {
                return Ok(());
            },
            _ => {},
        }

        match there_crack {
            // Blocked cracks
            Crack::DesertNorth | Crack::EasternRuinsSE | Crack::GraveyardLedgeLorule | Crack::DarkRuinsSE => {
                // Read vanilla Crack data, use it to add blockages relative to Cracks
                let obj_crack = self.read_obj(course, scene, unq);
                let clp = obj_crack.clp;
                let translate = obj_crack.srt.translate;
                let (wall_unq, wall_ser) = self.find_objs_unq_ser(course, scene);

                // Attach blockage to Crack (keeps light/dark fog from appearing until breakage is gone)
                self.modify_objs(course, scene, [call(unq, move |obj| obj.lnk = vec![(wall_unq, 0, 0)])]);

                // Different actors for WallBreakFieldLight and WallBreakFieldDark
                match there_crack {
                    Crack::DesertNorth | Crack::EasternRuinsSE => {
                        self.add_obj(
                            course,
                            scene,
                            Obj::wall_break_field_light(here_crack.get_flag(), clp, wall_ser, wall_unq, translate),
                        );
                        self.copy_bch("WallBreakFieldLight", (FieldLight, 30), (course, scene))?;
                    },
                    Crack::GraveyardLedgeLorule | Crack::DarkRuinsSE => {
                        self.add_obj(
                            course,
                            scene,
                            Obj::wall_break_field_dark(here_crack.get_flag(), clp, wall_ser, wall_unq, translate),
                        );
                        self.copy_bch("WallBreakFieldDark", (FieldDark, 30), (course, scene))?;
                    },
                    _ => unreachable!(),
                }
            },
            // Hyrule Castle Curtain
            Crack::HyruleCastle => {
                // Read vanilla Crack data, use it to add the Curtain and WallDisableIn actors
                let obj_crack = self.read_obj(course, scene, unq);
                let clp = obj_crack.clp;
                let t_curtain = obj_crack.srt.translate.add(Vec3 { x: 0.0, y: 0.0, z: 1.0 });
                let t_wall = obj_crack.srt.translate.add(Vec3 { x: -0.02731, y: -0.00001, z: 0.96672 });
                let (curtain_unq, curtain_ser) = self.find_objs_unq_ser(course, scene);

                // Attach Curtain to Crack (keeps light from appearing until Curtain is gone)
                self.modify_objs(course, scene, [call(unq, move |obj| obj.lnk = vec![(curtain_unq, 0, 0)])]);

                let flag = here_crack.get_flag();

                // Curtain
                self.add_obj(
                    course,
                    scene,
                    Obj {
                        arg: Arg(0, 0, 0, 0, flag.get_type(), 0, flag.get_value(), 0, 0, 0, 0, 0, 0, 0.0),
                        clp,
                        flg: (0, 0, 0, 0),
                        id: 550,
                        lnk: vec![],
                        nme: None,
                        ril: vec![],
                        ser: curtain_ser,
                        srt: Transform {
                            scale: Vec3 { x: 1.0, y: 16.0, z: 1.0 },
                            rotate: Vec3 { x: 339.3735, y: 0.0, z: 0.0 },
                            translate: t_curtain,
                        },
                        typ: 1,
                        unq: curtain_unq,
                    },
                );

                // WallDisableIn
                let (disable_merge_unq, disable_merge_ser) = self.find_objs_unq_ser(course, scene);
                self.add_obj(
                    course,
                    scene,
                    Obj {
                        arg: Arg(0, 0, 0, 0, flag.get_type(), 0, flag.get_value(), 0, 0, 0, 0, 0, 0, 0.0),
                        clp,
                        flg: (0, flag.get_type(), 0, flag.get_value()),
                        id: 568,
                        lnk: vec![],
                        nme: None,
                        ril: vec![],
                        ser: disable_merge_ser,
                        srt: Transform {
                            scale: Vec3 { x: 5.25577, y: 1.0, z: 2.43051 },
                            rotate: Vec3::ZERO,
                            translate: t_wall,
                        },
                        typ: 6,
                        unq: disable_merge_unq,
                    },
                );

                self.copy_bch("Curtain", (IndoorLight, 7), (course, scene))?;
            },
            _ => return Ok(()),
        };

        Ok(())
    }

    /// Copies an actor .bch file from a given stage to a different stage, if it doesn't already have it.
    fn copy_bch(
        &mut self, actor_name: &str, (from_course, from_stage): (CourseId, u16), (to_course, to_stage): (CourseId, u16),
    ) -> Result<()> {
        if !self.scene(to_course, to_stage - 1).unwrap().actors().contains(actor_name) {
            debug!(
                "Copying {} from {}{} to {}{}",
                actor_name,
                from_course.as_str(),
                from_stage,
                to_course.as_str(),
                to_stage
            );
            let actor = self.scene(from_course, from_stage - 1)?.actors().get_actor_bch(actor_name)?;
            self.scene(to_course, to_stage - 1).unwrap().actors_mut().add(actor)?;
        }

        Ok(())
    }

    /// Weather Vanes
    fn patch_weather_vane(
        &mut self, item: Randomizable, course: CourseId, scene: u16, unq: u16, vane: Vane,
        SeedInfo { settings, .. }: &SeedInfo,
    ) -> Result<()> {
        if settings.weather_vanes != Shuffled {
            return Ok(());
        }

        let wv_flag = match item {
            Randomizable::Vane(vane) => vane.flag().get_value(),
            _ => unreachable!(),
        };

        // Set Weather Vane flag to randomized value
        self.parse_args(course, scene, unq).6 = wv_flag;

        let wv_world_dest = Vane::get_world(item.into());
        let wv_world_vane = Vane::get_world(vane);

        // Actor info for model swaps
        let wv_config_info = match wv_world_dest {
            World::Hyrule => (165, "Telephone", FieldLight),
            World::Lorule => (464, "TelephoneDark", FieldDark),
        };

        // Forcibly set ID
        self.scene(course, scene).unwrap().stage_mut().get_mut().get_obj_mut(unq).unwrap().set_id(wv_config_info.0);

        // Swap the Weather Vane model if it's from the opposite world
        if wv_world_vane != wv_world_dest {
            Self::copy_bch(self, wv_config_info.1, (wv_config_info.2, 16), (course, scene + 1))?;
        }
        //     Add Actor if scene doesn't already have it
        //     if !self.scene(course, scene).unwrap().actors().contains(wv_config_info.1) {
        //         debug!("Adding {} to {}{}", wv_config_info.1, course.as_str(), scene + 1);
        //         let actor =
        //             self.scene(wv_config_info.2, 15)?.actors().get_actor_bch(wv_config_info.1)?;
        //         self.scene(course, scene).unwrap().actors_mut().add(actor)?;
        //     }
        // }

        Ok(())
    }

    pub fn prepare(self, seed_info: &SeedInfo, py: Option<Python>) -> Result<Patches> {
        init_progress_bar(py);
        let result = self.prepare_inner(seed_info, py);
        destroy_progress_bar(py);
        result
    }

    fn prepare_inner(mut self, seed_info: &SeedInfo, py: Option<Python>) -> Result<Patches> {
        regions::patch(&mut self, seed_info)?;
        actors::patch(&mut self, seed_info)?;
        lms::msbf::patch(&mut self, seed_info)?;
        messages::patch_messages(&mut self, seed_info)?;
        let prizes = get_dungeon_prizes(&seed_info.layout);
        prizes::patch_dungeon_prizes(&mut self, &prizes);
        byaml::get_item::patch(&mut self)?;
        byaml::course::patch(&mut self, &prizes, seed_info);
        byaml::stage::patch(&mut self, seed_info)?;
        let scene_env_file = byaml::scene_env::patch(&mut self, &seed_info.settings);
        let cutscenes = demo::build_replacement_cutscenes(seed_info)?;

        let mut common_archive = self.game.common()?;
        let mut item_actors = HashMap::new();
        let mut get_item_actors = Vec::new();

        info!("Patching Item Actors...");
        for (item, get_item) in self.game.match_items_to_get_items() {
            if Item::SpecialMove.as_str().eq(&get_item.0) {
                // fixme hacky and gross
                let mut actor = common_archive.get_actor_bch("SwordD")?.clone();
                actor.rename(String::from("World/Actor/SwordD.bch"));
                item_actors.insert(item, actor);
            } else if let Some(mut actor) = get_item.actor(&self.game) {
                let mut get_item_actor = actor.clone();
                get_item_actor.rename(format!("World/GetItem/{}.bch", get_item.name()));
                get_item_actors.push(get_item_actor);
                actor.rename(format!("World/Actor/{}.bch", get_item.actor_name()?));
                item_actors.insert(item, actor);
            }
        }

        for (item, name, color3, color4) in [
            (Item::RupeeG, "RupeeG", [0x00, 0xb0, 0x1c, 0xae], [0x00, 0x2b, 0x1a, 0xff]),
            (Item::RupeeB, "RupeeB", [0x00, 0x0a, 0xff, 0xae], [0x00, 0x05, 0x36, 0xff]),
            (Item::RupeeR, "RupeeR", [0xff, 0x00, 0x00, 0xae], [0x1c, 0x00, 0x0f, 0xff]),
            (Item::RupeePurple, "RupeeP", [0x45, 0x00, 0xe6, 0xae], [0x30, 0x00, 0x29, 0xff]),
        ] {
            let mut actor = common_archive
                .get_actor_bch("Rupee")?
                .try_map(|data| h3d::Resource::from_bytes(&data))?;
            actor.rename(format!("World/Actor/{}.bch", name));
            let param = actor.get()
                .get_model(0)?
                .get_material(0)?
                .get_param()?;
            param.set_color(h3d::MaterialColor::Constant3, color3);
            param.set_color(h3d::MaterialColor::Constant4, color4);
            param.set_tev_color(h3d::TevStage::Stage1, color3)?;
            param.set_tev_color(h3d::TevStage::Stage2, color4)?;
            param.set_tev_color(h3d::TevStage::Stage4, color3)?;
            item_actors.insert(item, actor.into_bytes());
        }

        {
            let Self { ref rentals, ref merchant, ref mut courses, .. } = self;
            let your_house_actors = courses.get_mut(&IndoorLight).unwrap().scenes.get_mut(&0).unwrap().actors_mut();
            for actor in rentals.iter().filter_map(|item| item_actors.get(item)) {
                your_house_actors.add(actor.clone())?;
            }
            let kakariko_actors = courses.get_mut(&FieldLight).unwrap().scenes.get_mut(&15).unwrap().actors_mut();
            kakariko_actors.add(item_actors.get(&merchant[0]).unwrap().clone())?;
            kakariko_actors.add(item_actors.get(&merchant[2]).unwrap().clone())?;

            if seed_info.settings.change_freestanding_models {
                for data in [HEART_PIECES.iter(), HEART_CONTAINERS.iter(), SMALL_KEYS.iter(), RUPEES.iter()] {
                    for (name, _, _, _) in data {
                        let item = seed_info.layout.get_by_name(name).normalize();
                        common_archive.add(item_actors.get(&item).unwrap().clone())?;
                    }
                }
                common_archive.add(item_actors.get(&Item::HeartPiece).unwrap().clone())?;
                common_archive.add(item_actors.get(&Item::HeartContainer).unwrap().clone())?;
            }
        }

        
        info!("Patching Code...");
        let code = code::create(&self, seed_info);

        info!("Creating romfs...");
        let Self { game, boot, courses, .. } = self;
        let mut romfs = Files(vec![]);

        // Add Actors to Common Archive
        //let mut common = game.common().unwrap();
        //common.add(chest_large)?;
        //common.add(fresco_arrow)?; // Sorta works... but not really...
        //romfs.add(common.into_archive().unwrap());

        let mut count = 1;
        if scene_env_file.is_some() {
            count += 1;
        }
        for (_, Course { scenes, scene_meta, .. }) in &courses {
            count += 1;
            if scene_meta.is_some() {
                count += 1;
            }
            count += scenes.len();
        }
        count += cutscenes.len();
        let step = 1.0 / (count as f32);
        let mut progress = 0.0;

        romfs.add(boot.into_archive());
        update_progress_bar(&mut progress, step, py);
        if let Some(common) = common_archive.into_archive() {
            romfs.add(common);
        }
        update_progress_bar(&mut progress, step, py);
        if let Some(scene_env_file) = scene_env_file {
            romfs.add_serialize(scene_env_file.into_file());
            update_progress_bar(&mut progress, step, py);
        };
        for (_, Course { language, scenes, scene_meta }) in courses {
            romfs.add(language.into_archive());
            update_progress_bar(&mut progress, step, py);
            if let Some(scene_meta) = scene_meta {
                romfs.add_serialize(scene_meta.into_file());
                update_progress_bar(&mut progress, step, py);
            }
            for (_, scene) in scenes {
                let (actors, stage) = scene.into_files();
                if let Some(archive) = actors {
                    romfs.add(archive);
                }
                romfs.add_serialize(stage);
                update_progress_bar(&mut progress, step, py);
            }
        }
        for cutscene in cutscenes {
            romfs.add(cutscene);
            update_progress_bar(&mut progress, step, py);
        }
        for actor in get_item_actors {
            romfs.add(actor);
        }
        Ok(Patches { game, code, romfs })
    }
}

fn init_progress_bar(py: Option<Python>) {
    if let Some(py) = py {
        let _ = py.run(cr#"
import tkinter
from tkinter import ttk
root = tkinter.Tk()
root.title("Patching...")
progress_bar = ttk.Progressbar(maximum=100)
progress_bar.place(x=10, y=10, width=200)
root.geometry("220x50")
root.update()
"#,
            None,
            None
        );
    };
}

fn update_progress_bar(progress: &mut f32, step: f32, py: Option<Python>) {
    *progress += step;
    if let Some(py) = py {
        let _ = py.run(
            CString::new(format!(r#"
progress_bar['value'] = {}
root.update()"#,
                *progress * 99.9))
                .unwrap()
                .as_c_str(),
            None,
            None
        );
    }
}

fn destroy_progress_bar(py: Option<Python>) {
    if let Some(py) = py {
        let _ = py.run(c"root.destroy()", None, None);
    }
}

/// Research MSBF and MSBT Files
#[allow(unused)]
#[deprecated]
pub fn research_msbf_msbt<C>(
    patcher: &mut Patcher, msbf_course: C, msbf_file: &str, msbt_course: game::Course, msbt_file: &str, edotor: bool,
) where
    C: Into<Option<game::Course>>,
{
    let labels = messages::research(patcher, msbt_course, msbt_file, edotor);
    lms::msbf::research(patcher, msbf_course, msbf_file, labels, edotor);

    info!("Early Debug Exit");
    std::process::exit(0);
}

#[derive(Debug)]
pub struct Course {
    language: Language,
    scenes: HashMap<u16, Scene>,
    scene_meta: Option<SceneMeta>,
}

#[derive(Clone, Debug)]
pub enum Patch {
    Chest { course: CourseId, stage: u16, unq: u16 },
    BigChest { course: CourseId, stage: u16, unq: u16 },
    Event { course: Option<CourseId>, name: &'static str, index: u16 },
    Heart { course: CourseId, scene: u16, unq: u16 },
    Key { course: CourseId, scene: u16, unq: u16 },
    Maiamai { course: CourseId, scene: u16, unq: u16 },
    SilverRupee { course: CourseId, scene: u16, unq: u16 },
    GoldRupee { course: CourseId, scene: u16, unq: u16 },
    Crack { course: CourseId, scene: u16, unq: u16, crack: Crack },
    WeatherVane { course: CourseId, scene: u16, unq: u16, vane: Vane },
    Shop(Shop),
    Multi(Vec<Patch>),
    None, // Workaround until everything is shufflable
}

impl Patch {
    pub fn apply<F>(self, patcher: &mut Patcher, seed_info: &SeedInfo, filler_item: F, loc_name: &str) -> Result<()>
    where
        F: Into<Option<Randomizable>>,
    {
        patcher.apply(self, seed_info, filler_item.into(), loc_name)
    }
}

#[derive(Clone, Debug)]
pub enum Shop {
    Ravio(u8),
    Merchant(u8),
}

#[derive(Debug)]
pub struct Patches {
    game: Rom,
    code: Code,
    romfs: Files,
}

impl Patches {
    pub fn dump<P>(self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let temp = tempdir()?;
        let moddir = temp.path().join(format!("{:016X}", self.game.id()));
        let romfs = moddir.join("romfs");
        fs::create_dir_all(&romfs)?;
        self.code.dump(&moddir, self.game.exheader())?;
        for file in self.romfs.0 {
            file.dump(&romfs)?;
        }
        let path = path.as_ref();
        info!("");
        info!("Writing Patch Files to:         {}\\{:016X}", &path.absolutize()?.display(), self.game.id());

        match fs_extra::copy_items(&[moddir], path, &CopyOptions { overwrite: true, ..Default::default() })
            .map_err(Error::io)
        {
            Ok(_) => Ok(()),
            Err(_) => {
                error!("Couldn't write to:              {}", path.display());
                error!("Please check that config.json points to a valid output destination.");
                fail!();
            },
        }
    }
}

#[derive(Debug)]
struct Files(Vec<File<Box<[u8]>>>);

impl Files {
    pub fn add<T>(&mut self, file: File<T>)
    where
        T: IntoBytes,
    {
        self.0.push(file.into_bytes());
    }

    pub fn add_serialize<T>(&mut self, file: File<T>)
    where
        T: Serialize,
    {
        self.0.push(file.serialize());
    }
}
