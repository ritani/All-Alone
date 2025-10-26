class LocationManager {
    constructor(scene) {
        this.scene = scene;
        this.currentLocation = 'shelter';
        this.exploredLocations = { 'shelter': 0 };
    }

    getCurrentLocation() {
        return Locations[this.currentLocation];
    }

    getAllLocations() {
        return Object.entries(Locations).map(([id, location]) => ({
            id: id,
            ...location,
            timesExplored: this.exploredLocations[id] || 0
        }));
    }

    travelTo(locationId) {
        if (!Locations[locationId]) {
            return { success: false, message: "Location not found!" };
        }

        this.currentLocation = locationId;
        return {
            success: true,
            message: `Traveled to ${Locations[locationId].name}`,
            location: Locations[locationId]
        };
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
        const numItems = Math.floor(Math.random() * 3) + 1; // 1-3 items

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
                    quantity: quantity
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
                totalHP: numZombies * GameConstants.ZOMBIE_BASE_HP
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
        if (location.loot && Math.random() > 0.4) { // 60% chance to find something
            const randomItem = location.loot[Math.floor(Math.random() * location.loot.length)];
            loot.push({
                id: randomItem,
                name: Items[randomItem].name,
                quantity: 1
            });
        }

        // Lower zombie encounter chance
        const zombieEncounter = Math.random() < (location.danger * 0.1) ?
            this.generateZombieEncounter({ ...location, danger: location.danger - 1 }) :
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
            exploredLocations: this.exploredLocations
        };
    }

    deserialize(data) {
        this.currentLocation = data.currentLocation || 'shelter';
        this.exploredLocations = data.exploredLocations || { 'shelter': 0 };
    }
}
