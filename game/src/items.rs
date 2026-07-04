crate::int_map! {
/**
 * An enum for in-game items.
 *
 * Item indexes match the array indexes found in `World/Byaml/GetItem.byaml`.
 */
Item(u16) {
    /**
     * An `Empty` item, or no item. Shows Link holding up nothing with the text: "It's Empty"
     *
     * Note: This is actually called `None` in the game files, but we've renamed it to avoid
     * confusion with Rust's [`Option::None`].
     */
    Empty = 0x0,
    /// A generic Small Key. Adds 1 to the key count of the current scene.
    KeySmall = 0x1,
    /// A generic Boss Key. Marks the Boss Key as obtained for the current scene.
    KeyBoss = 0x2,
    /// A generic Compass. Marks the Compass as obtained for the current scene.
    Compass = 0x3,
    /// Heart Container
    HeartContainer = 0x4,
    /// Red Rupee
    RupeeR = 0x5,
    /// Green Rupee
    RupeeG = 0x6,
    /// Blue Rupee
    RupeeB = 0x7,
    /// Heart Piece
    HeartPiece = 0x8,
    /// Ice Rod
    ItemIceRod = 0x9,
    /// Sand Rod
    ItemSandRod = 0xA,
    /// Tornado Rod
    ItemTornadeRod = 0xB,
    /// Bombs
    ItemBomb = 0xC,
    /// Fire Rod
    ItemFireRod = 0xD,
    /// Hookshot
    ItemHookShot = 0xE,
    /// Boomerang
    ItemBoomerang = 0xF,
    /// Hammer
    ItemHammer = 0x10,
    /// Bow
    ItemBow = 0x11,
    /// Shield (the one bought from Item Shops)
    ItemShield = 0x12,
    /**
     * Empty Bottles
     *
     * This [`Item`] represents all 5 Bottles and is a kind of faux-Progressive Item: The first
     * one obtained unlocks Bottle #1, the second unlocks Bottle #2, etc.
     *
     * You can determine which bottles are which in your inventory by emptying all of them and then
     * catching or buying any bottle item--the bottle it goes into is Bottle #1, regardless of how
     * items are arranged in the inventory. Repeat to identify them all if you're desperate to waste
     * 10 minutes of your Thursday evening.
     */
    ItemBottle = 0x13,
    /// Red Potion
    ItemPotShopRed = 0x14,
    /// Blue Potion
    ItemPotShopBlue = 0x15,
    /// Smooth Gem
    ItemStoneBeauty = 0x16,
    /// Pendant of Power
    PendantPower = 0x17,
    /// Pendant of Wisdom
    PendantWisdom = 0x18,
    /// Pendant of Courage
    PendantCourage = 0x19,
    /// Lamp
    ItemKandelaar = 0x1A,
    /// Forgotten Sword
    ItemSwordLv1 = 0x1B,
    /// Master Sword
    ItemSwordLv2 = 0x1C,
    /// Master Sword Lv2
    ItemSwordLv3 = 0x1D,
    /// Master Sword Lv3
    ItemSwordLv4 = 0x1E,
    /// Zora's Flippers
    ItemMizukaki = 0x1F,
    /// Rented Ice Rod
    ItemRentalIceRod = 0x20,
    /// Rented Sand Rod
    ItemRentalSandRod = 0x21,
    /// Rented Tornado Rod
    ItemRentalTornadeRod = 0x22,
    /// Rented Bombs
    ItemRentalBomb = 0x23,
    /// Rented Fire Rod
    ItemRentalFireRod = 0x24,
    /// Rented Hookshot
    ItemRentalHookShot = 0x25,
    /// Rented Boomerang
    ItemRentalBoomerang = 0x26,
    /// Rented Hammer
    ItemRentalHammer = 0x27,
    /// Rented Bow
    ItemRentalBow = 0x28,
    /**
     * Rented Shield
     *
     * The game files suggest that Shields were originally going to be rented from Ravio, but
     * this idea thankfully didn't make it to the final game.
     */
    ItemRentalShield = 0x29,
    /**
     * Ravio's Bracelet (unpowered)
     *
     * This is the first Bracelet Link receives from Ravio.
     *
     * It does not let you merge and it smells funny.
     */
    RingRental = 0x2A,
    /**
     * Ravio's Bracelet
     *
     * The upgraded version of the Bracelet that lets Link merge into walls.
     *
     * It's powered by plot points.
     */
    RingHekiga = 0x2B,
    /// Bell
    ItemBell = 0x2C,
    /// Gold Rupee
    RupeeGold = 0x2D,
    /// Silver Rupee
    RupeeSilver = 0x2E,
    /// Power Glove
    PowerGlove = 0x2F,
    /// Net
    ItemInsectNet = 0x30,
    /// Titan's Mitt
    PowerfulGlove = 0x31,
    /// Maiamai
    Kinsta = 0x32,
    /// Bee Badge
    BadgeBee = 0x33,
    /// Golden Bee (functionally identical to [`Item::GoldenBeeForSale`])
    GoldenBee = 0x34,
    /// Hint Glasses
    HintGlasses = 0x35,
    /// Scoot Fruit
    EscapeFruit = 0x36,
    /// Foul Fruit
    StopFruit = 0x37,
    /// Bee
    Bee = 0x38,
    /// Fairy
    Fairy = 0x39,
    /// Monster Tail
    LiverBlue = 0x3A,
    /// Monster Guts
    LiverPurple = 0x3B,
    /// Monster Horn
    LiverYellow = 0x3C,
    /// Captain's Sword
    PackageSword = 0x3D,
    /// Charm
    ZeldaAmulet = 0x3E,
    /// Blue Mail (progressive even in vanilla)
    ClothesBlue = 0x3F,
    /// Red Mail (progressive even in vanilla)
    ClothesRed = 0x40,
    /// Hylian Shield
    HyruleShield = 0x41,
    /// Master Ore (Dark Palace)
    OreYellow = 0x42,
    /// Master Ore (Skull Woods)
    OreGreen = 0x43,
    /// Master Ore (Thieves' Hideout)
    OreBlue = 0x44,
    /// Stamina Scroll
    GanbariPowerUp = 0x45,
    /// Pouch
    Pouch = 0x46,
    /// Pegasus Boots
    DashBoots = 0x47,
    /// Master Ore (Graveyard)
    OreRed = 0x48,
    /// Letter in a Bottle
    MessageBottle = 0x49,
    /// Premium Milk
    MilkMatured = 0x4A,
    /// Purple Potion
    ItemPotShopPurple = 0x4B,
    /// Yellow Potion
    ItemPotShopYellow = 0x4C,
    /// Nice Ice Rod
    ItemIceRodLv2 = 0x4D,
    /// Nice Sand Rod
    ItemSandRodLv2 = 0x4E,
    /// Nice Tornado Rod
    ItemTornadeRodLv2 = 0x4F,
    /// Nice Bombs
    ItemBombLv2 = 0x50,
    /// Nice Fire Rod
    ItemFireRodLv2 = 0x51,
    /// Nice Hookshot
    ItemHookShotLv2 = 0x52,
    /// Nice Boomerang
    ItemBoomerangLv2 = 0x53,
    /// Nice Hammer
    ItemHammerLv2 = 0x54,
    /// Nice Bow
    ItemBowLv2 = 0x55,
    /// Great Spin
    SpecialMove = 0x56,
    /// Milk (regular)
    Milk = 0x57,
    /// Super Lamp
    ItemKandelaarLv2 = 0x58,
    /// Super Net
    ItemInsectNetLv2 = 0x59,
    /// Energy Potion (for one-time event the first time you pick it up)
    GanbariTubo = 0x5A,
    /// Purple Rupee
    RupeePurple = 0x5B,
    /// Bow of Light
    ItemBowLight = 0x5C,
    /// This is not the Triforce of Courage obtained during the game, but a dummy item that's never used.
    TriforceCourage = 0x5D,
    /// Heart from Stylish Woman (Street Merchant & Big Cucco hearts are not GetItems)
    Heart = 0x5E,
    /// Osfala's Sand Rod, functionally the same as Rental Sand Rod but in vanilla you lose it immediately
    ItemRentalSandRodFirst = 0x5F,
    /// Golden Bee (functionally identical to [`Item::GoldenBee`])
    GoldenBeeForSale = 0x60,

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // IDs below are fake, to-be-added as new GetItems                                            //
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Sage Gulley (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageGulley = 0x61,
    /// Sage Oren (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageOren = 0x62,
    /// Sage Seres (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageSeres = 0x63,
    /// Sage Osfala (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageOsfala = 0x64,
    /// Sage Impa (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageImpa = 0x65,
    /// Sage Irene (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageIrene = 0x66,
    /// Sage Rosso (FAKE ITEM, acts as stand-in until we can add new GetItems)
    SageRosso = 0x67,

    /// Small Key (Eastern Palace)
    SmallKeyEastern = 0x68,
    /// Small Key (House of Gales)
    SmallKeyGales = 0x69,
    /// Small Key (Tower of Hera)
    SmallKeyHera = 0x6A,
    /// Small Key (Dark Palace)
    SmallKeyDark = 0x6B,
    /// Small Key (Swamp Palace)
    SmallKeySwamp = 0x6C,
    /// Small Key (Skull Woods)
    SmallKeySkull = 0x6D,
    /// Small Key (Thieves' Hideout)
    SmallKeyThieves = 0x6E,
    /// Small Key (Ice Ruins)
    SmallKeyIce = 0x6F,
    /// Small Key (Desert Palace)
    SmallKeyDesert = 0x70,
    /// Small Key (Turtle Rock)
    SmallKeyTurtle = 0x71,
    /// Small Key (Lorule Castle)
    SmallKeyCastle = 0x72,
    /// Small Key (Hyrule Sanctuary)
    SmallKeyHyrule = 0x73,
    /// Small Key (Lorule Sanctuary)
    SmallKeyLorule = 0x74,

    /// Big Key (Eastern Palace)
    BigKeyEastern = 0x75,
    /// Big Key (House of Gales)
    BigKeyGales = 0x76,
    /// Big Key (Tower of Hera)
    BigKeyHera = 0x77,
    /// Big Key (Dark Palace)
    BigKeyDark = 0x78,
    /// Big Key (Swamp Palace)
    BigKeySwamp = 0x79,
    /// Big Key (Skull Woods)
    BigKeySkull = 0x7A,
    /// Big Key (Thieves' Hideout)
    BigKeyThieves = 0x7B,
    /// Big Key (Ice Ruins)
    BigKeyIce = 0x7C,
    /// Big Key (Desert Palace)
    BigKeyDesert = 0x7D,
    /// Big Key (Turtle Rock)
    BigKeyTurtle = 0x7E,

    /// Compass (Eastern Palace)
    CompassEastern = 0x7F,
    /// Compass (House of Gales)
    CompassGales = 0x80,
    /// Compass (Tower of Hera)
    CompassHera = 0x81,
    /// Compass (Dark Palace)
    CompassDark = 0x82,
    /// Compass (Swamp Palace)
    CompassSwamp = 0x83,
    /// Compass (Skull Woods)
    CompassSkull = 0x84,
    /// Compass (Thieves' Hideout)
    CompassThieves = 0x85,
    /// Compass (Ice Ruins)
    CompassIce = 0x86,
    /// Compass (Desert Palace)
    CompassDesert = 0x87,
    /// Compass (Turtle Rock)
    CompassTurtle = 0x88,
    /// Compass (Lorule Castle)
    CompassCastle = 0x89,

    /// Ice Rod Upgrade
    UpgradeIceRod = 0x8a,
    /// Sand Rod Upgrade
    UpgradeSandRod = 0x8b,
    /// Tornado Rod Upgrade,
    UpgradeTornadoRod = 0x8c,
    /// Bombs Upgrade
    UpgradeBombs = 0x8d,
    /// Fire Rod Upgrade
    UpgradeFireRod = 0x8e,
    /// Hookshot Upgrade
    UpgradeHookshot = 0x8f,
    /// Boomerang Upgrade
    UpgradeBoomerang = 0x90,
    /// Hammer Upgrade
    UpgradeHammer = 0x91,
    /// Bow Upgrade
    UpgradeBow = 0x92,
    /// Lamp Upgrade
    UpgradeLamp = 0x93,
    /// Bug Net Upgrade
    UpgradeNet = 0x94,

    /// Key Ring (Eastern Palace)
    KeyRingEastern = 0x95,
    /// Key Ring (House of Gales)
    KeyRingGales = 0x96,
    /// Key Ring (Tower of Hera)
    KeyRingHera = 0x97,
    /// Key Ring (Dark Palace)
    KeyRingDark = 0x98,
    /// Key Ring (Swamp Palace)
    KeyRingSwamp = 0x99,
    /// Key Ring (Skull Woods)
    KeyRingSkull = 0x9A,
    /// Key Ring (Thieves' Hideout)
    KeyRingThieves = 0x9B,
    /// Key Ring (Ice Ruins)
    KeyRingIce = 0x9C,
    /// Key Ring (Desert Palace)
    KeyRingDesert = 0x9D,
    /// Key Ring (Turtle Rock)
    KeyRingTurtle = 0x9E,
    /// Key Ring (Lorule Castle)
    KeyRingCastle = 0x9F,
    /// Key Ring (Hyrule Sanctuary)
    KeyRingHyrule = 0xA0,
    /// Key Ring (Lorule Sanctuary)
    KeyRingLorule = 0xA1,
}}

impl Item {
    pub const SMALL_KEY_START: u32 = Item::SmallKeyEastern as u32;
    pub const SMALL_KEY_END: u32 = Item::SmallKeyLorule as u32;
    pub const BIG_KEY_START: u32 = Item::BigKeyEastern as u32;
    pub const BIG_KEY_END: u32 = Item::BigKeyTurtle as u32;
    pub const COMPASS_START: u32 = Item::CompassEastern as u32;
    pub const COMPASS_END: u32 = Item::CompassCastle as u32;
    pub const UPGRADE_START: u32 = Item::UpgradeIceRod as u32;
    pub const UPGRADE_END: u32 = Item::UpgradeNet as u32;
    pub const KEY_RING_START: u32 = Item::KeyRingEastern as u32;
    pub const KEY_RING_END: u32 = Item::KeyRingLorule as u32;

    pub fn new_items() -> impl Iterator<Item = Self> {
        const MAX_BASE_ITEM: Item = Item::GoldenBeeForSale;
        Self::iter().filter(|&item| item > MAX_BASE_ITEM)
    }

    /// Get a name used to identify the message displayed when getting an item
    pub fn get_item_message_name(&self) -> &str {
        match &self {
            Item::SageGulley => "sage_gulley",
            Item::SageOren => "sage_oren",
            Item::SageSeres => "sage_seres",
            Item::SageOsfala => "sage_osfala",
            Item::SageImpa => "sage_impa",
            Item::SageIrene => "sage_irene",
            Item::SageRosso => "sage_rosso",
            Item::SmallKeyEastern => "small_key_eastern",
            Item::SmallKeyGales => "small_key_gales",
            Item::SmallKeyHera => "small_key_hera",
            Item::SmallKeyDark => "small_key_dark",
            Item::SmallKeySwamp => "small_key_swamp",
            Item::SmallKeySkull => "small_key_skull",
            Item::SmallKeyThieves => "small_key_thieves",
            Item::SmallKeyIce => "small_key_ice",
            Item::SmallKeyDesert => "small_key_desert",
            Item::SmallKeyTurtle => "small_key_turtle",
            Item::SmallKeyCastle => "small_key_castle",
            Item::SmallKeyHyrule => "small_key_hyrule",
            Item::SmallKeyLorule => "small_key_lorule",
            Item::BigKeyEastern => "big_key_eastern",
            Item::BigKeyGales => "big_key_gales",
            Item::BigKeyHera => "big_key_hera",
            Item::BigKeyDark => "big_key_dark",
            Item::BigKeySwamp => "big_key_swamp",
            Item::BigKeySkull => "big_key_skull",
            Item::BigKeyThieves => "big_key_thieves",
            Item::BigKeyIce => "big_key_ice",
            Item::BigKeyDesert => "big_key_desert",
            Item::BigKeyTurtle => "big_key_turtle",
            Item::CompassEastern => "compass_eastern",
            Item::CompassGales => "compass_gales",
            Item::CompassHera => "compass_hera",
            Item::CompassDark => "compass_dark",
            Item::CompassSwamp => "compass_swamp",
            Item::CompassSkull => "compass_skull",
            Item::CompassThieves => "compass_thieves",
            Item::CompassIce => "compass_ice",
            Item::CompassDesert => "compass_desert",
            Item::CompassTurtle => "compass_turtle",
            Item::CompassCastle => "compass_castle",
            Item::UpgradeIceRod => "upgrade_ice_rod",
            Item::UpgradeSandRod => "upgrade_sand_rod",
            Item::UpgradeTornadoRod => "upgrade_tornado_rod",
            Item::UpgradeBombs => "upgrade_bombs",
            Item::UpgradeFireRod => "upgrade_fire_rod",
            Item::UpgradeHookshot => "upgrade_hookshot",
            Item::UpgradeBoomerang => "upgrade_boomerang",
            Item::UpgradeHammer => "upgrade_hammer",
            Item::UpgradeBow => "upgrade_bow",
            Item::UpgradeLamp => "upgrade_lamp",
            Item::UpgradeNet => "upgrade_net",
            Item::KeyRingEastern => "key_ring_eastern",
            Item::KeyRingGales => "key_ring_gales",
            Item::KeyRingHera => "key_ring_hera",
            Item::KeyRingDark => "key_ring_dark",
            Item::KeyRingSwamp => "key_ring_swamp",
            Item::KeyRingSkull => "key_ring_skull",
            Item::KeyRingThieves => "key_ring_thieves",
            Item::KeyRingIce => "key_ring_ice",
            Item::KeyRingDesert => "key_ring_desert",
            Item::KeyRingTurtle => "key_ring_turtle",
            Item::KeyRingCastle => "key_ring_castle",
            Item::KeyRingHyrule => "key_ring_hyrule",
            Item::KeyRingLorule => "key_ring_lorule",
            _ => { panic!("No get item message name found for item {}", self.as_str()); }
        }
    }

    /// Get the message to be displayed when getting an item
    pub fn get_item_message(&self) -> &str {
        match &self {
            Item::SageGulley => "Gulley has been rescued!",
            Item::SageOren => "Oren has been rescued!",
            Item::SageSeres => "Seres has been rescued!",
            Item::SageOsfala => "Osfala has been rescued!",
            Item::SageImpa => "Impa has been rescued!",
            Item::SageIrene => "Irene has been rescued!",
            Item::SageRosso => "Rosso has been rescued!",
            Item::SmallKeyEastern => "You got a small key for Eastern Palace!",
            Item::SmallKeyGales => "You got a small key for House of Gales!",
            Item::SmallKeyHera => "You got a small key for Tower of Hera!",
            Item::SmallKeyDark => "You got a small key for Dark Palace!",
            Item::SmallKeySwamp => "You got a small key for Swamp Palace!",
            Item::SmallKeySkull => "You got a small key for Skull Woods!",
            Item::SmallKeyThieves => "You got a small key for Thieves' Hideout!",
            Item::SmallKeyIce => "You got a small key for Ice Ruins!",
            Item::SmallKeyDesert => "You got a small key for Desert Palace!",
            Item::SmallKeyTurtle => "You got a small key for Turtle Rock!",
            Item::SmallKeyCastle => "You got a small key for Lorule Castle!",
            Item::SmallKeyHyrule => "You got a small key for Hyrule Sanctuary!",
            Item::SmallKeyLorule => "You got a small key for Lorule Sanctuary!",
            Item::BigKeyEastern => "You got the big key for Eastern Palace!",
            Item::BigKeyGales => "You got the big key for House of Gales!",
            Item::BigKeyHera => "You got the big key for Tower of Hera!",
            Item::BigKeyDark => "You got the big key for Dark Palace!",
            Item::BigKeySwamp => "You got the big key for Swamp Palace!",
            Item::BigKeySkull => "You got the big key for Skull Woods!",
            Item::BigKeyThieves => "You got the big key for Thieves' Hideout!",
            Item::BigKeyIce => "You got the big key for Ice Ruins!",
            Item::BigKeyDesert => "You got the big key for Desert Palace!",
            Item::BigKeyTurtle => "You got the big key for Turtle Rock!",
            Item::CompassEastern => "You got the compass for Eastern Palace!",
            Item::CompassGales => "You got the compass for House of Gales!",
            Item::CompassHera => "You got the compass for Tower of Hera!",
            Item::CompassDark => "You got the compass for Dark Palace!",
            Item::CompassSwamp => "You got the compass for Swamp Palace!",
            Item::CompassSkull => "You got the compass for Skull Woods!",
            Item::CompassThieves => "You got the compass for Thieves' Hideout!",
            Item::CompassIce => "You got the compass for Ice Ruins!",
            Item::CompassDesert => "You got the compass for Desert Palace!",
            Item::CompassTurtle => "You got the compass for Turtle Rock!",
            Item::CompassCastle => "You got the compass for Lorule Castle!",
            Item::UpgradeIceRod => "Your Ice Rod will be upgraded!",
            Item::UpgradeSandRod => "Your Sand Rod will be upgraded!",
            Item::UpgradeTornadoRod => "Your Tornado Rod will be upgraded!",
            Item::UpgradeBombs => "Your Bombs will be upgraded!",
            Item::UpgradeFireRod => "Your Fire Rod will be upgraded!",
            Item::UpgradeHookshot => "Your Hookshot will be upgraded!",
            Item::UpgradeBoomerang => "Your Boomerang will be upgraded!",
            Item::UpgradeHammer => "Your Hammer will be upgraded!",
            Item::UpgradeBow => "Your Bow will be upgraded!",
            Item::UpgradeLamp => "Your Lamp will be upgraded!",
            Item::UpgradeNet => "Your Bug Net will be upgraded!",
            Item::KeyRingEastern => "You got the key ring for Eastern Palace!",
            Item::KeyRingGales => "You got the key ring for House of Gales!",
            Item::KeyRingHera => "You got the key ring for Tower of Hera!",
            Item::KeyRingDark => "You got the key ring for Dark Palace!",
            Item::KeyRingSwamp => "You got the key ring for Swamp Palace!",
            Item::KeyRingSkull => "You got the key ring for Skull Woods!",
            Item::KeyRingThieves => "You got the key ring for Thieves' Hideout!",
            Item::KeyRingIce => "You got the key ring for Ice Ruins!",
            Item::KeyRingDesert => "You got the key ring for Desert Palace!",
            Item::KeyRingTurtle => "You got the key ring for Turtle Rock!",
            Item::KeyRingCastle => "You got the key ring for Lorule Castle!",
            Item::KeyRingHyrule => "You got the key ring for Hyrule Sanctuary!",
            Item::KeyRingLorule => "You got the key ring for Lorule Sanctuary!",
            _ => { panic!("No get item message name found for item {}", self.as_str()); }
        }
    }
}
