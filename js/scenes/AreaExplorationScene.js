class AreaExplorationScene extends Phaser.Scene {
    constructor() {
        super({ key: 'AreaExplorationScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.targetLocation = data.targetLocation;
        this.action = data.action;
        this.areas = LocationAreas[this.targetLocation.id] || [];
        this.currentAreaIndex = 0;
        this.exploredAreas = [];
        this.foundLoot = [];
    }

    create() {
        const centerX = this.cameras.main.width / 2;

        // Get location visuals
        const visuals = LocationVisuals[this.targetLocation.id] || LocationVisuals['shelter'];

        // Themed background
        this.add.rectangle(0, 0, 800, 600, Phaser.Display.Color.HexStringToColor(visuals.bgColor).color).setOrigin(0, 0);

        // Create ExpeditionHUD
        this.expeditionHUD = new ExpeditionHUD(this, this.gameSceneRef.expeditionManager);
        this.expeditionHUD.create();

        // Location banner with icon and emojis
        const bannerBg = this.add.rectangle(centerX, 90, 700, 50, Phaser.Display.Color.HexStringToColor(visuals.accentColor).color)
            .setAlpha(0.8);

        // Location icon
        this.add.text(90, 90, visuals.icon, {
            fontSize: '40px'
        }).setOrigin(0.5);

        // Title
        this.titleText = this.add.text(centerX, 90, `EXPLORING: ${this.targetLocation.name.toUpperCase()}`, {
            fontSize: '24px',
            fontFamily: 'Arial Black',
            color: '#ffffff'
        }).setOrigin(0.5);

        // Decorative emojis
        this.add.text(710, 90, visuals.emoji, {
            fontSize: '20px'
        }).setOrigin(0.5);

        // Progress
        this.progressText = this.add.text(centerX, 135, '', {
            fontSize: '16px',
            color: '#888888'
        }).setOrigin(0.5);

        // Current area display
        this.areaNameText = this.add.text(centerX, 180, '', {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#ffffff'
        }).setOrigin(0.5);

        // Area description
        this.areaDescText = this.add.text(centerX, 160, '', {
            fontSize: '16px',
            color: '#cccccc'
        }).setOrigin(0.5);

        // Status message
        this.statusText = this.add.text(centerX, 220, '', {
            fontSize: '18px',
            color: '#ffffff',
            align: 'center',
            wordWrap: { width: 700 }
        }).setOrigin(0.5);

        // Action buttons area
        this.buttonContainer = [];

        // Start exploring first area
        if (this.areas.length > 0) {
            this.exploreNextArea();
        } else {
            this.noAreasFound();
        }
    }

    exploreNextArea() {
        if (this.currentAreaIndex >= this.areas.length) {
            // Done exploring all areas
            this.completeExploration();
            return;
        }

        const area = this.areas[this.currentAreaIndex];
        const container = ContainerTypes[area.containerType];

        this.updateProgress();
        this.areaNameText.setText(`📍 ${area.name}`);
        this.areaDescText.setText(container.description);

        // Check for zombie encounter
        const zombieRoll = Math.random();
        if (zombieRoll < area.zombieChance) {
            this.encounterZombies(area);
        } else {
            // No zombies, proceed to search
            this.searchArea(area);
        }
    }

    encounterZombies(area) {
        // Calculate zombie stats based on game progression
        const day = this.gameSceneRef.gameState.day;
        const zombieCount = Math.floor(Math.random() * 2) + 1; // 1-2 zombies per area
        const zombieHP = GameConstants.ZOMBIE_BASE_HP + (day * GameConstants.ZOMBIE_HP_SCALING);
        const zombieDamage = 8 + (day * GameConstants.ZOMBIE_DAMAGE_SCALING);

        this.statusText.setText(`💀 ${zombieCount} ZOMBIE(S) ATTACKING!\n\nHP: ${zombieHP} each | Day ${day} Difficulty`);
        this.statusText.setColor('#ff0000');

        this.clearButtons();

        // Fight button
        const fightBtn = this.createActionButton(400, 320, 'FIGHT', '#8B0000', () => {
            this.fightZombies(area, zombieCount, zombieHP, zombieDamage);
        });

        // Flee button
        const fleeBtn = this.createActionButton(400, 380, 'FLEE (Skip Area)', '#444444', () => {
            this.gameSceneRef.addMessage(`Fled from ${area.name}. Area not searched.`, '#ffaa00');
            this.currentAreaIndex++;
            this.exploreNextArea();
        });
    }

    fightZombies(area, zombieCount, zombieHP, zombieDamage) {
        const weaponDamage = this.gameSceneRef.inventoryManager.getWeaponDamage();
        const totalZombieHP = zombieCount * zombieHP;

        // Calculate combat
        const turnsToKill = Math.ceil(totalZombieHP / weaponDamage);
        const totalDamageTaken = Math.floor(turnsToKill * zombieDamage * zombieCount);

        // Apply mood accuracy penalty
        const mood = this.gameSceneRef.gameState.mood;
        let accuracyMultiplier = 1;
        if (mood < 50) {
            accuracyMultiplier = 1 - ((50 - mood) * GameConstants.MOOD_ACCURACY_PENALTY);
        }

        const actualDamageTaken = Math.floor(totalDamageTaken / accuracyMultiplier);

        // Apply damage
        this.gameSceneRef.gameState.hp = Math.max(0, this.gameSceneRef.gameState.hp - actualDamageTaken);
        this.gameSceneRef.gameState.energy = Math.max(0, this.gameSceneRef.gameState.energy - 10);

        if (this.gameSceneRef.gameState.hp <= 0) {
            this.gameSceneRef.addMessage('💀 Killed by zombies!', '#ff0000');
            this.gameSceneRef.gameOver();
            this.scene.stop();
            return;
        }

        this.gameSceneRef.addMessage(`⚔️ Defeated ${zombieCount} zombie(s) in ${area.name}. Took ${actualDamageTaken} damage.`, '#ffaa00');
        this.gameSceneRef.updateUI();

        // After combat, search the area
        this.searchArea(area);
    }

    searchArea(area) {
        const container = ContainerTypes[area.containerType];

        this.statusText.setText(`Searching ${area.name}...\n${container.name} - ${container.description}`);
        this.statusText.setColor('#88ccff');

        this.clearButtons();

        // Check if area has items
        const itemRoll = Math.random();
        if (itemRoll < area.itemChance) {
            // Items found!
            if (container.locked) {
                this.handleLockedContainer(area, container);
            } else {
                this.findItems(area, container);
            }
        } else {
            // No items
            this.statusText.setText(`Searched ${area.name}.\nNo useful items found.`);
            this.statusText.setColor('#888888');

            this.time.delayedCall(1500, () => {
                this.currentAreaIndex++;
                this.exploreNextArea();
            });
        }
    }

    handleLockedContainer(area, container) {
        this.statusText.setText(`🔒 LOCKED ${container.name.toUpperCase()}!\n\nBase time: ${container.timeToSearch}h`);
        this.statusText.setColor('#ffaa00');

        this.clearButtons();

        // Check for tools
        const tools = this.gameSceneRef.inventoryManager.getInventoryList().filter(item => item.utility);
        const hasTool = tools.length > 0;

        // Force open button
        const forceTime = container.timeToSearch;
        const forceBtn = this.createActionButton(400, 300, `FORCE OPEN (${forceTime}h)`, '#8B0000', () => {
            this.openContainer(area, container, forceTime);
        });

        // Tool buttons
        if (hasTool) {
            let yPos = 360;
            tools.forEach(tool => {
                const reducedTime = Math.max(0.25, forceTime * GameConstants.TOOL_TIME_REDUCTION);
                const toolBtn = this.createActionButton(400, yPos,
                    `Use ${tool.name} (${reducedTime.toFixed(2)}h) ⚡`, '#006600', () => {
                    this.gameSceneRef.addMessage(`Used ${tool.name} to break open container.`, '#00ff00');
                    this.openContainer(area, container, reducedTime);
                });
                yPos += 50;
            });
        }

        // Skip button
        const skipBtn = this.createActionButton(400, hasTool ? 460 : 360, 'SKIP (Leave Locked)', '#444444', () => {
            this.gameSceneRef.addMessage(`Left ${area.name} locked. Did not search.`, '#888888');
            this.currentAreaIndex++;
            this.exploreNextArea();
        });
    }

    openContainer(area, container, timeSpent) {
        this.gameSceneRef.advanceTime(timeSpent);
        this.findItems(area, container);
    }

    findItems(area, container) {
        // Generate 1-3 items from location's loot table
        const location = this.targetLocation;
        const numItems = Math.floor(Math.random() * 3) + 1;
        const foundItems = [];

        for (let i = 0; i < numItems; i++) {
            if (Math.random() > 0.3) { // 70% chance per roll
                const randomItem = location.loot[Math.floor(Math.random() * location.loot.length)];
                const quantity = Math.random() > 0.8 ? 2 : 1;
                foundItems.push({
                    id: randomItem,
                    name: Items[randomItem].name,
                    quantity: quantity,
                    weight: Items[randomItem].weight * quantity
                });
            }
        }

        if (foundItems.length === 0) {
            this.statusText.setText(`Searched ${container.name}.\nContainer was empty.`);
            this.statusText.setColor('#888888');
        } else {
            this.foundLoot.push(...foundItems);
            const itemList = foundItems.map(item => `• ${item.name} x${item.quantity}`).join('\n');
            this.statusText.setText(`✅ FOUND IN ${container.name.toUpperCase()}:\n\n${itemList}`);
            this.statusText.setColor('#00ff00');

            this.gameSceneRef.addMessage(`Found ${foundItems.length} item type(s) in ${area.name}.`, '#00ff00');
        }

        // Advance time for searching
        this.gameSceneRef.advanceTime(container.timeToSearch);

        // Continue button
        this.clearButtons();
        const continueBtn = this.createActionButton(400, 400, 'CONTINUE', '#006600', () => {
            this.currentAreaIndex++;
            this.exploreNextArea();
        });
    }

    completeExploration() {
        this.statusText.setText(`🎉 EXPLORATION COMPLETE!\n\nSearched ${this.areas.length} areas.\nFound ${this.foundLoot.length} item type(s) total.`);
        this.statusText.setColor('#ffaa00');

        this.areaNameText.setText('');
        this.areaDescText.setText('');

        this.clearButtons();

        // Return button
        const returnBtn = this.createActionButton(400, 420, 'GATHER LOOT & RETURN', '#00aa00', () => {
            this.scene.stop();
            this.scene.launch('LootSelectionScene', {
                gameScene: this.gameSceneRef,
                loot: this.foundLoot,
                targetLocation: this.targetLocation
            });
        });
    }

    noAreasFound() {
        this.statusText.setText('No areas to explore at this location.');
        this.statusText.setColor('#888888');

        this.time.delayedCall(2000, () => {
            this.scene.stop();
            this.completeExploration();
        });
    }

    updateProgress() {
        this.progressText.setText(`Area ${this.currentAreaIndex + 1} of ${this.areas.length}`);
    }

    createActionButton(x, y, text, color, callback) {
        const btn = this.add.text(x, y, text, {
            fontSize: '20px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: color,
            padding: { x: 30, y: 12 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        btn.on('pointerover', () => {
            btn.setScale(1.05);
            const brighterColor = this.brightenColor(color);
            btn.setBackgroundColor(brighterColor);
        });

        btn.on('pointerout', () => {
            btn.setScale(1);
            btn.setBackgroundColor(color);
        });

        btn.on('pointerdown', callback);

        this.buttonContainer.push(btn);
        return btn;
    }

    clearButtons() {
        this.buttonContainer.forEach(btn => btn.destroy());
        this.buttonContainer = [];
    }

    brightenColor(hexColor) {
        // Simple color brightening
        const hex = hexColor.replace('#', '');
        const num = parseInt(hex, 16);
        const r = Math.min(255, ((num >> 16) & 0xFF) + 40);
        const g = Math.min(255, ((num >> 8) & 0xFF) + 40);
        const b = Math.min(255, (num & 0xFF) + 40);
        return '#' + ((r << 16) | (g << 8) | b).toString(16).padStart(6, '0');
    }

    shutdown() {
        // Clean up HUD when scene stops
        if (this.expeditionHUD) {
            this.expeditionHUD.destroy();
        }
    }
}
