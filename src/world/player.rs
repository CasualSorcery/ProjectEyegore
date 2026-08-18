use crate::world::map::CartesianPos;

/// Types of items enumerator
#[derive(PartialEq)]
pub enum Items {
    Weapon {
        name: String,
        damage: f32,
        ammo: u32,
    },
    Key {
        name: String,
        opens: u8,
    },
}

/// The inventory's main structure
pub struct Inventory {
    equipped_wpn: usize,
    stashed_items: Vec<Items>,
}
impl Inventory {
    /// Adds an item to the inventory vector, if the item is an existing weapon in the inventory,
    /// updates the ammo quantity, else just adds the item.
    ///
    /// # Arguments
    ///
    /// * `new_item` - the new item being added to the inventory.
    pub fn add_item(&mut self, new_item: Items) {
        match new_item {
            Items::Weapon { name, damage, ammo } => {
                // look for an existing Weapon with the same name
                for item in self.stashed_items.iter_mut() {
                    if let Items::Weapon {
                        name: existing_name,
                        ammo: existing_ammo,
                        ..
                    } = item
                        && *existing_name == name
                    {
                        // if is the same Weapon, only add the ammo
                        *existing_ammo += ammo;
                        return;
                    }
                }
                // else just add the Weapon to the inventory
                self.stashed_items
                    .push(Items::Weapon { name, damage, ammo });
            }
            Items::Key { .. } => {
                self.stashed_items.push(new_item);
            }
        }
    }

    /// Gets the amount of weapons on the inventory.
    ///
    /// # Returns
    ///
    /// the amount of weapons as `usize`.
    fn get_wpn_count(&self) -> usize {
        self.stashed_items
            .iter()
            .filter(|item| matches!(item, Items::Weapon { .. }))
            .count()
    }

    /// Gets the current equipped weapon according to Inventory's `self.equipped_wpn`.
    ///
    /// # Returns
    ///
    /// the weapon id as `usize`.
    pub fn get_equipped_wpn(&self) -> usize {
        self.equipped_wpn
    }

    /// Gets the current weapon from the actual Inventory's vector
    ///
    /// # Returns
    ///
    /// either `Null` or `&Item` matching the equipped weapon.
    pub fn get_current_wpn(&self) -> Option<&Items> {
        self.stashed_items
            .iter()
            .filter(|item| matches!(item, Items::Weapon { .. }))
            .nth(self.equipped_wpn)
    }

    /// Gets the current weapon as a mutable reference from the actual Inventory's vector.
    ///
    /// # Returns
    ///
    /// either `Null` or `&mut Item` matching the equipped weapon.
    pub fn get_current_wpn_mut(&mut self) -> Option<&mut Items> {
        let equipped = self.equipped_wpn;
        self.stashed_items
            .iter_mut()
            .filter(|item| matches!(item, Items::Weapon { .. }))
            .nth(equipped)
    }

    /// Cycles to the next weapon in the inventory.
    pub fn change_weapon(&mut self) {
        let wpn_count = self.get_wpn_count();
        //
        if wpn_count > 0 {
            self.equipped_wpn = (self.equipped_wpn + 1) % wpn_count;
        }
    }

    /// struct's constructor
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
    pub health: f32,
    pub armor: f32,
    pub inventory: Inventory,
}
impl Player {
    // struct's constructor
    pub fn new(position: CartesianPos, direction: CartesianPos, plane: CartesianPos) -> Player {
        Self {
            position,
            direction,
            plane,
            move_speed: 5.0,
            health: 100.0,
            armor: 100.0,
            inventory: Inventory::new(),
        }
    }
    /// Simple damage handler
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
    /// Simple healing handler
    pub fn heal(&mut self, amount: f32) {
        self.health += amount;
        // max health clamp
        if self.health > 100.0 {
            self.health = 100.0;
        }
    }
    /// Simple armor healing handler
    pub fn heal_armor(&mut self, amount: f32) {
        self.armor += amount;
        // max armor clamp
        if self.armor > 100.0 {
            self.armor = 100.0;
        }
    }
}