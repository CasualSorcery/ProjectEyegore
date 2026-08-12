use crate::map::CartesianPos;

#[derive(PartialEq)]
pub enum Items {
    WEAPON {
        name: String,
        damage: f32,
        ammo: u32,
    },
    KEY {
        name: String,
        opens: u8,
    },
}
pub struct Inventory {
    equipped_wpn: usize,
    stashed_items: Vec<Items>,
}
impl Inventory {
    pub fn add_item(&mut self, new_item: Items) {
        match new_item {
            Items::WEAPON { name, damage, ammo } => {
                // look for an existing weapon with the same name
                for item in self.stashed_items.iter_mut() {
                    if let Items::WEAPON {
                        name: existing_name,
                        ammo: existing_ammo,
                        ..
                    } = item
                    {
                        if *existing_name == name {
                            // if is the same weapon, only add the ammo
                            *existing_ammo += ammo;
                            return;
                        }
                    } else {
                        return;
                    }
                }
                // else just add the weapon to the inventory
                self.stashed_items
                    .push(Items::WEAPON { name, damage, ammo });
            }
            Items::KEY { .. } => {
                self.stashed_items.push(new_item);
            }
        }
    }
    fn get_wpn_count(&self) -> usize {
        let mut wpn_count = 0;

        // get every weapon in the inventory
        for item in &self.stashed_items {
            if matches!(item, Items::WEAPON { .. }) {
                wpn_count += 1;
            }
        }

        wpn_count
    }
    fn get_equipped_wpn(&self) -> usize {
        self.equipped_wpn
    }
    pub fn get_current_wpn(&self) -> Option<&Items> {
        self.stashed_items
            .iter()
            .filter(|item| matches!(item, Items::WEAPON { .. }))
            .nth(self.equipped_wpn)
    }
    pub fn change_weapon(&mut self) {
        let wpn_count = self.get_wpn_count();
        //
        if wpn_count > 0 {
            self.equipped_wpn = (self.equipped_wpn + 1) % wpn_count;
        }
    }
    pub fn new() -> Self {
        // creates an empty new inventory
        Self {
            equipped_wpn: 0,
            stashed_items: Vec::new(),
        }
    }
}
pub struct Player {
    pub position: CartesianPos,
    pub direction: CartesianPos,
    pub plane: CartesianPos,
    pub move_speed: f64,
    pub rotation_speed: f64,
    pub health: f32,
    pub armor: f32,
    pub inventory: Inventory,
}
impl Player {
    // HINT: maybe can add more parameters
    pub fn new(position: CartesianPos, direction: CartesianPos, plane: CartesianPos) -> Player {
        Self {
            position,
            direction,
            plane,
            move_speed: 5.0,
            rotation_speed: 3.0,
            health: 100.0,
            armor: 100.0,
            inventory: Inventory::new(),
        }
    }
    pub fn take_damage(&mut self, amount: f32) {
        // if player has armor, do damage to armor
        if self.armor > 0.0 {
            self.armor -= amount;

            // minimum armor clamp
            if self.armor < 0.0 {
                // apply the remaining damage to the health
                self.health -= self.armor.abs();
                self.armor = 0.0;
            }
        } else {
            // else, do damage to health
            self.health -= amount;
            // minimum health clamp
            if self.health < 0.0 {
                self.health = 0.0;
            }
        }
    }
    pub fn heal(&mut self, amount: f32) {
        self.health += amount;
        // max health clamp
        if self.health > 100.0 {
            self.health = 100.0;
        }
    }
    pub fn heal_armor(&mut self, amount: f32) {
        self.armor += amount;
        // max armor clamp
        if self.armor > 100.0 {
            self.armor = 100.0;
        }
    }
}