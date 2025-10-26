class BuildingManager {
    constructor(scene, inventoryManager) {
        this.scene = scene;
        this.inventory = inventoryManager;
        this.builtBuildings = {};
        this.buildingBonuses = {
            craft_speed: 0,
            rest_efficiency: 0,
            defense: 0,
            storage: 0
        };
    }

    canBuild(buildingId) {
        const building = Buildings[buildingId];
        if (!building) return false;

        // Check if already built
        if (this.builtBuildings[buildingId]) {
            return false;
        }

        // Check materials
        for (const [materialId, quantity] of Object.entries(building.materials)) {
            if (!this.inventory.hasItem(materialId, quantity)) {
                return false;
            }
        }

        return true;
    }

    build(buildingId) {
        const building = Buildings[buildingId];
        if (!building) {
            return { success: false, message: "Building not found!" };
        }

        if (this.builtBuildings[buildingId]) {
            return { success: false, message: "Building already exists!" };
        }

        if (!this.canBuild(buildingId)) {
            return { success: false, message: "Not enough materials!" };
        }

        // Remove materials
        for (const [materialId, quantity] of Object.entries(building.materials)) {
            this.inventory.removeItem(materialId, quantity);
        }

        // Build the building
        this.builtBuildings[buildingId] = {
            id: buildingId,
            ...building,
            builtDay: this.scene.gameState.day
        };

        // Apply bonuses
        this.applyBuildingBonus(building);

        return {
            success: true,
            message: `Built ${building.name}!`,
            building: building,
            timeCost: building.buildTime
        };
    }

    applyBuildingBonus(building) {
        if (building.benefit && building.value) {
            if (this.buildingBonuses.hasOwnProperty(building.benefit)) {
                this.buildingBonuses[building.benefit] += building.value;
            }
        }
    }

    getBuiltBuildings() {
        return Object.values(this.builtBuildings);
    }

    hasBuilding(buildingId) {
        return this.builtBuildings.hasOwnProperty(buildingId);
    }

    getAvailableBuildings() {
        const available = [];
        for (const [buildingId, building] of Object.entries(Buildings)) {
            const canBuild = this.canBuild(buildingId);
            const alreadyBuilt = this.builtBuildings[buildingId] !== undefined;
            available.push({
                id: buildingId,
                ...building,
                canBuild: canBuild,
                built: alreadyBuilt
            });
        }
        return available;
    }

    getMissingMaterials(buildingId) {
        const building = Buildings[buildingId];
        if (!building) return [];

        const missing = [];
        for (const [materialId, required] of Object.entries(building.materials)) {
            const current = this.inventory.getItemCount(materialId);
            if (current < required) {
                missing.push({
                    id: materialId,
                    name: Items[materialId].name,
                    required: required,
                    current: current,
                    needed: required - current
                });
            }
        }
        return missing;
    }

    // Get bonus multiplier for crafting
    getCraftSpeedMultiplier() {
        return 1 - this.buildingBonuses.craft_speed;
    }

    // Get bonus multiplier for resting
    getRestEfficiencyMultiplier() {
        return 1 + this.buildingBonuses.rest_efficiency;
    }

    // Get defense bonus
    getDefenseBonus() {
        return this.buildingBonuses.defense;
    }

    // Get storage bonus
    getStorageBonus() {
        return this.buildingBonuses.storage;
    }

    // Check if can recruit survivors
    canRecruitSurvivors() {
        return this.hasBuilding('radio_station');
    }

    // Daily resource generation
    generateDailyResources() {
        const generated = {};

        // Water collector
        if (this.hasBuilding('water_collector')) {
            generated.water = (generated.water || 0) + 1;
        }

        // Garden
        if (this.hasBuilding('garden')) {
            // Generates every 2 days
            if (this.scene.gameState.day % 2 === 0) {
                generated.vegetables = (generated.vegetables || 0) + 1;
            }
        }

        // Add to inventory
        for (const [itemId, quantity] of Object.entries(generated)) {
            this.inventory.addItem(itemId, quantity);
        }

        return generated;
    }

    serialize() {
        return {
            builtBuildings: this.builtBuildings,
            buildingBonuses: this.buildingBonuses
        };
    }

    deserialize(data) {
        this.builtBuildings = data.builtBuildings || {};
        this.buildingBonuses = data.buildingBonuses || {
            craft_speed: 0,
            rest_efficiency: 0,
            defense: 0,
            storage: 0
        };
    }
}
