use modinfo::Settings;
use rand::Rng;
use rand::rngs::StdRng;

pub fn choose_mother_maiamai_costs(settings: &Settings, rng: &mut StdRng) -> Vec<u8> {
    (0..9).map(|_| rng.gen_range(0 .. settings.maiamai_limit + 1) as u8).collect()
}