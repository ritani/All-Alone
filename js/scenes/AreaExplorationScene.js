class AreaExplorationScene extends Phaser.Scene {
    constructor() {
        super({ key: 'AreaExplorationScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.targetLocation = data.targetLocation;
        this.action = data.action;
        this.areas = LocationAreas[this.targetLocation.id] || [];

        // Initialize area states for this expedition if not exists
        if (!this.gameSceneRef.expeditionManager.areaStates) {
            this.gameSceneRef.expeditionManager.areaStates = {};
        }
        if (!this.gameSceneRef.expeditionManager.areaStates[this.targetLocation.id]) {
            this.initializeAreaStates();
        }

        this.areaStates = this.gameSceneRef.expeditionManager.areaStates[this.targetLocation.id];
        this.currentMode = 'map'; // 'map', 'combat', 'search'
        this.selectedArea = null;
    }

    initializeAreaStates() {
        const states = {};
        this.areas.forEach((area, index) => {
            states[area.id] = {
                status: index === 0 ? 'unlocked' : 'locked', // First area always unlocked
                zombieDefeated: false,
                itemsSearched: false,
                remainingItems: null, // Will be populated when items are generated
                lastVisited: null
            };
        });
        this.gameSceneRef.expeditionManager.areaStates[this.targetLocation.id] = states;
    }

    create() {
        const centerX = this.cameras.main.width / 2;
        const visuals = LocationVisuals[this.targetLocation.id] || LocationVisuals['shelter'];

        // Themed background
        this.add.rectangle(0, 0, 800, 600, Phaser.Display.Color.HexStringToColor(visuals.bgColor).color).setOrigin(0, 0);

        // Create Vital Stats HUD
        this.vitalStatsHUD = new VitalStatsHUD(this, this.gameSceneRef.gameState);
        this.vitalStatsHUD.create();

        // Create Expedition HUD
        this.expeditionHUD = new ExpeditionHUD(this, this.gameSceneRef.expeditionManager);
        this.expeditionHUD.create();

        // Location banner
        const bannerY = 70;
        this.add.rectangle(centerX, bannerY, 700, 40, Phaser.Display.Color.HexStringToColor(visuals.accentColor).color)
            .setAlpha(0.9);

        this.add.text(80, bannerY, visuals.icon, {
            fontSize: '30px'
        }).setOrigin(0.5);

        this.add.text(centerX, bannerY, `EXPLORING: ${this.targetLocation.name.toUpperCase()}`, {
            fontSize: '20px',
            fontFamily: 'Arial Black',
            color: '#ffffff'
        }).setOrigin(0.5);

        this.add.text(720, bannerY, visuals.emoji, {
            fontSize: '18px'
        }).setOrigin(0.5);

        // Instructions
        this.instructionText = this.add.text(centerX, 120, 'Click on an unlocked area to explore', {
            fontSize: '14px',
            color: '#cccccc'
        }).setOrigin(0.5);

        // Show map view by default
        this.showMapView();
    }

    showMapView() {
        this.currentMode = 'map';
        this.clearDynamicContent();

        // Draw area map
        this.areas.forEach(area => {
            const state = this.areaStates[area.id];
            this.drawAreaNode(area, state);
        });

        // Draw connections
        this.drawConnections();

        // Exit button
        this.createExitButton();
    }

    drawAreaNode(area, state) {
        const x = area.position.x;
        const y = area.position.y + 100; // Offset for HUD

        // Determine node appearance based on state
        let bgColor, strokeColor, textColor, statusIcon;

        if (state.status === 'locked') {
            bgColor = 0x333333;
            strokeColor = 0x666666;
            textColor = '#666666';
            statusIcon = '🔒';
        } else if (state.zombieDefeated && state.itemsSearched) {
            bgColor = 0x004400;
            strokeColor = 0x00ff00;
            textColor = '#00ff00';
            statusIcon = '✅';
        } else if (state.zombieDefeated || state.itemsSearched) {
            bgColor = 0x444400;
            strokeColor = 0xffaa00;
            textColor = '#ffaa00';
            statusIcon = '⚠️';
        } else {
            bgColor = 0x442200;
            strokeColor = 0xffaa00;
            textColor = '#ffffff';
            statusIcon = '❓';
        }

        // Node circle
        const node = this.add.circle(x, y, 30, bgColor)
            .setStrokeStyle(3, strokeColor);

        // Status icon
        const icon = this.add.text(x, y - 5, statusIcon, {
            fontSize: '24px'
        }).setOrigin(0.5);

        // Area name
        const nameText = this.add.text(x, y + 50, area.name, {
            fontSize: '11px',
            color: textColor,
            backgroundColor: '#00000099',
            padding: { x: 4, y: 2 },
            align: 'center',
            wordWrap: { width: 100 }
        }).setOrigin(0.5);

        // Make clickable if unlocked
        if (state.status === 'unlocked') {
            node.setInteractive({ useHandCursor: true });
            icon.setInteractive({ useHandCursor: true });
            nameText.setInteractive({ useHandCursor: true });

            const onClick = () => this.selectArea(area);
            const onOver = () => {
                node.setScale(1.2);
                icon.setScale(1.2);
                nameText.setBackgroundColor('#333333');
            };
            const onOut = () => {
                node.setScale(1);
                icon.setScale(1);
                nameText.setBackgroundColor('#00000099');
            };

            node.on('pointerdown', onClick);
            icon.on('pointerdown', onClick);
            nameText.on('pointerdown', onClick);

            node.on('pointerover', onOver);
            icon.on('pointerover', onOver);
            nameText.on('pointerover', onOver);

            node.on('pointerout', onOut);
            icon.on('pointerout', onOut);
            nameText.on('pointerout', onOut);
        }

        this.dynamicObjects.push(node, icon, nameText);
    }

    drawConnections() {
        this.areas.forEach(area => {
            if (area.unlocks && area.unlocks.length > 0) {
                area.unlocks.forEach(targetId => {
                    const targetArea = this.areas.find(a => a.id === targetId);
                    if (targetArea) {
                        const x1 = area.position.x;
                        const y1 = area.position.y + 100;
                        const x2 = targetArea.position.x;
                        const y2 = targetArea.position.y + 100;

                        const line = this.add.line(0, 0, x1, y1, x2, y2, 0x666666, 0.5)
                            .setOrigin(0, 0)
                            .setLineWidth(2);

                        this.dynamicObjects.push(line);
                    }
                });
            }
        });
    }

    selectArea(area) {
        this.selectedArea = area;
        const state = this.areaStates[area.id];

        // Check if area has zombie
        const hasZombie = !state.zombieDefeated && Math.random() < area.zombieChance;

        if (hasZombie) {
            this.encounterZombie(area);
        } else {
            this.searchArea(area);
        }
    }

    encounterZombie(area) {
        this.currentMode = 'combat';
        this.clearDynamicContent();

        const day = this.gameSceneRef.gameState.day;
        const zombieCount = Math.floor(Math.random() * 2) + 1;
        const zombieHP = GameConstants.ZOMBIE_BASE_HP + (day * GameConstants.ZOMBIE_HP_SCALING);
        const zombieDamage = 8 + (day * GameConstants.ZOMBIE_DAMAGE_SCALING);

        this.instructionText.setText(`💀 ZOMBIE ENCOUNTER!`);

        const centerX = this.cameras.main.width / 2;

        const infoText = this.add.text(centerX, 200,
            `${zombieCount} ZOMBIE(S) in ${area.name}!\n\nHP: ${zombieHP} each | Damage: ${zombieDamage} | Day ${day}`, {
            fontSize: '18px',
            color: '#ff0000',
            align: 'center'
        }).setOrigin(0.5);

        this.dynamicObjects.push(infoText);

        // Fight button
        const fightBtn = this.createActionButton(centerX - 100, 300, 'FIGHT ⚔️', '#8B0000', () => {
            this.fightZombie(area, zombieCount, zombieHP, zombieDamage);
        });

        // Flee button
        const fleeBtn = this.createActionButton(centerX + 100, 300, 'FLEE 🏃', '#444444', () => {
            this.fleeFromZombie(area, zombieCount, zombieDamage);
        });

        this.dynamicObjects.push(fightBtn);
        this.dynamicObjects.push(fleeBtn);
    }

    fightZombie(area, zombieCount, zombieHP, zombieDamage) {
        const weaponDamage = this.gameSceneRef.inventoryManager.getWeaponDamage();
        const totalZombieHP = zombieCount * zombieHP;
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
        this.vitalStatsHUD.update();

        if (this.gameSceneRef.gameState.hp <= 0) {
            this.gameSceneRef.addMessage('💀 Killed by zombies!', '#ff0000');
            this.gameSceneRef.gameOver();
            this.scene.stop();
            return;
        }

        this.gameSceneRef.addMessage(`⚔️ Defeated ${zombieCount} zombie(s) in ${area.name}. Took ${actualDamageTaken} damage.`, '#ffaa00');

        // Mark zombie as defeated
        this.areaStates[area.id].zombieDefeated = true;

        // Unlock connected areas
        this.unlockAreas(area);

        // Now search the area
        this.searchArea(area);
    }

    fleeFromZombie(area, zombieCount, zombieDamage) {
        // 40% chance to take damage when fleeing
        const hitChance = 0.4;
        let damageTaken = 0;

        if (Math.random() < hitChance) {
            damageTaken = Math.floor(zombieDamage * zombieCount * 0.5); // Half damage
            this.gameSceneRef.gameState.hp = Math.max(0, this.gameSceneRef.gameState.hp - damageTaken);
            this.vitalStatsHUD.update();

            if (this.gameSceneRef.gameState.hp <= 0) {
                this.gameSceneRef.addMessage('💀 Hit while fleeing! You died.', '#ff0000');
                this.gameSceneRef.gameOver();
                this.scene.stop();
                return;
            }

            this.gameSceneRef.addMessage(`🏃 Fled from ${area.name}! Took ${damageTaken} damage while escaping.`, '#ffaa00');
        } else {
            this.gameSceneRef.addMessage(`🏃 Successfully fled from ${area.name}!`, '#888888');
        }

        // Return to map
        this.showMapView();
    }

    searchArea(area) {
        this.currentMode = 'search';
        this.clearDynamicContent();

        const state = this.areaStates[area.id];
        const container = ContainerTypes[area.containerType];
        const centerX = this.cameras.main.width / 2;

        this.instructionText.setText(`Searching: ${area.name}`);

        // Check if area has items (or has remaining items from before)
        const hasItems = state.remainingItems || (!state.itemsSearched && Math.random() < area.itemChance);

        if (hasItems) {
            if (container.locked && !state.itemsSearched) {
                this.handleLockedContainer(area, container);
            } else {
                this.findItems(area, container, state);
            }
        } else {
            const infoText = this.add.text(centerX, 250,
                `Searched ${container.name}.\nNo useful items found.`, {
                fontSize: '16px',
                color: '#888888',
                align: 'center'
            }).setOrigin(0.5);

            this.dynamicObjects.push(infoText);

            // Mark as searched
            this.areaStates[area.id].itemsSearched = true;
            this.unlockAreas(area);

            setTimeout(() => {
                this.showMapView();
            }, 1500);
        }
    }

    handleLockedContainer(area, container) {
        const centerX = this.cameras.main.width / 2;

        const infoText = this.add.text(centerX, 200,
            `🔒 LOCKED ${container.name.toUpperCase()}!\n\nTime to open: ${container.timeToSearch}h`, {
            fontSize: '16px',
            color: '#ffaa00',
            align: 'center'
        }).setOrigin(0.5);

        this.dynamicObjects.push(infoText);

        // Check for tools
        const tools = this.gameSceneRef.inventoryManager.getInventoryList().filter(item => item.utility);
        const hasTool = tools.length > 0;

        const forceTime = container.timeToSearch;

        // Force open button
        const forceBtn = this.createActionButton(centerX, 280, `FORCE OPEN (${forceTime}h)`, '#8B0000', () => {
            this.openContainer(area, container, forceTime);
        });
        this.dynamicObjects.push(forceBtn);

        // Tool buttons
        if (hasTool) {
            let yPos = 340;
            tools.forEach(tool => {
                const reducedTime = Math.max(0.25, forceTime * GameConstants.TOOL_TIME_REDUCTION);
                const toolBtn = this.createActionButton(centerX, yPos,
                    `Use ${tool.name} (${reducedTime.toFixed(2)}h) ⚡`, '#006600', () => {
                    this.gameSceneRef.addMessage(`Used ${tool.name} to break open container.`, '#00ff00');
                    this.openContainer(area, container, reducedTime);
                });
                this.dynamicObjects.push(toolBtn);
                yPos += 50;
            });
        }

        // Skip button
        const skipBtn = this.createActionButton(centerX, hasTool ? 450 : 340, 'SKIP (Leave Locked)', '#444444', () => {
            this.gameSceneRef.addMessage(`Left ${area.name} locked.`, '#888888');
            this.showMapView();
        });
        this.dynamicObjects.push(skipBtn);
    }

    openContainer(area, container, timeSpent) {
        this.gameSceneRef.advanceTime(timeSpent);
        this.vitalStatsHUD.update();
        this.findItems(area, container, this.areaStates[area.id]);
    }

    findItems(area, container, state) {
        this.clearDynamicContent();
        const centerX = this.cameras.main.width / 2;

        // Generate or use existing items
        let foundItems = state.remainingItems;

        if (!foundItems) {
            // Generate new items
            foundItems = [];
            const location = this.targetLocation;
            const numItems = Math.floor(Math.random() * 3) + 1;

            for (let i = 0; i < numItems; i++) {
                if (Math.random() > 0.3) {
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
        }

        if (foundItems.length === 0) {
            const infoText = this.add.text(centerX, 250,
                `Searched ${container.name}.\nContainer was empty.`, {
                fontSize: '16px',
                color: '#888888',
                align: 'center'
            }).setOrigin(0.5);

            this.dynamicObjects.push(infoText);

            this.areaStates[area.id].itemsSearched = true;
            this.areaStates[area.id].remainingItems = null;
            this.unlockAreas(area);

            setTimeout(() => {
                this.showMapView();
            }, 1500);
            return;
        }

        // Show items
        const itemList = foundItems.map(item => `• ${item.name} x${item.quantity} (${item.weight}kg)`).join('\n');
        const infoText = this.add.text(centerX, 180,
            `✅ FOUND IN ${container.name.toUpperCase()}:\n\n${itemList}`, {
            fontSize: '14px',
            color: '#00ff00',
            align: 'left'
        }).setOrigin(0.5);

        this.dynamicObjects.push(infoText);

        // Try to add items to expedition inventory
        const capacity = this.gameSceneRef.expeditionManager.getExpeditionCapacity();
        const itemsTaken = [];
        const itemsLeft = [];

        foundItems.forEach(item => {
            const result = this.gameSceneRef.expeditionManager.addExpeditionItem(item.id, item.quantity);
            if (result.added === item.quantity) {
                itemsTaken.push(item);
            } else if (result.added > 0) {
                itemsTaken.push({ ...item, quantity: result.added });
                itemsLeft.push({ ...item, quantity: item.quantity - result.added });
            } else {
                itemsLeft.push(item);
            }
        });

        // Update HUD
        this.expeditionHUD.update();

        // Show what was taken/left
        let statusMsg = '';
        if (itemsTaken.length > 0) {
            statusMsg += `✅ Collected ${itemsTaken.length} item type(s)\n`;
        }
        if (itemsLeft.length > 0) {
            statusMsg += `⚠️ Left ${itemsLeft.length} item type(s) (capacity full)\nCome back later to collect them!`;
            this.areaStates[area.id].remainingItems = itemsLeft;
        } else {
            statusMsg += `✅ All items collected!`;
            this.areaStates[area.id].itemsSearched = true;
            this.areaStates[area.id].remainingItems = null;
        }

        const statusText = this.add.text(centerX, 350, statusMsg, {
            fontSize: '12px',
            color: itemsLeft.length > 0 ? '#ffaa00' : '#00ff00',
            align: 'center'
        }).setOrigin(0.5);

        this.dynamicObjects.push(statusText);

        // Advance time
        this.gameSceneRef.advanceTime(container.timeToSearch);
        this.vitalStatsHUD.update();

        // Unlock connected areas
        this.unlockAreas(area);

        // Continue button
        const continueBtn = this.createActionButton(centerX, 450, 'CONTINUE', '#006600', () => {
            this.showMapView();
        });
        this.dynamicObjects.push(continueBtn);
    }

    unlockAreas(area) {
        if (area.unlocks && area.unlocks.length > 0) {
            area.unlocks.forEach(targetId => {
                if (this.areaStates[targetId]) {
                    this.areaStates[targetId].status = 'unlocked';
                }
            });
        }
    }

    createActionButton(x, y, text, color, callback) {
        const btn = this.add.text(x, y, text, {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: color,
            padding: { x: 20, y: 10 }
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

        return btn;
    }

    createExitButton() {
        const centerX = this.cameras.main.width / 2;
        const exitBtn = this.createActionButton(centerX, 550, 'FINISH & RETURN TO SHELTER', '#00aa00', () => {
            this.completeExploration();
        });
        this.dynamicObjects.push(exitBtn);
    }

    completeExploration() {
        // Collect all loot that was taken
        const allLoot = [];
        // Expedition manager already has the loot, just transition

        this.scene.stop();
        this.scene.launch('LootSelectionScene', {
            gameScene: this.gameSceneRef,
            loot: [], // Empty because items are already in expedition inventory
            targetLocation: this.targetLocation
        });
    }

    clearDynamicContent() {
        if (!this.dynamicObjects) {
            this.dynamicObjects = [];
        }
        this.dynamicObjects.forEach(obj => {
            if (obj && obj.destroy) {
                obj.destroy();
            }
        });
        this.dynamicObjects = [];
    }

    brightenColor(hexColor) {
        const hex = hexColor.replace('#', '');
        const num = parseInt(hex, 16);
        const r = Math.min(255, ((num >> 16) & 0xFF) + 40);
        const g = Math.min(255, ((num >> 8) & 0xFF) + 40);
        const b = Math.min(255, (num & 0xFF) + 40);
        return '#' + ((r << 16) | (g << 8) | b).toString(16).padStart(6, '0');
    }

    shutdown() {
        this.clearDynamicContent();
        if (this.vitalStatsHUD) {
            this.vitalStatsHUD.destroy();
        }
        if (this.expeditionHUD) {
            this.expeditionHUD.destroy();
        }
    }
}
