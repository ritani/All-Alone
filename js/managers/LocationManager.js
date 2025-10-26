class LocationManager {
    constructor(scene) {
        this.scene = scene;
        this.currentLocation = 'shelter';
        this.exploredLocations = { 'shelter': 0 };
        this.unlockedLocations = { 'shelter': true, 'convenience_store': true, 'hardware_store': true };
        this.expeditionInventory = {}; // Items carried on expedition
        this.expeditionWeight = 0;
    }

    getCurrentLocation() {
        return Locations[this.currentLocation];
    }

    getAllLocations() {
        return Object.entries(Locations).map(([id, location]) => ({
            id: id,
            ...location,
            timesExplored: this.exploredLocations[id] || 0,
            unlocked: this.unlockedLocations[id] || false
        }));
    }

    getUnlockedLocations() {
        return this.getAllLocations().filter(loc => loc.unlocked);
    }

    unlockLocation(locationId) {
        if (Locations[locationId]) {
            this.unlockedLocations[locationId] = true;
            return true;
        }
        return false;
    }

    isLocationUnlocked(locationId) {
        return this.unlockedLocations[locationId] === true;
    }

    // Calculate travel from current location to target
    calculateTravelCost(targetLocationId) {
        const targetLocation = Locations[targetLocationId];
        if (!targetLocation) return null;

        return {
            time: targetLocation.travelTime,
            energyCost: targetLocation.travelTime * GameConstants.TRAVEL_ENERGY_COST_PER_HOUR,
            encounterChance: GameConstants.TRAVEL_ENCOUNTER_BASE_CHANCE * targetLocation.travelTime
        };
    }

    // Travel to a location with chance of random encounter
    travelTo(locationId, playerStats) {
        if (!Locations[locationId]) {
            return { success: false, message: "Location not found!" };
        }

        if (!this.isLocationUnlocked(locationId)) {
            return { success: false, message: "Location not discovered yet!" };
        }

        const travelCost = this.calculateTravelCost(locationId);
        const location = Locations[locationId];

        // Check if player has enough energy
        if (playerStats.energy < travelCost.energyCost) {
            return {
                success: false,
                message: `Too tired to travel! Need ${travelCost.energyCost} energy.`
            };
        }

        // Random encounter during travel
        const encounterRoll = Math.random();
        let travelEncounter = { encountered: false, count: 0, totalHP: 0 };

        if (encounterRoll < travelCost.encounterChance) {
            const numZombies = Math.floor(Math.random() * 2) + 1; // 1-2 zombies during travel
            travelEncounter = {
                encountered: true,
                count: numZombies,
                totalHP: numZombies * GameConstants.ZOMBIE_BASE_HP,
                type: 'travel'
            };
        }

        // Chance to discover new location while traveling
        const discoveryChance = 0.15; // 15% chance
        let discoveredLocation = null;
        if (Math.random() < discoveryChance) {
            const undiscovered = Object.keys(Locations).filter(id =>
                !this.unlockedLocations[id] && id !== 'shelter'
            );
            if (undiscovered.length > 0) {
                discoveredLocation = undiscovered[Math.floor(Math.random() * undiscovered.length)];
                this.unlockLocation(discoveredLocation);
            }
        }

        // Update current location
        this.currentLocation = locationId;

        return {
            success: true,
            message: `Traveled to ${location.name}`,
            location: location,
            travelTime: travelCost.time,
            energyCost: travelCost.energyCost,
            travelEncounter: travelEncounter,
            discoveredLocation: discoveredLocation
        };
    }

    // Start expedition - prepare expedition inventory
    startExpedition() {
        this.expeditionInventory = {};
        this.expeditionWeight = 0;
    }

    // Add item to expedition inventory (returns success)
    addToExpedition(itemId, quantity = 1) {
        if (!Items[itemId]) return false;

        const item = Items[itemId];
        const totalWeight = item.weight * quantity;

        if (this.expeditionWeight + totalWeight > GameConstants.EXPEDITION_MAX_WEIGHT) {
            return false; // Too heavy
        }

        if (!this.expeditionInventory[itemId]) {
            this.expeditionInventory[itemId] = 0;
        }
        this.expeditionInventory[itemId] += quantity;
        this.expeditionWeight += totalWeight;
        return true;
    }

    // Remove from expedition inventory
    removeFromExpedition(itemId, quantity = 1) {
        if (!this.expeditionInventory[itemId] || this.expeditionInventory[itemId] < quantity) {
            return false;
        }

        const item = Items[itemId];
        this.expeditionInventory[itemId] -= quantity;
        this.expeditionWeight -= item.weight * quantity;

        if (this.expeditionInventory[itemId] <= 0) {
            delete this.expeditionInventory[itemId];
        }
        return true;
    }

    getExpeditionCapacity() {
        return {
            current: this.expeditionWeight,
            max: GameConstants.EXPEDITION_MAX_WEIGHT,
            remaining: GameConstants.EXPEDITION_MAX_WEIGHT - this.expeditionWeight
        };
    }

    // Complete expedition - returns items found
    completeExpedition() {
        const items = { ...this.expeditionInventory };
        this.expeditionInventory = {};
        this.expeditionWeight = 0;
        return items;
    }

    explore(locationId) {
        const location = Locations[locationId];
        if (!location) {
            return { success: false, message: "Location not found!" };
        }

        // Track exploration count
        if (!this.exploredLocations[locationId]) {
            this.exploredLocations[locationId] = 0;
        }
        this.exploredLocations[locationId]++;

        // Calculate loot based on danger level
        const loot = this.generateLoot(location);

        // Calculate zombie encounter
        const zombieEncounter = this.generateZombieEncounter(location);

        return {
            success: true,
            location: location.name,
            loot: loot,
            zombieEncounter: zombieEncounter,
            timeCost: GameConstants.EXPLORE_TIME_COST
        };
    }

    generateLoot(location) {
        if (!location.loot || location.loot.length === 0) {
            return [];
        }

        const foundItems = [];
        const numItems = Math.floor(Math.random() * 4) + 2; // 2-5 items

        for (let i = 0; i < numItems; i++) {
            const randomItem = location.loot[Math.floor(Math.random() * location.loot.length)];

            // Rarity check - higher danger locations have better chances
            const rarityMultiplier = 1 + (location.danger * 0.1);
            const findChance = Math.random() * rarityMultiplier;

            if (findChance > 0.3) { // 70% base chance
                const quantity = Math.random() > 0.7 ? 2 : 1; // 30% chance for 2x
                foundItems.push({
                    id: randomItem,
                    name: Items[randomItem].name,
                    quantity: quantity,
                    weight: Items[randomItem].weight * quantity
                });
            }
        }

        return foundItems;
    }

    generateZombieEncounter(location) {
        const encounterChance = location.danger * 0.2; // danger 0-5, so 0-100% chance

        if (Math.random() < encounterChance) {
            // Number of zombies based on danger
            const numZombies = Math.floor(Math.random() * location.danger) + 1;
            return {
                encountered: true,
                count: numZombies,
                totalHP: numZombies * GameConstants.ZOMBIE_BASE_HP,
                type: 'exploration'
            };
        }

        return { encountered: false, count: 0, totalHP: 0 };
    }

    scavenge(locationId) {
        const location = Locations[locationId];
        if (!location) {
            return { success: false, message: "Location not found!" };
        }

        // Scavenging is quicker but finds less
        const loot = [];
        const numItems = Math.floor(Math.random() * 2) + 1; // 1-2 items

        for (let i = 0; i < numItems; i++) {
            if (location.loot && Math.random() > 0.3) { // 70% chance to find something
                const randomItem = location.loot[Math.floor(Math.random() * location.loot.length)];
                loot.push({
                    id: randomItem,
                    name: Items[randomItem].name,
                    quantity: 1,
                    weight: Items[randomItem].weight
                });
            }
        }

        // Lower zombie encounter chance
        const zombieEncounter = Math.random() < (location.danger * 0.1) ?
            this.generateZombieEncounter({ ...location, danger: Math.max(1, location.danger - 1) }) :
            { encountered: false, count: 0, totalHP: 0 };

        return {
            success: true,
            location: location.name,
            loot: loot,
            zombieEncounter: zombieEncounter,
            timeCost: GameConstants.SCAVENGE_TIME_COST
        };
    }

    serialize() {
        return {
            currentLocation: this.currentLocation,
            exploredLocations: this.exploredLocations,
            unlockedLocations: this.unlockedLocations,
            expeditionInventory: this.expeditionInventory,
            expeditionWeight: this.expeditionWeight
        };
    }

    deserialize(data) {
        this.currentLocation = data.currentLocation || 'shelter';
        this.exploredLocations = data.exploredLocations || { 'shelter': 0 };
        this.unlockedLocations = data.unlockedLocations || { 'shelter': true, 'convenience_store': true, 'hardware_store': true };
        this.expeditionInventory = data.expeditionInventory || {};
        this.expeditionWeight = data.expeditionWeight || 0;
    }
}
