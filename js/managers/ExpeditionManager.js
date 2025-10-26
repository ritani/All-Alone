class ExpeditionManager {
    constructor(scene, inventoryManager, locationManager) {
        this.scene = scene;
        this.inventory = inventoryManager;
        this.location = locationManager;
        this.expeditionState = 'none'; // none, planning, traveling, exploring, returning
        this.foundLoot = [];
    }

    // Start planning an expedition
    startExpeditionPlanning() {
        this.expeditionState = 'planning';
        this.location.startExpedition();
        this.foundLoot = [];
    }

    // Execute travel to location
    executeTravel(locationId, gameState) {
        const result = this.location.travelTo(locationId, gameState);

        if (!result.success) {
            return result;
        }

        this.expeditionState = 'traveling';

        // Consume energy and time
        gameState.energy -= result.energyCost;

        return result;
    }

    // Add loot to expedition
    addLootToExpedition(itemId, quantity) {
        return this.location.addToExpedition(itemId, quantity);
    }

    // Remove loot from expedition
    removeLootFromExpedition(itemId, quantity) {
        return this.location.removeFromExpedition(itemId, quantity);
    }

    // Get expedition capacity info
    getExpeditionCapacity() {
        return this.location.getExpeditionCapacity();
    }

    // Process found loot - returns items that can be carried
    processFoundLoot(loot) {
        this.foundLoot = loot;
        const capacity = this.getExpeditionCapacity();

        // Auto-add items until full
        const autoAdded = [];
        const cannotCarry = [];

        for (const item of loot) {
            let added = 0;
            for (let i = 0; i < item.quantity; i++) {
                if (this.addLootToExpedition(item.id, 1)) {
                    added++;
                } else {
                    break;
                }
            }

            if (added > 0) {
                autoAdded.push({
                    ...item,
                    quantity: added
                });
            }

            if (added < item.quantity) {
                cannotCarry.push({
                    ...item,
                    quantity: item.quantity - added
                });
            }
        }

        return {
            autoAdded: autoAdded,
            cannotCarry: cannotCarry,
            capacity: this.getExpeditionCapacity()
        };
    }

    // Complete expedition - transfer loot to main inventory
    completeExpedition() {
        const expeditionItems = this.location.completeExpedition();

        // Transfer to main inventory
        for (const [itemId, quantity] of Object.entries(expeditionItems)) {
            this.inventory.addItem(itemId, quantity);
        }

        this.expeditionState = 'none';
        this.foundLoot = [];

        return expeditionItems;
    }

    // Get items currently in expedition
    getExpeditionItems() {
        const items = [];
        for (const [itemId, quantity] of Object.entries(this.location.expeditionInventory)) {
            items.push({
                id: itemId,
                ...Items[itemId],
                quantity: quantity
            });
        }
        return items;
    }

    // Cancel expedition
    cancelExpedition() {
        this.location.expeditionInventory = {};
        this.location.expeditionWeight = 0;
        this.expeditionState = 'none';
        this.foundLoot = [];
    }

    isOnExpedition() {
        return this.expeditionState !== 'none';
    }

    getExpeditionState() {
        return this.expeditionState;
    }

    setExpeditionState(state) {
        this.expeditionState = state;
    }
}
