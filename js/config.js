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

    // Expedition limits
    EXPEDITION_MAX_WEIGHT: 20, // Max items you can carry on expedition
    BACKPACK_BASE_CAPACITY: 10, // Base carrying capacity

    // Travel
    TRAVEL_ENCOUNTER_BASE_CHANCE: 0.15, // 15% base chance per travel
    TRAVEL_ENERGY_COST_PER_HOUR: 3, // Energy cost per hour of travel
};

// Item definitions
const Items = {
    // Food
    'canned_food': { name: 'Canned Food', type: 'food', hunger: -30, hp: 5, rarity: 'common', weight: 1 },
    'water': { name: 'Water Bottle', type: 'consumable', energy: 10, mood: 5, rarity: 'common', weight: 1 },
    'chocolate': { name: 'Chocolate', type: 'food', hunger: -15, mood: 10, energy: 5, rarity: 'common', weight: 0.5 },
    'mre': { name: 'MRE', type: 'food', hunger: -50, energy: 15, hp: 10, rarity: 'uncommon', weight: 1 },

    // Medicine
    'bandage': { name: 'Bandage', type: 'medicine', hp: 15, rarity: 'common', weight: 0.5 },
    'first_aid': { name: 'First Aid Kit', type: 'medicine', hp: 40, rarity: 'uncommon', weight: 1 },
    'pills': { name: 'Pills', type: 'medicine', hp: 10, mood: 5, rarity: 'common', weight: 0.5 },

    // Weapons
    'pipe': { name: 'Metal Pipe', type: 'weapon', damage: 8, rarity: 'common', weight: 2 },
    'knife': { name: 'Knife', type: 'weapon', damage: 12, rarity: 'common', weight: 1 },
    'axe': { name: 'Axe', type: 'weapon', damage: 20, rarity: 'uncommon', weight: 3 },
    'baseball_bat': { name: 'Baseball Bat', type: 'weapon', damage: 15, rarity: 'common', weight: 2 },

    // Materials
    'wood': { name: 'Wood', type: 'material', rarity: 'common', weight: 2 },
    'metal': { name: 'Metal Scrap', type: 'material', rarity: 'common', weight: 2 },
    'cloth': { name: 'Cloth', type: 'material', rarity: 'common', weight: 0.5 },
    'rope': { name: 'Rope', type: 'material', rarity: 'uncommon', weight: 1 },
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
        canRest: true,
        travelTime: 0,
        distance: 0,
        unlocked: true,
        mapPosition: { x: 400, y: 300 }
    },
    'convenience_store': {
        name: 'Convenience Store',
        description: 'A ransacked store. Might still have supplies.',
        danger: 2,
        loot: ['canned_food', 'water', 'chocolate', 'bandage'],
        travelTime: 1, // hours
        distance: 0.5, // km
        unlocked: true, // Always available at start
        mapPosition: { x: 300, y: 200 }
    },
    'pharmacy': {
        name: 'Pharmacy',
        description: 'Medical supplies might be here.',
        danger: 3,
        loot: ['bandage', 'first_aid', 'pills', 'water'],
        travelTime: 2,
        distance: 1,
        unlocked: false, // Need to discover
        mapPosition: { x: 550, y: 180 }
    },
    'hardware_store': {
        name: 'Hardware Store',
        description: 'Tools and materials for crafting.',
        danger: 2,
        loot: ['pipe', 'axe', 'wood', 'metal', 'rope'],
        travelTime: 1,
        distance: 0.6,
        unlocked: true,
        mapPosition: { x: 500, y: 350 }
    },
    'residential': {
        name: 'Residential Area',
        description: 'Abandoned homes. Search carefully.',
        danger: 3,
        loot: ['canned_food', 'water', 'cloth', 'knife', 'pills', 'baseball_bat'],
        travelTime: 2,
        distance: 1.2,
        unlocked: false,
        mapPosition: { x: 250, y: 380 }
    },
    'warehouse': {
        name: 'Warehouse',
        description: 'Large storage area. High risk, high reward.',
        danger: 4,
        loot: ['mre', 'first_aid', 'axe', 'metal', 'wood', 'rope'],
        travelTime: 3,
        distance: 2,
        unlocked: false,
        mapPosition: { x: 600, y: 420 }
    },
    'hospital': {
        name: 'Abandoned Hospital',
        description: 'Dangerous, but rich in medical supplies.',
        danger: 5,
        loot: ['first_aid', 'pills', 'bandage', 'mre'],
        travelTime: 3,
        distance: 2.5,
        unlocked: false,
        mapPosition: { x: 200, y: 120 }
    }
};
