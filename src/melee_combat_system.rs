use rltk::console;
use crate::{CombatStats, Name, SufferDamage, WantsToMelee};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_to_melee, names, combat_stats, mut inflict_damage) = data;
        for(_entity, wants_melee, name, stats) in (&entities, &wants_to_melee, &names, &combat_stats).join() {
            if stats.hp > 0 {
                let target_stats_o = combat_stats.get(wants_melee.target);
                match target_stats_o {
                    None => {
                        console::log(&format!("Couldnt find target {:?}", wants_melee.target));
                    }
                    Some(target_stats) => {
                        if target_stats.hp > 0 {
                            let target_name = names.get(wants_melee.target).unwrap();
                            let damage = i32::max(0, stats.power - target_stats.defence);

                            if damage == 0 {
                                console::log(&format!("{} is unable to hurt {}", &name.name, &target_name.name));
                            } else {
                                console::log(&format!("{} hits {} for {} hp", &name.name, &target_name.name, damage));
                                SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                            }
                        }
                    }
                }

            }
        }
    }
}