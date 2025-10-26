class ShelterManager {
    constructor(scene, inventoryManager) {
        this.scene = scene;
        this.inventory = inventoryManager;
        this.shelters = {
            'main_shelter': {
                name: 'Main Shelter',
                location: 'shelter',
                storage: {},
                maxStorage: 100,
                discovered: true
            }
        };
        this.currentShelter = 'main_shelter';
    }

    // Get current shelter's storage
    getShelterStorage() {
        return this.shelters[this.currentShelter].storage;
    }

    // Get storage capacity info
    getStorageCapacity() {
        const shelter = this.shelters[this.currentShelter];
        let used = 0;
        for (const [itemId, quantity] of Object.entries(shelter.storage)) {
            used += quantity;
        }
        return {
            used: used,
            max: shelter.maxStorage,
            remaining: shelter.maxStorage - used
        };
    }

    // Transfer item from inventory to shelter storage
    storeItem(itemId, quantity = 1) {
        const shelter = this.shelters[this.currentShelter];

        // Check if item exists in player inventory
        if (!this.inventory.hasItem(itemId, quantity)) {
            return { success: false, message: "Don't have this item!" };
        }

        // Check storage capacity
        const capacity = this.getStorageCapacity();
        if (capacity.remaining < quantity) {
            return { success: false, message: "Shelter storage full!" };
        }

        // Transfer
        this.inventory.removeItem(itemId, quantity);
        if (!shelter.storage[itemId]) {
            shelter.storage[itemId] = 0;
        }
        shelter.storage[itemId] += quantity;

        return { success: true, message: `Stored ${quantity}x ${Items[itemId].name}` };
    }

    // Take item from shelter storage to inventory
    retrieveItem(itemId, quantity = 1) {
        const shelter = this.shelters[this.currentShelter];

        // Check if item exists in storage
        if (!shelter.storage[itemId] || shelter.storage[itemId] < quantity) {
            return { success: false, message: "Item not in storage!" };
        }

        // Transfer
        shelter.storage[itemId] -= quantity;
        if (shelter.storage[itemId] <= 0) {
            delete shelter.storage[itemId];
        }
        this.inventory.addItem(itemId, quantity);

        return { success: true, message: `Retrieved ${quantity}x ${Items[itemId].name}` };
    }

    // Get list of items in shelter storage
    getStorageList() {
        const shelter = this.shelters[this.currentShelter];
        const items = [];
        for (const [itemId, quantity] of Object.entries(shelter.storage)) {
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

    // Add storage capacity (from buildings)
    addStorageCapacity(amount) {
        this.shelters[this.currentShelter].maxStorage += amount;
    }

    serialize() {
        return {
            shelters: this.shelters,
            currentShelter: this.currentShelter
        };
    }

    deserialize(data) {
        this.shelters = data.shelters || this.shelters;
        this.currentShelter = data.currentShelter || 'main_shelter';
    }
}
