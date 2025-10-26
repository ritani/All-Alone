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
    INITIAL_THIRST: 0,

    // Stat decay rates per hour
    ENERGY_DECAY: 5,
    MOOD_DECAY: 3,
    HUNGER_INCREASE: 8,
    THIRST_INCREASE: 10,

    // Time
    HOUR_DURATION: 3000, // 3 seconds = 1 hour in game

    // Combat
    COMBAT_BASE_DAMAGE: 5,
    ZOMBIE_BASE_HP: 30,
    ZOMBIE_HP_SCALING: 5, // HP increase per day
    ZOMBIE_DAMAGE_SCALING: 1, // Damage increase per day
    MOOD_ACCURACY_PENALTY: 0.02, // 2% accuracy loss per 10 mood points below 50

    // Exploration time costs (in hours)
    EXPLORE_TIME_COST: 2,
    SCAVENGE_TIME_COST: 1,
    REST_TIME_COST: 4,
    BUILD_TIME_COST: 3,
    AREA_SEARCH_TIME: 0.25, // 15 minutes per area

    // Container mechanics
    LOCKED_CONTAINER_BASE_TIME: 1, // 1 hour to force open
    TOOL_TIME_REDUCTION: 0.5, // Tools reduce time by 50%

    // Expedition limits
    EXPEDITION_MAX_WEIGHT: 20, // Max items you can carry on expedition
    BACKPACK_BASE_CAPACITY: 10, // Base carrying capacity

    // Travel
    TRAVEL_ENCOUNTER_BASE_CHANCE: 0.15, // 15% base chance per travel
    TRAVEL_ENERGY_COST_PER_HOUR: 3, // Energy cost per hour of travel
    BANDIT_ENCOUNTER_CHANCE: 0.08, // 8% chance of bandit encounter
};

// Item definitions
const Items = {
    // Food
    'canned_food': { name: 'Canned Food', type: 'food', hunger: -30, hp: 5, rarity: 'common', weight: 1 },
    'water': { name: 'Water Bottle', type: 'consumable', thirst: -40, mood: 5, rarity: 'common', weight: 1 },
    'chocolate': { name: 'Chocolate', type: 'food', hunger: -15, mood: 10, energy: 5, rarity: 'common', weight: 0.5 },
    'mre': { name: 'MRE', type: 'food', hunger: -50, energy: 15, hp: 10, rarity: 'uncommon', weight: 1 },
    'bread': { name: 'Bread', type: 'food', hunger: -20, rarity: 'common', weight: 0.5 },
    'coffee': { name: 'Coffee', type: 'consumable', energy: 20, mood: 15, thirst: -10, rarity: 'common', weight: 0.5 },
    'juice': { name: 'Juice', type: 'consumable', thirst: -30, hunger: -10, mood: 5, rarity: 'common', weight: 1 },
    'meat': { name: 'Meat', type: 'food', hunger: -40, hp: 10, rarity: 'uncommon', weight: 1.5 },
    'vegetables': { name: 'Vegetables', type: 'food', hunger: -25, hp: 5, mood: 5, rarity: 'common', weight: 1 },

    // Medicine
    'bandage': { name: 'Bandage', type: 'medicine', hp: 15, rarity: 'common', weight: 0.5 },
    'first_aid': { name: 'First Aid Kit', type: 'medicine', hp: 40, rarity: 'uncommon', weight: 1 },
    'pills': { name: 'Pills', type: 'medicine', hp: 10, mood: 5, rarity: 'common', weight: 0.5 },
    'antibiotics': { name: 'Antibiotics', type: 'medicine', hp: 25, rarity: 'uncommon', weight: 0.5 },
    'painkillers': { name: 'Painkillers', type: 'medicine', hp: 15, mood: 10, rarity: 'common', weight: 0.5 },

    // Weapons (some are dual-purpose tools)
    'pipe': { name: 'Metal Pipe', type: 'weapon', damage: 8, rarity: 'common', weight: 2 },
    'knife': { name: 'Knife', type: 'weapon', damage: 12, rarity: 'common', weight: 1 },
    'axe': { name: 'Axe', type: 'weapon', damage: 20, rarity: 'uncommon', weight: 3, utility: 'breaking', utilityPower: 3 },
    'baseball_bat': { name: 'Baseball Bat', type: 'weapon', damage: 15, rarity: 'common', weight: 2 },
    'crowbar': { name: 'Crowbar', type: 'weapon', damage: 14, rarity: 'common', weight: 2, utility: 'prying', utilityPower: 3 },
    'machete': { name: 'Machete', type: 'weapon', damage: 18, rarity: 'uncommon', weight: 2, utility: 'cutting', utilityPower: 2 },
    'hammer': { name: 'Hammer', type: 'weapon', damage: 10, rarity: 'common', weight: 1.5, utility: 'breaking', utilityPower: 2 },

    // Materials
    'wood': { name: 'Wood', type: 'material', rarity: 'common', weight: 2 },
    'metal': { name: 'Metal Scrap', type: 'material', rarity: 'common', weight: 2 },
    'cloth': { name: 'Cloth', type: 'material', rarity: 'common', weight: 0.5 },
    'rope': { name: 'Rope', type: 'material', rarity: 'uncommon', weight: 1 },
    'nails': { name: 'Nails', type: 'material', rarity: 'common', weight: 0.5 },
    'wire': { name: 'Wire', type: 'material', rarity: 'common', weight: 0.5 },
    'plastic': { name: 'Plastic', type: 'material', rarity: 'common', weight: 0.5 },
    'electronics': { name: 'Electronics', type: 'material', rarity: 'uncommon', weight: 1 },

    // Tools & Equipment
    'flashlight': { name: 'Flashlight', type: 'tool', rarity: 'common', weight: 1 },
    'backpack': { name: 'Backpack', type: 'equipment', rarity: 'uncommon', weight: 2, capacityBonus: 5 },
    'radio': { name: 'Radio', type: 'tool', rarity: 'uncommon', weight: 1.5 },
};

// Crafting recipes
const Recipes = {
    'bandage': {
        name: 'Bandage',
        materials: { 'cloth': 2 },
        result: 'bandage',
        time: 1
    },
    'rope': {
        name: 'Rope',
        materials: { 'cloth': 3 },
        result: 'rope',
        time: 1
    },
    'wooden_club': {
        name: 'Wooden Club',
        materials: { 'wood': 3, 'rope': 1 },
        result: 'pipe',
        time: 2
    },
    'reinforced_bat': {
        name: 'Reinforced Bat',
        materials: { 'wood': 2, 'nails': 3, 'metal': 1 },
        result: 'baseball_bat',
        time: 2
    },
};

// Building definitions for shelter upgrades
const Buildings = {
    'workbench': {
        name: 'Workbench',
        description: 'Improves crafting efficiency. Reduces craft time by 25%.',
        materials: { 'wood': 10, 'metal': 5, 'nails': 8 },
        buildTime: 3,
        benefit: 'craft_speed',
        value: 0.25
    },
    'water_collector': {
        name: 'Water Collector',
        description: 'Collects rainwater. Generates 1 water per day.',
        materials: { 'plastic': 5, 'metal': 3, 'rope': 2 },
        buildTime: 2,
        benefit: 'water_generation',
        value: 1
    },
    'storage_chest': {
        name: 'Storage Chest',
        description: 'Increases inventory capacity by 10.',
        materials: { 'wood': 8, 'nails': 6 },
        buildTime: 2,
        benefit: 'storage',
        value: 10
    },
    'bed': {
        name: 'Comfortable Bed',
        description: 'Rest recovers 50% more HP and Energy.',
        materials: { 'wood': 6, 'cloth': 10, 'rope': 2 },
        buildTime: 3,
        benefit: 'rest_efficiency',
        value: 0.5
    },
    'garden': {
        name: 'Small Garden',
        description: 'Grow vegetables. Generates 1 vegetables per 2 days.',
        materials: { 'wood': 5, 'plastic': 3, 'water': 2 },
        buildTime: 4,
        benefit: 'food_generation',
        value: 0.5
    },
    'barricade': {
        name: 'Barricade',
        description: 'Reduces chance of home invasion by 30%.',
        materials: { 'wood': 15, 'metal': 8, 'nails': 12 },
        buildTime: 4,
        benefit: 'defense',
        value: 0.3
    },
    'rain_barrel': {
        name: 'Rain Barrel',
        description: 'Stores collected water. +3 water storage.',
        materials: { 'plastic': 8, 'metal': 2 },
        buildTime: 1,
        benefit: 'water_storage',
        value: 3
    },
    'radio_station': {
        name: 'Radio Station',
        description: 'Communicate with survivors. Unlocks recruitment.',
        materials: { 'electronics': 5, 'metal': 4, 'wire': 6 },
        buildTime: 5,
        benefit: 'recruitment',
        value: 1
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
        loot: ['canned_food', 'water', 'chocolate', 'bandage', 'bread', 'juice', 'coffee'],
        travelTime: 1,
        distance: 0.5,
        unlocked: true,
        mapPosition: { x: 300, y: 200 }
    },
    'pharmacy': {
        name: 'Pharmacy',
        description: 'Medical supplies might be here.',
        danger: 3,
        loot: ['bandage', 'first_aid', 'pills', 'water', 'antibiotics', 'painkillers'],
        travelTime: 2,
        distance: 1,
        unlocked: false,
        mapPosition: { x: 550, y: 180 }
    },
    'hardware_store': {
        name: 'Hardware Store',
        description: 'Tools and materials for crafting.',
        danger: 2,
        loot: ['pipe', 'axe', 'wood', 'metal', 'rope', 'nails', 'wire', 'hammer', 'crowbar', 'flashlight'],
        travelTime: 1,
        distance: 0.6,
        unlocked: true,
        mapPosition: { x: 500, y: 350 }
    },
    'residential': {
        name: 'Residential Area',
        description: 'Abandoned homes. Search carefully.',
        danger: 3,
        loot: ['canned_food', 'water', 'cloth', 'knife', 'pills', 'baseball_bat', 'bread', 'plastic', 'vegetables'],
        travelTime: 2,
        distance: 1.2,
        unlocked: false,
        mapPosition: { x: 250, y: 380 }
    },
    'warehouse': {
        name: 'Warehouse',
        description: 'Large storage area. High risk, high reward.',
        danger: 4,
        loot: ['mre', 'first_aid', 'axe', 'metal', 'wood', 'rope', 'plastic', 'nails', 'wire', 'backpack'],
        travelTime: 3,
        distance: 2,
        unlocked: false,
        mapPosition: { x: 600, y: 420 }
    },
    'hospital': {
        name: 'Abandoned Hospital',
        description: 'Dangerous, but rich in medical supplies.',
        danger: 5,
        loot: ['first_aid', 'pills', 'bandage', 'mre', 'antibiotics', 'painkillers', 'electronics'],
        travelTime: 3,
        distance: 2.5,
        unlocked: false,
        mapPosition: { x: 200, y: 120 }
    }
};

// Container types for area exploration
const ContainerTypes = {
    'open': {
        name: 'Open Area',
        timeToSearch: 0.25, // 15 minutes
        locked: false,
        description: 'Items are lying in the open'
    },
    'box': {
        name: 'Box',
        timeToSearch: 0.5, // 30 minutes
        locked: false,
        description: 'A cardboard or wooden box'
    },
    'cabinet': {
        name: 'Cabinet',
        timeToSearch: 0.5, // 30 minutes
        locked: false,
        description: 'A storage cabinet'
    },
    'locker': {
        name: 'Locker',
        timeToSearch: 0.75, // 45 minutes
        locked: true,
        lockDifficulty: 1,
        description: 'A metal locker - locked!'
    },
    'safe': {
        name: 'Safe',
        timeToSearch: 2, // 2 hours
        locked: true,
        lockDifficulty: 3,
        description: 'A heavy safe - very secure!'
    },
    'crate': {
        name: 'Crate',
        timeToSearch: 1, // 1 hour
        locked: true,
        lockDifficulty: 2,
        description: 'A locked wooden crate'
    }
};

// Area definitions for each location (multiple searchable areas per location)
const LocationAreas = {
    'convenience_store': [
        { name: 'Front Counter', containerType: 'open', zombieChance: 0.3, itemChance: 0.7 },
        { name: 'Shelves', containerType: 'open', zombieChance: 0.2, itemChance: 0.8 },
        { name: 'Storage Room', containerType: 'box', zombieChance: 0.5, itemChance: 0.6 },
        { name: 'Back Office', containerType: 'cabinet', zombieChance: 0.4, itemChance: 0.5 },
        { name: 'Employee Lockers', containerType: 'locker', zombieChance: 0.2, itemChance: 0.7 }
    ],
    'pharmacy': [
        { name: 'Pharmacy Counter', containerType: 'cabinet', zombieChance: 0.4, itemChance: 0.8 },
        { name: 'Medicine Aisles', containerType: 'open', zombieChance: 0.5, itemChance: 0.7 },
        { name: 'Prescription Room', containerType: 'locker', zombieChance: 0.6, itemChance: 0.9 },
        { name: 'Storage Closet', containerType: 'box', zombieChance: 0.4, itemChance: 0.6 },
        { name: 'Manager Office', containerType: 'cabinet', zombieChance: 0.3, itemChance: 0.5 },
        { name: 'Safe', containerType: 'safe', zombieChance: 0.2, itemChance: 0.9 }
    ],
    'hardware_store': [
        { name: 'Tool Section', containerType: 'open', zombieChance: 0.3, itemChance: 0.9 },
        { name: 'Storage Area', containerType: 'crate', zombieChance: 0.5, itemChance: 0.8 },
        { name: 'Lumber Yard', containerType: 'open', zombieChance: 0.4, itemChance: 0.7 },
        { name: 'Supply Closet', containerType: 'cabinet', zombieChance: 0.3, itemChance: 0.6 },
        { name: 'Employee Break Room', containerType: 'locker', zombieChance: 0.2, itemChance: 0.5 }
    ],
    'residential': [
        { name: 'Living Room', containerType: 'open', zombieChance: 0.5, itemChance: 0.6 },
        { name: 'Kitchen', containerType: 'cabinet', zombieChance: 0.4, itemChance: 0.7 },
        { name: 'Bedroom', containerType: 'cabinet', zombieChance: 0.6, itemChance: 0.6 },
        { name: 'Bathroom', containerType: 'cabinet', zombieChance: 0.3, itemChance: 0.5 },
        { name: 'Garage', containerType: 'box', zombieChance: 0.4, itemChance: 0.8 },
        { name: 'Basement', containerType: 'box', zombieChance: 0.7, itemChance: 0.7 },
        { name: 'Master Bedroom Safe', containerType: 'safe', zombieChance: 0.2, itemChance: 0.9 }
    ],
    'warehouse': [
        { name: 'Loading Dock', containerType: 'open', zombieChance: 0.6, itemChance: 0.7 },
        { name: 'Main Storage', containerType: 'crate', zombieChance: 0.7, itemChance: 0.9 },
        { name: 'Shipping Area', containerType: 'box', zombieChance: 0.5, itemChance: 0.8 },
        { name: 'Office Area', containerType: 'cabinet', zombieChance: 0.4, itemChance: 0.6 },
        { name: 'Break Room', containerType: 'locker', zombieChance: 0.3, itemChance: 0.5 },
        { name: 'Forklift Zone', containerType: 'open', zombieChance: 0.6, itemChance: 0.7 },
        { name: 'Manager Office', containerType: 'safe', zombieChance: 0.2, itemChance: 0.9 }
    ],
    'hospital': [
        { name: 'Emergency Room', containerType: 'open', zombieChance: 0.8, itemChance: 0.8 },
        { name: 'Pharmacy', containerType: 'locker', zombieChance: 0.7, itemChance: 0.9 },
        { name: 'Surgery Room', containerType: 'cabinet', zombieChance: 0.7, itemChance: 0.9 },
        { name: 'Patient Rooms', containerType: 'cabinet', zombieChance: 0.6, itemChance: 0.6 },
        { name: 'Supply Closet', containerType: 'locker', zombieChance: 0.5, itemChance: 0.8 },
        { name: 'Morgue', containerType: 'box', zombieChance: 0.9, itemChance: 0.7 },
        { name: 'Lab', containerType: 'cabinet', zombieChance: 0.6, itemChance: 0.8 },
        { name: 'Admin Office', containerType: 'safe', zombieChance: 0.3, itemChance: 0.9 }
    ]
};
