use serde::Serialize;
use strum::{Display, EnumString};

#[derive(Clone, Copy, Debug, Display, EnumString, PartialEq, Eq, Hash, Serialize)]
pub enum Location {
    // Hyrule -------------------
    HyruleBellTravel,
    HyruleField,
    MaiamaiCave,
    EasternRuinsUpper,
    EasternRuinsEastLedge,
    EasternRuinsBlockedCrack,
    EasternFairyCave,
    EasternBigFairyCave,
    WitchCave,
    WitchHouse,
    RavioShop,
    ZoraDomain,
    ZoraDomainArea,
    WaterfallCave,
    WaterfallCaveShallowWater,
    MergeDungeon,
    EastRuinsBombCaveUpper,
    EastRuinsBombCaveLower,
    HouseOfGalesIsland,
    RossosHouse,
    RossoCave,
    TornadoRodDungeon,
    GraveyardLedgeHyrule,
    GraveyardLedgeLorule,
    GraveyardLedgeCave,
    BlacksmithHouse,
    BlacksmithCave,
    HyruleCastleCourtyard,
    HyruleCastleLeftRoom,
    HyruleCastleRightRoom,
    HyruleCastleInterior,
    HyruleCastleRoof,
    HyruleCastleDungeon,
    HyruleCastleDungeonBoss,
    LostWoods,
    MasterSwordArea,
    FortuneTeller,
    KakarikoJailCell,
    WellUpper,
    WellLower,
    WomanHouse,
    StylishWomanHouse,
    MilkBar,
    BeeGuyHouse,
    KakarikoItemShop,
    LakesideItemShop,
    ItemSellerCave,
    FlippersDungeon,
    SouthernRuinsBombCave,
    SouthernRuinsPillars,
    LakeDarkCave,
    IceRodCave,
    Sanctuary,
    SanctuaryChurch,
    CuccoDungeonLedge,
    CuccoDungeon,

    ZoraRiver,
    LakeHylia,
    BridgeShallowWater,
    DarkRuinsRiver,
    MiseryMireLeftPillarMerged,
    LoruleRiverCrackShallows,

    WaterfallLedge,
    CuccoHouse,
    CuccoHouseRear,

    MoldormCave,
    MoldormCaveTop,
    MoldormLedge,

    DeathMountainBase,
    DeathWestLedge,
    DeathSecondFloor,
    DeathBombCave,
    DeathWeatherVaneCaveLeft,
    DeathFairyCave,
    DonkeyCaveLower,
    DonkeyCaveUpper,
    DeathThirdFloor,
    AmidaCaveLower,
    AmidaCaveUpper,
    DeathTopLeftLedge,
    DeathMountainWestTop,
    DeathMountainEastTop,
    SpectacleRock,
    SpectacleRockCaveLeft,
    SpectacleRockCaveRight,
    HookshotDungeon,
    FireCaveTop,
    FireCaveCenter,
    FireCaveMiddle,
    FireCaveBottom,
    BoulderingLedgeLeft,
    BoulderingLedgeBottom,
    BoulderingLedgeRight,
    RossosOreMine,
    RossosOreMineFairyCave,
    FloatingIslandHyrule,

    // Lorule -------------------
    LoruleBellTravel,
    LoruleCastleArea,
    ThievesTownItemShop,
    VeteranThiefsHouse,
    FortunesChoiceLorule,
    BigBombFlowerShop,
    BigBombFlowerField,
    LoruleGraveyard,
    LoruleSanctuary,
    LoruleSanctuaryCaveLower,
    LoruleSanctuaryCaveUpper,
    KusDomainSouth,
    KusDomain,
    GreatRupeeFairyCave,
    LoruleBlacksmith,
    BootsDungeon,
    VacantHouseBottom,
    VacantHouseTop,
    ThiefGirlCave,
    SwampCave,
    BigBombCave,
    HauntedGroveLedge,

    Desert,
    DesertNorthLedge,
    DesertUseBlockedCrackRight,
    DesertUseBlockedCrackLeft,
    DesertFairyLedge,
    DesertFairyCave,
    DesertCenterLedge,
    DesertSouthWestLedge,
    DesertPalaceWeatherVane,
    DesertPalaceMidwayLedge,
    DesertZaganagaLedge,

    MiseryMire,
    SandRodDungeon,
    MiseryMireLedge,
    MiseryMireBridge,
    MiseryMireOoB,

    LoruleLakeWater,
    LoruleLakeEast,
    LoruleLakeNorthWest,
    LoruleLakeSouthWest,
    LoruleLakesideItemShop,

    DarkRuins,
    DarkRuinsBlockedCrack,
    DarkMazeEntrance,
    DarkMazeHalfway,
    DarkPalaceWeatherVane,
    DarkRuinsShallowWater,
    HinoxCaveWater,
    HinoxCaveShallowWater,
    HinoxCave,
    SkullWoodsOverworld,
    MysteriousManCave,

    RossosOreMineLorule,
    LoruleDeathWest,
    LoruleDeathEastTop,
    LoruleDeathEastLedgeUpper,
    LoruleDeathEastLedgeLower,

    IceCaveEast,
    IceCaveCenter,
    IceCaveWest,
    IceCaveNorthWest,
    IceCaveSouthWest,
    IceCaveSouth,

    FloatingIslandLorule,

    // Dungeons -----------------
    EasternPalaceFoyer,
    EasternPalace1F,
    EasternPalaceMiniboss,
    EasternPalace2F,
    EasternPalaceBoss,
    EasternPalacePostYuga,
    EasternPalaceEscape,

    HouseOfGalesFoyer,
    HouseOfGalesEast1F,
    HouseOfGalesWest1F,
    HouseOfGales2F,
    HouseOfGales3F,
    HouseOfGalesBoss,
    HouseOfGalesPostBoss,

    TowerOfHeraEntrancePegs,
    TowerOfHeraFoyer,
    TowerOfHeraBottom,
    TowerOfHeraMiddle,
    TowerOfHeraTop,
    TowerOfHeraBoss,
    TowerOfHeraPostBoss,

    DarkPalaceFoyer,
    DarkPalaceSecondRoom,
    DarkPalaceMain,
    DarkPalaceLockedDoors,
    DarkPalaceBoss,
    DarkPalaceAfterBoss,

    SwampPalaceOutside,
    SwampPalaceAntechamber,
    SwampPalaceFoyer,
    SwampPalaceMain,
    SwampPalacePostBoss,

    SkullWoodsFoyer,
    SkullWoodsMain,
    SkullWoodsB2,
    SkullWoodsElevatorHallway,
    SkullWoodsBossHallway,
    SkullWoodsEastB1NorthFoyer,
    SkullWoodsEastB1North,
    SkullWoodsEastB1SouthFoyer,
    SkullWoodsEastB1South,
    SkullWoodsEastB1SouthLedges,
    SkullWoodsOutdoor3,
    SkullWoodsBossRoom,
    SkullWoodsSeresGrove,

    ThievesHideoutB1,
    ThievesBoss,
    ThievesPostBoss,

    TurtleRockWeatherVane,
    TurtleRockFrontDoor,
    TurtleRockFoyer,
    TurtleRockMain,
    TurtleRockLeftBalcony,
    TurtleRockLeftBalconyPath,
    TurtleRockRightBalcony,
    TurtleRockRightBalconyPath,
    TurtleRockBoss,
    TurtleRockPostBoss,

    DesertPalaceFoyer,
    DesertPalace1F,
    DesertPalace2FMiniboss,
    DesertPalace2F,
    DesertPalace3F,
    DesertPalaceExit3F,
    ZaganagasArena,
    MiseryMireRewardBasket,

    IceRuinsFoyer,
    IceRuins,
    IceRuinsBoss,
    IceRuinsPostBoss,

    LoruleCastle1F,
    LoruleCastleEastLedge1F,
    LoruleCastleCenter1F,
    LoruleCastle2F3F,
    LoruleCastle4F5F,
    HildasStudy,
    ZeldasStudy,
    ThroneRoom,

    SacredRealm,
}
