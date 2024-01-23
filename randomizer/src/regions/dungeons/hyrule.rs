crate::region! {
    course: IndoorLight,
    name: "Hyrule Castle",
    color: Name,
    castle {
        locations: [
            "[HC] Portal": None @Portal(7[10] HyruleCastle),

            "[HC] Battlement": RupeePurple @Chest(FieldLight 18[224]),
            "[HC] West Wing": RupeeB @Chest(12[48]),
            "[HC] Zelda": ZeldaAmulet @Event(FieldLight_1B_Zelda[36]),
        ],
    },
}
