class GameScene extends Phaser.Scene {
    constructor() {
        super({ key: 'GameScene' });
    }

    init(data) {
        this.isNewGame = data.newGame !== false;
    }

    create() {
        // Initialize managers
        this.inventoryManager = new InventoryManager(this);
        this.craftingManager = new CraftingManager(this, this.inventoryManager);
        this.locationManager = new LocationManager(this);
        this.buildingManager = new BuildingManager(this, this.inventoryManager);
        this.expeditionManager = null; // Initialized when starting expedition

        // Initialize game state
        this.gameState = {
            day: 1,
            hour: 8,
            hp: GameConstants.INITIAL_HP,
            energy: GameConstants.INITIAL_ENERGY,
            mood: GameConstants.INITIAL_MOOD,
            hunger: GameConstants.INITIAL_HUNGER,
            thirst: GameConstants.INITIAL_THIRST,
            isAlive: true
        };

        // Load save or start new game
        if (!this.isNewGame) {
            this.loadGame();
        } else {
            this.startNewGame();
        }

        // Create UI
        this.createUI();

        // Time system
        this.lastHourUpdate = 0;
        this.isPaused = false;

        // Message log
        this.messageLog = [];

        // Auto-save every 30 seconds
        this.time.addEvent({
            delay: 30000,
            callback: () => this.saveGame(),
            loop: true
        });

        this.addMessage('Welcome to Buried City. Survive as long as you can.', '#ffaa00');
        this.updateUI();
    }

    update(time, delta) {
        if (this.isPaused || !this.gameState.isAlive) return;

        // Time progression
        this.lastHourUpdate += delta;
        if (this.lastHourUpdate >= GameConstants.HOUR_DURATION) {
            this.lastHourUpdate = 0;
            this.advanceTime(1);
        }
    }

    startNewGame() {
        // Give player starting items
        this.inventoryManager.addItem('canned_food', 3);
        this.inventoryManager.addItem('water', 2);
        this.inventoryManager.addItem('bandage', 2);
        this.inventoryManager.addItem('knife', 1);
        this.inventoryManager.equipWeapon('knife');
    }

    createUI() {
        const width = this.cameras.main.width;
        const height = this.cameras.main.height;

        // Background
        this.add.rectangle(0, 0, width, height, 0x1a1a1a).setOrigin(0, 0);

        // Top bar - Stats
        const topBar = this.add.rectangle(0, 0, width, 80, 0x2a2a2a).setOrigin(0, 0);
        this.add.rectangle(0, 80, width, 2, 0x8B0000).setOrigin(0, 0);

        // Stats display
        this.statsText = this.add.text(20, 15, '', {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#ffffff'
        });

        // Time display
        this.timeText = this.add.text(width - 20, 15, '', {
            fontSize: '18px',
            fontFamily: 'Arial Bold',
            color: '#ffaa00'
        }).setOrigin(1, 0);

        // Location display
        this.locationText = this.add.text(width - 20, 45, '', {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#88ccff'
        }).setOrigin(1, 0);

        // Main content area
        const contentY = 90;
        const contentHeight = height - 170;

        // Message log area
        this.messageLogBg = this.add.rectangle(10, contentY, width - 20, 120, 0x2a2a2a)
            .setOrigin(0, 0)
            .setStrokeStyle(2, 0x4a4a4a);

        this.messageLogText = this.add.text(20, contentY + 10, '', {
            fontSize: '14px',
            fontFamily: 'Courier',
            color: '#cccccc',
            wordWrap: { width: width - 40 }
        });

        // Action buttons area
        const buttonsY = contentY + 130;
        this.createActionButtons(buttonsY);

        // Bottom bar - Inventory & Crafting
        this.createBottomTabs();
    }

    createActionButtons(startY) {
        const width = this.cameras.main.width;
        const buttonWidth = (width - 50) / 3;
        const buttonHeight = 50;

        const buttonStyle = {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: '#3a3a3a',
            padding: { x: 15, y: 10 }
        };

        // Explore button
        this.exploreBtn = this.createButton(20, startY, 'Explore', buttonWidth, buttonHeight, buttonStyle, () => {
            this.showMapView('explore');
        });

        // Rest button
        this.restBtn = this.createButton(30 + buttonWidth, startY, 'Rest', buttonWidth, buttonHeight, buttonStyle, () => {
            this.rest();
        });

        // Scavenge button
        this.scavengeBtn = this.createButton(40 + buttonWidth * 2, startY, 'Scavenge', buttonWidth, buttonHeight, buttonStyle, () => {
            this.showMapView('scavenge');
        });

        // Second row
        const secondRowY = startY + 60;

        // Craft button
        this.craftBtn = this.createButton(20, secondRowY, 'Craft', buttonWidth, buttonHeight, buttonStyle, () => {
            this.showCraftingMenu();
        });

        // Use Item button
        this.useBtn = this.createButton(30 + buttonWidth, secondRowY, 'Use Item', buttonWidth, buttonHeight, buttonStyle, () => {
            this.showInventoryMenu();
        });

        // Save & Menu button
        this.menuBtn = this.createButton(40 + buttonWidth * 2, secondRowY, 'Menu', buttonWidth, buttonHeight, buttonStyle, () => {
            this.showGameMenu();
        });
    }

    createButton(x, y, text, width, height, style, callback) {
        const button = this.add.text(x, y, text, {
            ...style,
            fixedWidth: width,
            fixedHeight: height,
            align: 'center'
        }).setOrigin(0, 0).setInteractive({ useHandCursor: true });

        button.on('pointerover', () => {
            button.setBackgroundColor('#4a4a4a');
        });

        button.on('pointerout', () => {
            button.setBackgroundColor(style.backgroundColor);
        });

        button.on('pointerdown', callback);

        return button;
    }

    createBottomTabs() {
        const width = this.cameras.main.width;
        const height = this.cameras.main.height;
        const bottomY = height - 80;

        // Bottom bar
        this.add.rectangle(0, bottomY, width, 80, 0x2a2a2a).setOrigin(0, 0);
        this.add.rectangle(0, bottomY, width, 2, 0x8B0000).setOrigin(0, 0);

        // Weapon display
        this.weaponText = this.add.text(20, bottomY + 15, '', {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#ffcc00'
        });

        // Quick inventory display
        this.inventoryQuickText = this.add.text(20, bottomY + 45, '', {
            fontSize: '14px',
            fontFamily: 'Arial',
            color: '#aaaaaa'
        });
    }

    updateUI() {
        // Update stats
        const hpColor = this.gameState.hp > 50 ? '#00ff00' : this.gameState.hp > 25 ? '#ffaa00' : '#ff0000';
        const energyColor = this.gameState.energy > 50 ? '#00aaff' : this.gameState.energy > 25 ? '#ffaa00' : '#ff0000';
        const moodColor = this.gameState.mood > 50 ? '#ff00ff' : this.gameState.mood > 25 ? '#ffaa00' : '#ff0000';
        const hungerColor = this.gameState.hunger < 50 ? '#00ff00' : this.gameState.hunger < 75 ? '#ffaa00' : '#ff0000';

        this.statsText.setText([
            `HP: ${Math.floor(this.gameState.hp)}/${GameConstants.INITIAL_HP}`,
            `Energy: ${Math.floor(this.gameState.energy)}/${GameConstants.INITIAL_ENERGY}  ` +
            `Mood: ${Math.floor(this.gameState.mood)}/${GameConstants.INITIAL_MOOD}  ` +
            `Hunger: ${Math.floor(this.gameState.hunger)}/100`
        ]);

        // Update time
        const timeOfDay = this.gameState.hour < 12 ? 'Morning' :
                         this.gameState.hour < 18 ? 'Afternoon' :
                         this.gameState.hour < 22 ? 'Evening' : 'Night';
        this.timeText.setText(`Day ${this.gameState.day} - ${this.gameState.hour}:00 (${timeOfDay})`);

        // Update location
        const currentLoc = this.locationManager.getCurrentLocation();
        this.locationText.setText(`📍 ${currentLoc.name}`);

        // Update weapon
        const weapon = this.inventoryManager.getEquippedWeapon();
        this.weaponText.setText(weapon ?
            `⚔️ Equipped: ${weapon.name} (${weapon.damage} DMG)` :
            `⚔️ No weapon equipped (${GameConstants.COMBAT_BASE_DAMAGE} DMG)`
        );

        // Update quick inventory
        const items = this.inventoryManager.getInventoryList();
        const itemCount = this.inventoryManager.getTotalItems();
        this.inventoryQuickText.setText(`🎒 Inventory: ${itemCount}/${this.inventoryManager.maxSlots} items`);

        // Update message log
        const recentMessages = this.messageLog.slice(-4);
        this.messageLogText.setText(recentMessages.join('\n'));
    }

    addMessage(text, color = '#cccccc') {
        const timestamp = `[${this.gameState.hour}:00]`;
        const message = `${timestamp} ${text}`;
        this.messageLog.push(message);
        this.updateUI();
    }

    advanceTime(hours) {
        this.gameState.hour += hours;
        if (this.gameState.hour >= 24) {
            this.gameState.hour -= 24;
            this.gameState.day += 1;
            this.addMessage(`=== Day ${this.gameState.day} ===`, '#ffaa00');
        }

        // Stat decay
        this.gameState.energy = Math.max(0, this.gameState.energy - GameConstants.ENERGY_DECAY * hours);
        this.gameState.mood = Math.max(0, this.gameState.mood - GameConstants.MOOD_DECAY * hours);
        this.gameState.hunger = Math.min(100, this.gameState.hunger + GameConstants.HUNGER_INCREASE * hours);

        // Hunger damage
        if (this.gameState.hunger >= 80) {
            this.gameState.hp = Math.max(0, this.gameState.hp - 2 * hours);
            this.addMessage('You are starving! HP decreased.', '#ff0000');
        }

        // Low energy penalty
        if (this.gameState.energy <= 10) {
            this.addMessage('You are exhausted!', '#ff0000');
        }

        // Check death
        if (this.gameState.hp <= 0) {
            this.gameOver();
            return;
        }

        this.updateUI();
    }

    rest() {
        if (this.locationManager.currentLocation !== 'shelter') {
            this.addMessage('You can only rest at your shelter!', '#ff0000');
            return;
        }

        const hoursToRest = GameConstants.REST_TIME_COST;
        this.gameState.energy = Math.min(GameConstants.INITIAL_ENERGY, this.gameState.energy + 40);
        this.gameState.hp = Math.min(GameConstants.INITIAL_HP, this.gameState.hp + 20);
        this.gameState.mood = Math.min(GameConstants.INITIAL_MOOD, this.gameState.mood + 15);

        this.addMessage(`Rested for ${hoursToRest} hours. Feeling better!`, '#00ff00');
        this.advanceTime(hoursToRest);
    }

    showMapView(action) {
        if (this.gameState.energy < 10) {
            this.addMessage('Too tired to travel! Rest first.', '#ff0000');
            return;
        }

        // Launch the map scene
        this.scene.launch('MapScene', {
            gameScene: this,
            action: action
        });
    }

    // Note: Old exploration methods replaced by expedition system
    // These are kept for reference only and are no longer called

    handleCombat(encounter, callback) {
        this.addMessage(`💀 ${encounter.count} zombie(s) appeared!`, '#ff0000');

        const weaponDamage = this.inventoryManager.getWeaponDamage();
        const playerHP = this.gameState.hp;

        // Calculate combat outcome
        const turnsToKill = Math.ceil(encounter.totalHP / weaponDamage);
        const damagePerTurn = encounter.count * 8; // Each zombie does ~8 damage
        const totalDamageTaken = turnsToKill * damagePerTurn;

        // Apply damage
        this.gameState.hp = Math.max(0, this.gameState.hp - totalDamageTaken);
        this.gameState.energy = Math.max(0, this.gameState.energy - 15);

        if (this.gameState.hp > 0) {
            this.addMessage(`⚔️ Defeated zombies! Took ${Math.floor(totalDamageTaken)} damage.`, '#ffaa00');
            this.updateUI();
            if (callback) callback();
        } else {
            this.addMessage('💀 You were killed by zombies!', '#ff0000');
            this.gameOver();
        }
    }

    // collectLoot method removed - now handled by expedition system

    showInventoryMenu() {
        this.createModal('INVENTORY', (container) => {
            const consumables = this.inventoryManager.getInventoryList()
                .filter(item => ['food', 'medicine', 'consumable'].includes(item.type));

            if (consumables.length === 0) {
                const noItems = this.add.text(400, 200, 'No usable items in inventory.', {
                    fontSize: '18px',
                    color: '#888888'
                }).setOrigin(0.5);
                container.push(noItems);
                return;
            }

            let yPos = 150;
            consumables.forEach(item => {
                const effects = [];
                if (item.hp) effects.push(`HP +${item.hp}`);
                if (item.energy) effects.push(`Energy +${item.energy}`);
                if (item.mood) effects.push(`Mood +${item.mood}`);
                if (item.hunger) effects.push(`Hunger ${item.hunger}`);

                const btnText = `${item.name} (x${item.quantity})\n${effects.join(', ')}`;

                const btn = this.add.text(400, yPos, btnText, {
                    fontSize: '16px',
                    fontFamily: 'Arial',
                    color: '#ffffff',
                    backgroundColor: '#3a3a3a',
                    padding: { x: 20, y: 10 },
                    align: 'center',
                    fixedWidth: 500
                }).setOrigin(0.5, 0).setInteractive({ useHandCursor: true });

                btn.on('pointerover', () => btn.setBackgroundColor('#4a4a4a'));
                btn.on('pointerout', () => btn.setBackgroundColor('#3a3a3a'));
                btn.on('pointerdown', () => {
                    const result = this.inventoryManager.useItem(item.id);
                    if (result.success) {
                        // Apply effects
                        if (result.effects.hp) this.gameState.hp = Math.min(GameConstants.INITIAL_HP, this.gameState.hp + result.effects.hp);
                        if (result.effects.energy) this.gameState.energy = Math.min(GameConstants.INITIAL_ENERGY, this.gameState.energy + result.effects.energy);
                        if (result.effects.mood) this.gameState.mood = Math.min(GameConstants.INITIAL_MOOD, this.gameState.mood + result.effects.mood);
                        if (result.effects.hunger) this.gameState.hunger = Math.max(0, this.gameState.hunger + result.effects.hunger);

                        this.addMessage(`Used ${result.itemName}`, '#00ff00');
                        this.advanceTime(0.5);
                        container.forEach(obj => obj.destroy());
                        this.updateUI();
                    }
                });

                container.push(btn);
                yPos += 70;
            });
        });
    }

    showCraftingMenu() {
        this.createModal('CRAFTING', (container) => {
            const recipes = this.craftingManager.getAvailableRecipes();

            if (recipes.length === 0) {
                const noRecipes = this.add.text(400, 200, 'No crafting recipes available.', {
                    fontSize: '18px',
                    color: '#888888'
                }).setOrigin(0.5);
                container.push(noRecipes);
                return;
            }

            let yPos = 150;
            recipes.forEach(recipe => {
                const materials = Object.entries(recipe.materials)
                    .map(([id, qty]) => `${Items[id].name} x${qty}`)
                    .join(', ');

                const btnText = `${recipe.name}\nNeeds: ${materials}`;
                const btnColor = recipe.canCraft ? '#3a3a3a' : '#2a2a2a';

                const btn = this.add.text(400, yPos, btnText, {
                    fontSize: '16px',
                    fontFamily: 'Arial',
                    color: recipe.canCraft ? '#ffffff' : '#666666',
                    backgroundColor: btnColor,
                    padding: { x: 20, y: 10 },
                    align: 'center',
                    fixedWidth: 500
                }).setOrigin(0.5, 0);

                if (recipe.canCraft) {
                    btn.setInteractive({ useHandCursor: true });
                    btn.on('pointerover', () => btn.setBackgroundColor('#4a4a4a'));
                    btn.on('pointerout', () => btn.setBackgroundColor(btnColor));
                    btn.on('pointerdown', () => {
                        const result = this.craftingManager.craft(recipe.id);
                        if (result.success) {
                            this.addMessage(result.message, '#00ff00');
                            this.advanceTime(result.timeCost);
                            container.forEach(obj => obj.destroy());
                            this.updateUI();
                        }
                    });
                }

                container.push(btn);
                yPos += 70;
            });
        });
    }

    showGameMenu() {
        this.createModal('MENU', (container) => {
            const buttons = [
                { text: 'Save Game', callback: () => {
                    this.saveGame();
                    this.addMessage('Game saved!', '#00ff00');
                    container.forEach(obj => obj.destroy());
                }},
                { text: 'Return to Main Menu', callback: () => {
                    this.saveGame();
                    this.scene.start('MenuScene');
                }},
                { text: 'Resume', callback: () => {
                    container.forEach(obj => obj.destroy());
                }}
            ];

            let yPos = 200;
            buttons.forEach(buttonData => {
                const btn = this.add.text(400, yPos, buttonData.text, {
                    fontSize: '20px',
                    fontFamily: 'Arial',
                    color: '#ffffff',
                    backgroundColor: '#3a3a3a',
                    padding: { x: 30, y: 15 },
                    align: 'center'
                }).setOrigin(0.5, 0).setInteractive({ useHandCursor: true });

                btn.on('pointerover', () => btn.setBackgroundColor('#4a4a4a'));
                btn.on('pointerout', () => btn.setBackgroundColor('#3a3a3a'));
                btn.on('pointerdown', buttonData.callback);

                container.push(btn);
                yPos += 70;
            });
        });
    }

    createModal(title, contentCallback) {
        const container = [];

        // Overlay
        const overlay = this.add.rectangle(0, 0, 800, 600, 0x000000, 0.85)
            .setOrigin(0, 0)
            .setInteractive();
        container.push(overlay);

        // Panel
        const panel = this.add.rectangle(400, 300, 700, 500, 0x2a2a2a)
            .setStrokeStyle(3, 0x8B0000);
        container.push(panel);

        // Title
        const titleText = this.add.text(400, 80, title, {
            fontSize: '32px',
            fontFamily: 'Arial Black',
            color: '#ffffff'
        }).setOrigin(0.5);
        container.push(titleText);

        // Content
        contentCallback(container);

        // Close button
        const closeBtn = this.add.text(400, 520, 'CLOSE', {
            fontSize: '20px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        closeBtn.on('pointerover', () => closeBtn.setBackgroundColor('#aa0000'));
        closeBtn.on('pointerout', () => closeBtn.setBackgroundColor('#8B0000'));
        closeBtn.on('pointerdown', () => {
            container.forEach(obj => obj.destroy());
        });
        container.push(closeBtn);
    }

    gameOver() {
        this.gameState.isAlive = false;
        this.isPaused = true;

        this.time.delayedCall(1000, () => {
            this.createModal('GAME OVER', (container) => {
                const stats = this.add.text(400, 200, [
                    `You survived ${this.gameState.day} days`,
                    ``,
                    `Final Stats:`,
                    `HP: ${Math.floor(this.gameState.hp)}`,
                    `Items Collected: ${this.inventoryManager.getTotalItems()}`,
                    ``,
                    `Cause of Death: ${this.gameState.hp <= 0 ? 'Combat/Starvation' : 'Unknown'}`
                ], {
                    fontSize: '20px',
                    fontFamily: 'Arial',
                    color: '#ffffff',
                    align: 'center',
                    lineSpacing: 10
                }).setOrigin(0.5);
                container.push(stats);

                // Remove close button, add menu button
                const menuBtn = this.add.text(400, 450, 'MAIN MENU', {
                    fontSize: '24px',
                    fontFamily: 'Arial',
                    color: '#ffffff',
                    backgroundColor: '#8B0000',
                    padding: { x: 30, y: 15 }
                }).setOrigin(0.5).setInteractive({ useHandCursor: true });

                menuBtn.on('pointerdown', () => {
                    // Clear save
                    localStorage.removeItem('buriedCitySave');
                    this.scene.start('MenuScene');
                });

                container.push(menuBtn);
            });
        });
    }

    saveGame() {
        const saveData = {
            gameState: this.gameState,
            inventory: this.inventoryManager.serialize(),
            location: this.locationManager.serialize(),
            timestamp: Date.now()
        };

        try {
            localStorage.setItem('buriedCitySave', JSON.stringify(saveData));
            return true;
        } catch (e) {
            console.error('Failed to save game:', e);
            return false;
        }
    }

    loadGame() {
        try {
            const saveData = JSON.parse(localStorage.getItem('buriedCitySave'));
            if (saveData) {
                this.gameState = saveData.gameState;
                this.inventoryManager.deserialize(saveData.inventory);
                this.locationManager.deserialize(saveData.location);
                return true;
            }
        } catch (e) {
            console.error('Failed to load game:', e);
        }
        return false;
    }
}
