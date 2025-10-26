class LootSelectionScene extends Phaser.Scene {
    constructor() {
        super({ key: 'LootSelectionScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.loot = data.loot;
        this.targetLocation = data.targetLocation;
        this.selectedItems = {};
    }

    create() {
        const centerX = this.cameras.main.width / 2;

        // Dark overlay
        this.add.rectangle(0, 0, 800, 600, 0x000000, 0.95).setOrigin(0, 0);

        // Title
        this.add.text(centerX, 30, 'LOOT FOUND', {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#00ff00'
        }).setOrigin(0.5);

        // Capacity display
        this.capacityText = this.add.text(centerX, 65, '', {
            fontSize: '16px',
            color: '#ffaa00'
        }).setOrigin(0.5);

        this.updateCapacityDisplay();

        if (this.loot.length === 0) {
            // No loot found
            this.add.text(centerX, 200, 'Nothing useful found...', {
                fontSize: '20px',
                color: '#888888'
            }).setOrigin(0.5);

            // Return button
            this.createReturnButton();
            return;
        }

        // Auto-add items and show what can't be carried
        const result = this.gameSceneRef.expeditionManager.processFoundLoot(this.loot);

        if (result.autoAdded.length > 0) {
            this.gameSceneRef.addMessage(`Automatically picked up ${result.autoAdded.length} item type(s).`, '#00ff00');
        }

        if (result.cannotCarry.length > 0) {
            this.gameSceneRef.addMessage(`⚠️ Can't carry ${result.cannotCarry.length} item type(s) - too heavy!`, '#ff8800');
        }

        // Show items in expedition
        this.showExpeditionItems();

        // Show items left behind
        if (result.cannotCarry.length > 0) {
            this.showLeftBehindItems(result.cannotCarry);
        }

        // Return button
        this.createReturnButton();
    }

    showExpeditionItems() {
        const items = this.gameSceneRef.expeditionManager.getExpeditionItems();

        if (items.length === 0) return;

        this.add.text(100, 110, '📦 CARRYING:', {
            fontSize: '18px',
            color: '#00ff00',
            fontStyle: 'bold'
        });

        let yPos = 140;
        items.forEach(item => {
            const totalWeight = item.weight * item.quantity;
            const itemText = this.add.text(110, yPos, `• ${item.name} x${item.quantity} (${totalWeight}kg)`, {
                fontSize: '14px',
                color: '#cccccc'
            });

            // Drop button
            const dropBtn = this.add.text(500, yPos, 'DROP', {
                fontSize: '12px',
                color: '#ffffff',
                backgroundColor: '#8B0000',
                padding: { x: 10, y: 3 }
            }).setInteractive({ useHandCursor: true });

            dropBtn.on('pointerover', () => dropBtn.setBackgroundColor('#aa0000'));
            dropBtn.on('pointerout', () => dropBtn.setBackgroundColor('#8B0000'));
            dropBtn.on('pointerdown', () => {
                // Drop item
                this.gameSceneRef.expeditionManager.removeLootFromExpedition(item.id, item.quantity);
                this.gameSceneRef.addMessage(`Dropped ${item.name} x${item.quantity}`, '#888888');
                this.scene.restart();
            });

            yPos += 25;
        });
    }

    showLeftBehindItems(leftBehind) {
        this.add.text(100, 320, '❌ LEFT BEHIND (too heavy):', {
            fontSize: '16px',
            color: '#ff8800',
            fontStyle: 'bold'
        });

        let yPos = 350;
        leftBehind.forEach(item => {
            const totalWeight = item.weight;
            this.add.text(110, yPos, `• ${item.name} x${item.quantity} (${totalWeight}kg)`, {
                fontSize: '14px',
                color: '#888888'
            });

            yPos += 20;
        });
    }

    updateCapacityDisplay() {
        const capacity = this.gameSceneRef.expeditionManager.getExpeditionCapacity();
        const percentage = (capacity.current / capacity.max) * 100;
        let color = '#00ff00';
        if (percentage > 80) color = '#ff8800';
        if (percentage >= 100) color = '#ff0000';

        this.capacityText.setText(`Carrying: ${capacity.current.toFixed(1)}/${capacity.max}kg (${percentage.toFixed(0)}%)`);
        this.capacityText.setColor(color);
    }

    createReturnButton() {
        const centerX = this.cameras.main.width / 2;

        const returnBtn = this.add.text(centerX, 520, 'RETURN TO SHELTER', {
            fontSize: '24px',
            color: '#ffffff',
            backgroundColor: '#00aa00',
            padding: { x: 30, y: 12 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        returnBtn.on('pointerover', () => returnBtn.setBackgroundColor('#00cc00'));
        returnBtn.on('pointerout', () => returnBtn.setBackgroundColor('#00aa00'));
        returnBtn.on('pointerdown', () => {
            this.returnToShelter();
        });

        // Warning about travel back
        const travelCost = this.gameSceneRef.locationManager.calculateTravelCost(this.targetLocation.id);
        this.add.text(centerX, 560, `(Travel back: ${travelCost.time}h, ${travelCost.energyCost} energy, ${Math.floor(travelCost.encounterChance * 100)}% encounter)`, {
            fontSize: '12px',
            color: '#888888'
        }).setOrigin(0.5);
    }

    returnToShelter() {
        // Travel back to shelter
        const travelResult = this.gameSceneRef.expeditionManager.executeTravel(
            'shelter',
            this.gameSceneRef.gameState
        );

        if (!travelResult.success) {
            this.gameSceneRef.addMessage(travelResult.message, '#ff0000');
            return;
        }

        // Apply travel time
        this.gameSceneRef.advanceTime(travelResult.travelTime);

        // Check for travel encounter on way back
        if (travelResult.travelEncounter.encountered) {
            this.gameSceneRef.addMessage(`⚠️ Ambushed on way back! ${travelResult.travelEncounter.count} zombie(s)!`, '#ff0000');
            this.scene.stop();
            this.gameSceneRef.handleCombat(travelResult.travelEncounter, () => {
                this.completeExpedition();
            });
        } else {
            this.gameSceneRef.addMessage('Returned to shelter safely.', '#00ff00');
            this.scene.stop();
            this.completeExpedition();
        }
    }

    completeExpedition() {
        // Transfer expedition items to main inventory
        const expeditionItems = this.gameSceneRef.expeditionManager.completeExpedition();

        let itemCount = 0;
        for (const quantity of Object.values(expeditionItems)) {
            itemCount += quantity;
        }

        if (itemCount > 0) {
            this.gameSceneRef.addMessage(`✅ Expedition complete! Brought back ${itemCount} item(s).`, '#00ff00');
        } else {
            this.gameSceneRef.addMessage('Expedition complete. Returned empty-handed.', '#888888');
        }

        this.gameSceneRef.updateUI();
    }
}
