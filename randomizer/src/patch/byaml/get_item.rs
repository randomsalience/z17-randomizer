use game::Item;
use rom::{byaml, File, GetItem, item::GetItems};
use crate::patch::Patcher;
use crate::Result;

/// Get the GetItem corresponding to a new item
fn to_get_item(item: &Item) -> GetItem {
    match item {
        Item::SageGulley
        | Item::SageOren
        | Item::SageSeres
        | Item::SageOsfala
        | Item::SageImpa
        | Item::SageIrene
        | Item::SageRosso => {
            GetItem(item.as_str().into(), "".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                "".into(), "".into(), "".into(), -2, -2, -2, 0, 0)
        },
        Item::SmallKeyHyrule
        | Item::SmallKeyEastern
        | Item::SmallKeyGales
        | Item::SmallKeyHera
        | Item::SmallKeyLorule
        | Item::SmallKeyDark
        | Item::SmallKeySwamp
        | Item::SmallKeySkull
        | Item::SmallKeyThieves
        | Item::SmallKeyIce
        | Item::SmallKeyDesert
        | Item::SmallKeyTurtle
        | Item::SmallKeyCastle => {
            GetItem(item.as_str().into(), "Actor/KeySmall.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, -0.4, 0.0,
                -47.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::BigKeyEastern
        | Item::BigKeyGales
        | Item::BigKeyHera
        | Item::BigKeyDark
        | Item::BigKeySwamp
        | Item::BigKeySkull
        | Item::BigKeyThieves
        | Item::BigKeyIce
        | Item::BigKeyDesert
        | Item::BigKeyTurtle => {
            GetItem(item.as_str().into(), "Actor/KeyBoss.bch".into(), 1.0, 0.0, 0.0, -0.2, 0.0, -0.64, 0.0,
                -56.16, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 1)
        },
        Item::CompassEastern
        | Item::CompassGales
        | Item::CompassHera
        | Item::CompassDark
        | Item::CompassSwamp
        | Item::CompassSkull
        | Item::CompassThieves
        | Item::CompassIce
        | Item::CompassDesert
        | Item::CompassTurtle
        | Item::CompassCastle => {
            GetItem(item.as_str().into(), "Actor/Compass.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, -0.1, 0.0,
                -11.6, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeIceRod => {
            GetItem(item.as_str().into(), "Actor/GtEvRodIceB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeSandRod => {
            GetItem(item.as_str().into(), "Actor/GtEvRodSandB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeTornadoRod => {
            GetItem(item.as_str().into(), "Actor/GtEvTornadoB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeBombs => {
            GetItem(item.as_str().into(), "Actor/BombM.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeFireRod => {
            GetItem(item.as_str().into(), "Actor/GtEvRodFireB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeHookshot => {
            GetItem(item.as_str().into(), "Actor/GtEvHookshotB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeBoomerang => {
            GetItem(item.as_str().into(), "Actor/GtEvBoomerangB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeHammer => {
            GetItem(item.as_str().into(), "Actor/GtEvHammerB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeBow => {
            GetItem(item.as_str().into(), "Actor/GtEvBowB.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeLamp => {
            GetItem(item.as_str().into(), "Actor/GtEvKandelaar.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        Item::UpgradeNet => {
            GetItem(item.as_str().into(), "Actor/GtEvNet.bch".into(), 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, "".into(), "".into(), "".into(), -2, -2, -2, 0, 2)
        },
        _ => { panic!("No get item entry found for item {}", item.as_str()); }
    }
}

/// GetItem.byaml patches
pub fn patch(patcher: &mut Patcher) -> Result<()> {
    // Read and deserialize GetItem.byaml from RegionBoot
    let raw = patcher.boot.archive.get_mut().read("World/Byaml/GetItem.byaml").unwrap();
    let mut file_get_items: File<GetItems> = raw.try_map(|data| byaml::from_bytes(&data)).unwrap();

    // Add new items
    let get_items = &mut file_get_items.get_mut().0;
    get_items.extend(Item::new_items().map(|item| to_get_item(&item)));

    // Unused "None" Item
    // let none_item = get_items.get_mut(Item::Empty as usize).expect("Couldn't get \"None\" GetItem entry");
    // none_item.1 = String::from("Actor/KeyBoss.bch");
    // none_item.2 = 1.0;
    // none_item.set_345(Vec3 { x: 0.0, y: 0.0, z: -0.2 });
    // none_item.set_678(Vec3 { x: 0.0, y: -0.64, z: 0.0 });
    // none_item.set_rotate(Vec3 { x: -56.16, y: 0.0, z: 0.0 });
    // none_item.15 = -2;
    // none_item.16 = -2;
    // none_item.17 = -2;
    // none_item.18 = 0;
    // none_item.19 = 1;

    // Update
    let serialized = file_get_items.serialize();
    patcher.boot.archive.get_mut().update(serialized).unwrap();
    
    patcher.game.get_items().get_mut().extend(Item::new_items().map(|item| to_get_item(&item)));

    Ok(())
}
