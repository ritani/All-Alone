class InventoryManager {
    constructor(scene) {
        this.scene = scene;
        this.inventory = {};
        this.equippedWeapon = null;
        this.maxSlots = 30;
    }

    addItem(itemId, quantity = 1) {
        if (!Items[itemId]) {
            console.warn(`Item ${itemId} not found in Items database`);
            return false;
        }

        if (!this.inventory[itemId]) {
            this.inventory[itemId] = 0;
        }

        this.inventory[itemId] += quantity;
        return true;
    }

    removeItem(itemId, quantity = 1) {
        if (!this.inventory[itemId] || this.inventory[itemId] < quantity) {
            return false;
        }

        this.inventory[itemId] -= quantity;
        if (this.inventory[itemId] <= 0) {
            delete this.inventory[itemId];
        }
        return true;
    }

    hasItem(itemId, quantity = 1) {
        return this.inventory[itemId] && this.inventory[itemId] >= quantity;
    }

    getItemCount(itemId) {
        return this.inventory[itemId] || 0;
    }

    useItem(itemId) {
        if (!this.hasItem(itemId)) {
            return { success: false, message: "You don't have this item!" };
        }

        const item = Items[itemId];
        const effects = {};

        // Apply item effects
        if (item.hp) effects.hp = item.hp;
        if (item.energy) effects.energy = item.energy;
        if (item.mood) effects.mood = item.mood;
        if (item.hunger) effects.hunger = item.hunger;

        // Remove item from inventory
        this.removeItem(itemId, 1);

        return { success: true, effects: effects, itemName: item.name };
    }

    equipWeapon(itemId) {
        if (!this.hasItem(itemId)) {
            return false;
        }

        const item = Items[itemId];
        if (item.type !== 'weapon') {
            return false;
        }

        this.equippedWeapon = itemId;
        return true;
    }

    getEquippedWeapon() {
        if (this.equippedWeapon && this.hasItem(this.equippedWeapon)) {
            return Items[this.equippedWeapon];
        }
        return null;
    }

    getWeaponDamage() {
        const weapon = this.getEquippedWeapon();
        return weapon ? weapon.damage : GameConstants.COMBAT_BASE_DAMAGE;
    }

    getInventoryList() {
        const items = [];
        for (const [itemId, quantity] of Object.entries(this.inventory)) {
            if (quantity > 0) {
                items.push({
                    id: itemId,
                    ...Items[itemId],
                    quantity: quantity
                });
            }
        }
        return items;
    }

    getItemsByType(type) {
        return this.getInventoryList().filter(item => item.type === type);
    }

    getTotalItems() {
        let total = 0;
        for (const quantity of Object.values(this.inventory)) {
            total += quantity;
        }
        return total;
    }

    clear() {
        this.inventory = {};
        this.equippedWeapon = null;
    }

    serialize() {
        return {
            inventory: this.inventory,
            equippedWeapon: this.equippedWeapon
        };
    }

    deserialize(data) {
        this.inventory = data.inventory || {};
        this.equippedWeapon = data.equippedWeapon || null;
    }
}
