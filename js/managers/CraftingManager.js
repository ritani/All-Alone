class CraftingManager {
    constructor(scene, inventoryManager) {
        this.scene = scene;
        this.inventory = inventoryManager;
    }

    canCraft(recipeId) {
        const recipe = Recipes[recipeId];
        if (!recipe) return false;

        for (const [materialId, quantity] of Object.entries(recipe.materials)) {
            if (!this.inventory.hasItem(materialId, quantity)) {
                return false;
            }
        }
        return true;
    }

    craft(recipeId) {
        const recipe = Recipes[recipeId];
        if (!recipe) {
            return { success: false, message: "Recipe not found!" };
        }

        if (!this.canCraft(recipeId)) {
            return { success: false, message: "Not enough materials!" };
        }

        // Remove materials
        for (const [materialId, quantity] of Object.entries(recipe.materials)) {
            this.inventory.removeItem(materialId, quantity);
        }

        // Add crafted item
        this.inventory.addItem(recipe.result, 1);

        return {
            success: true,
            message: `Crafted ${recipe.name}!`,
            item: recipe.result,
            timeCost: recipe.time
        };
    }

    getAvailableRecipes() {
        const available = [];
        for (const [recipeId, recipe] of Object.entries(Recipes)) {
            const canCraft = this.canCraft(recipeId);
            available.push({
                id: recipeId,
                ...recipe,
                canCraft: canCraft
            });
        }
        return available;
    }

    getMissingMaterials(recipeId) {
        const recipe = Recipes[recipeId];
        if (!recipe) return [];

        const missing = [];
        for (const [materialId, required] of Object.entries(recipe.materials)) {
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
}
