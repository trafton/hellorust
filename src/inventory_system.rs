use crate::gamelog::GameLog;
use crate::{CombatStats, Consumable, InBackPack, Name, Position, ProvidesHealing, WantsToUseItem, WantsToDropItem, WantsToPickupItem, InflictsDamage, Map, SufferDamage};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackPack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackPack{ owner: pickup.collected_by }).expect("Unable to insert into backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!("You pick up {}.", names.get(pickup.item).unwrap().name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (ReadExpect<'a, Entity>,
                       ReadExpect<'a, Map>,
                       WriteExpect<'a, GameLog>,
                       Entities<'a>,
                       WriteStorage<'a, WantsToUseItem>,
                       ReadStorage<'a, Name>,
                       ReadStorage<'a, Consumable>,
                       ReadStorage<'a, ProvidesHealing>,
                       ReadStorage<'a, InflictsDamage>,
                       WriteStorage<'a, CombatStats>,
                       WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, map, mut gamelog, entities, mut wants_use, names,consumables, healing, inflicts_damage, mut combat_stats, mut suffer_damage) = data;

        let mut used_item = false;
        for(entity, use_item, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            let item_heals = healing.get(use_item.item);
            match item_heals {
                None => {}
                Some(heals) => {
                    stats.hp = i32::min(stats.max_hp, stats.hp + heals.heal_amount);
                    if entity == *player_entity {
                        gamelog.entries.push(format!("You drink the {}, healing {} hp.", names.get(use_item.item).unwrap().name, heals.heal_amount));
                    }
                }
            }

            let item_damages = inflicts_damage.get(use_item.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    let target_point = use_item.target.unwrap();
                    let idx = map.xy_idx(target_point.x, target_point.y);
                    used_item = false;

                    for mob in map.tile_content[idx].iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(use_item.item).unwrap();
                            gamelog.entries.push(format!("You use {} on {}, infliciting {} damage.", item_name.name, mob_name.name, damage.damage));
                        }

                        used_item = true;
                    }
                }
            }

            let consumable = consumables.get(use_item.item);
            match consumable {
                None => {}
                Some(_) => {
                    entities.delete(use_item.item).expect("Delete use item failed");
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToDropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackPack>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!("You drop the {}.", names.get(to_drop.item).unwrap().name));
            }
        }

        wants_drop.clear();
    }
}
