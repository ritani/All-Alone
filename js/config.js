// Game configuration
const gameConfig = {
    type: Phaser.AUTO,
    parent: 'game-container',
    width: 800,
    height: 600,
    scale: {
        mode: Phaser.Scale.FIT,
        autoCenter: Phaser.Scale.CENTER_BOTH
    },
    backgroundColor: '#2d2d2d',
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 0 },
            debug: false
        }
    }
};

// Game constants
const GameConstants = {
    INITIAL_HP: 100,
    INITIAL_ENERGY: 100,
    INITIAL_MOOD: 100,
    INITIAL_HUNGER: 0,

    // Stat decay rates per hour
    ENERGY_DECAY: 5,
    MOOD_DECAY: 3,
    HUNGER_INCREASE: 8,

    // Time
    HOUR_DURATION: 3000, // 3 seconds = 1 hour in game

    // Combat
    COMBAT_BASE_DAMAGE: 5,
    ZOMBIE_BASE_HP: 30,

    // Exploration time costs (in hours)
    EXPLORE_TIME_COST: 2,
    SCAVENGE_TIME_COST: 1,
    REST_TIME_COST: 4,
};

// Item definitions
const Items = {
    // Food
    'canned_food': { name: 'Canned Food', type: 'food', hunger: -30, hp: 5, rarity: 'common' },
    'water': { name: 'Water Bottle', type: 'consumable', energy: 10, mood: 5, rarity: 'common' },
    'chocolate': { name: 'Chocolate', type: 'food', hunger: -15, mood: 10, energy: 5, rarity: 'common' },
    'mre': { name: 'MRE', type: 'food', hunger: -50, energy: 15, hp: 10, rarity: 'uncommon' },

    // Medicine
    'bandage': { name: 'Bandage', type: 'medicine', hp: 15, rarity: 'common' },
    'first_aid': { name: 'First Aid Kit', type: 'medicine', hp: 40, rarity: 'uncommon' },
    'pills': { name: 'Pills', type: 'medicine', hp: 10, mood: 5, rarity: 'common' },

    // Weapons
    'pipe': { name: 'Metal Pipe', type: 'weapon', damage: 8, rarity: 'common' },
    'knife': { name: 'Knife', type: 'weapon', damage: 12, rarity: 'common' },
    'axe': { name: 'Axe', type: 'weapon', damage: 20, rarity: 'uncommon' },
    'baseball_bat': { name: 'Baseball Bat', type: 'weapon', damage: 15, rarity: 'common' },

    // Materials
    'wood': { name: 'Wood', type: 'material', rarity: 'common' },
    'metal': { name: 'Metal Scrap', type: 'material', rarity: 'common' },
    'cloth': { name: 'Cloth', type: 'material', rarity: 'common' },
    'rope': { name: 'Rope', type: 'material', rarity: 'uncommon' },
};

// Crafting recipes
const Recipes = {
    'bandage': {
        name: 'Bandage',
        materials: { 'cloth': 2 },
        result: 'bandage',
        time: 1
    },
    'wooden_club': {
        name: 'Wooden Club',
        materials: { 'wood': 3, 'rope': 1 },
        result: 'wooden_club',
        time: 2
    },
    'barricade': {
        name: 'Barricade',
        materials: { 'wood': 5, 'metal': 2 },
        result: 'barricade',
        time: 3
    },
};

// Locations for exploration
const Locations = {
    'shelter': {
        name: 'Your Shelter',
        description: 'Your safe haven in this dangerous world.',
        danger: 0,
        canRest: true
    },
    'convenience_store': {
        name: 'Convenience Store',
        description: 'A ransacked store. Might still have supplies.',
        danger: 2,
        loot: ['canned_food', 'water', 'chocolate', 'bandage']
    },
    'pharmacy': {
        name: 'Pharmacy',
        description: 'Medical supplies might be here.',
        danger: 3,
        loot: ['bandage', 'first_aid', 'pills', 'water']
    },
    'hardware_store': {
        name: 'Hardware Store',
        description: 'Tools and materials for crafting.',
        danger: 2,
        loot: ['pipe', 'axe', 'wood', 'metal', 'rope']
    },
    'residential': {
        name: 'Residential Area',
        description: 'Abandoned homes. Search carefully.',
        danger: 3,
        loot: ['canned_food', 'water', 'cloth', 'knife', 'pills', 'baseball_bat']
    },
    'warehouse': {
        name: 'Warehouse',
        description: 'Large storage area. High risk, high reward.',
        danger: 4,
        loot: ['mre', 'first_aid', 'axe', 'metal', 'wood', 'rope']
    }
};
