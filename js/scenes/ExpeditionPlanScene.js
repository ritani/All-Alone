class ExpeditionPlanScene extends Phaser.Scene {
    constructor() {
        super({ key: 'ExpeditionPlanScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.targetLocation = data.targetLocation;
        this.action = data.action;
    }

    create() {
        const centerX = this.cameras.main.width / 2;

        // Dark overlay
        this.add.rectangle(0, 0, 800, 600, 0x000000, 0.95).setOrigin(0, 0);

        // Title
        this.add.text(centerX, 30, 'EXPEDITION PLANNING', {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#ffaa00'
        }).setOrigin(0.5);

        // Destination
        this.add.text(centerX, 65, `Destination: ${this.targetLocation.name}`, {
            fontSize: '18px',
            color: '#88ccff'
        }).setOrigin(0.5);

        // Expedition info panel
        const travelCost = this.gameSceneRef.locationManager.calculateTravelCost(this.targetLocation.id);

        const infoText = [
            `Travel Time: ${travelCost.time} hour(s)`,
            `Energy Cost: ${travelCost.energyCost}`,
            `${this.action === 'explore' ? 'Exploration' : 'Scavenging'} Time: ${this.action === 'explore' ? GameConstants.EXPLORE_TIME_COST : GameConstants.SCAVENGE_TIME_COST} hour(s)`,
            `Total Time: ${travelCost.time + (this.action === 'explore' ? GameConstants.EXPLORE_TIME_COST : GameConstants.SCAVENGE_TIME_COST) + travelCost.time} hours`,
            ``,
            `Carrying Capacity: ${GameConstants.EXPEDITION_MAX_WEIGHT} kg`,
            `Encounter Chance: ${Math.floor(travelCost.encounterChance * 100)}% each way`
        ].join('\n');

        this.add.text(centerX, 170, infoText, {
            fontSize: '14px',
            color: '#cccccc',
            align: 'center',
            lineSpacing: 5
        }).setOrigin(0.5);

        // Warning
        this.add.text(centerX, 290, '⚠️ You can only carry limited items back. Choose wisely! ⚠️', {
            fontSize: '14px',
            color: '#ff8800',
            align: 'center'
        }).setOrigin(0.5);

        // Departure button
        const departBtn = this.add.text(centerX, 350, 'BEGIN EXPEDITION', {
            fontSize: '24px',
            color: '#ffffff',
            backgroundColor: '#00aa00',
            padding: { x: 40, y: 15 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        departBtn.on('pointerover', () => departBtn.setBackgroundColor('#00cc00'));
        departBtn.on('pointerout', () => departBtn.setBackgroundColor('#00aa00'));
        departBtn.on('pointerdown', () => {
            this.beginExpedition();
        });

        // Cancel button
        const cancelBtn = this.add.text(centerX, 410, 'CANCEL', {
            fontSize: '20px',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        cancelBtn.on('pointerover', () => cancelBtn.setBackgroundColor('#aa0000'));
        cancelBtn.on('pointerout', () => cancelBtn.setBackgroundColor('#8B0000'));
        cancelBtn.on('pointerdown', () => {
            this.scene.stop();
        });

        // Tips
        const tips = [
            '💡 Tips:',
            '• Lighter items = more items you can carry',
            '• Prioritize medicine and food',
            '• Zombies may attack during travel!',
            '• Make sure you have enough energy to return'
        ].join('\n');

        this.add.text(centerX, 490, tips, {
            fontSize: '12px',
            color: '#888888',
            align: 'center',
            lineSpacing: 3
        }).setOrigin(0.5);
    }

    beginExpedition() {
        // Initialize expedition manager
        if (!this.gameSceneRef.expeditionManager) {
            this.gameSceneRef.expeditionManager = new ExpeditionManager(
                this.gameSceneRef,
                this.gameSceneRef.inventoryManager,
                this.gameSceneRef.locationManager
            );
        }

        this.gameSceneRef.expeditionManager.startExpeditionPlanning();

        // Execute travel
        const travelResult = this.gameSceneRef.expeditionManager.executeTravel(
            this.targetLocation.id,
            this.gameSceneRef.gameState
        );

        if (!travelResult.success) {
            // Show error and return
            this.scene.stop();
            this.gameSceneRef.addMessage(travelResult.message, '#ff0000');
            return;
        }

        // Store travel result
        this.travelResult = travelResult;

        // Launch animated travel scene
        this.scene.stop();
        this.scene.launch('TravelAnimationScene', {
            gameScene: this.gameSceneRef,
            targetLocation: this.targetLocation,
            travelResult: travelResult,
            returning: false,
            onComplete: () => {
                // Travel complete - apply time and handle results
                this.handleTravelComplete();
            }
        });
    }

    handleTravelComplete() {
        // Apply travel time
        this.gameSceneRef.advanceTime(this.travelResult.travelTime);

        // Check for travel encounter
        if (this.travelResult.travelEncounter.encountered) {
            this.gameSceneRef.addMessage(`⚠️ Zombies attack during travel! ${this.travelResult.travelEncounter.count} zombie(s)!`, '#ff0000');
            this.gameSceneRef.handleCombat(this.travelResult.travelEncounter, () => {
                // After combat, proceed with exploration
                this.proceedToExploration();
            });
        } else {
            this.gameSceneRef.addMessage(`Traveled to ${this.travelResult.location.name} safely.`, '#00ff00');

            // Check for discovery
            if (this.travelResult.discoveredLocation) {
                const discoveredLoc = Locations[this.travelResult.discoveredLocation];
                this.gameSceneRef.addMessage(`🗺️ Discovered new location: ${discoveredLoc.name}!`, '#ffaa00');
            }

            // Proceed to exploration
            this.proceedToExploration();
        }
    }

    proceedToExploration() {
        this.gameSceneRef.addMessage(`Entering ${this.targetLocation.name}...`, '#88ccff');

        // Launch area-based exploration scene
        this.scene.launch('AreaExplorationScene', {
            gameScene: this.gameSceneRef,
            targetLocation: this.targetLocation,
            action: this.action
        });
    }
}
