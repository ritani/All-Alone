class MapScene extends Phaser.Scene {
    constructor() {
        super({ key: 'MapScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.action = data.action; // 'explore' or 'scavenge'
    }

    create() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Dark overlay
        this.add.rectangle(0, 0, 800, 600, 0x000000, 0.9).setOrigin(0, 0);

        // Title
        this.add.text(centerX, 40, 'MAP - SELECT DESTINATION', {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#ffaa00'
        }).setOrigin(0.5);

        // Current location indicator
        const currentLoc = this.gameSceneRef.locationManager.getCurrentLocation();
        this.add.text(centerX, 75, `Current Location: ${currentLoc.name}`, {
            fontSize: '16px',
            color: '#88ccff'
        }).setOrigin(0.5);

        // Map area
        const mapBg = this.add.rectangle(centerX, 300, 700, 400, 0x1a1a1a).setStrokeStyle(2, 0x4a4a4a);

        // Draw locations
        this.drawLocations();

        // Legend
        this.drawLegend();

        // Close button
        const closeBtn = this.add.text(centerX, 550, 'BACK', {
            fontSize: '20px',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        closeBtn.on('pointerover', () => closeBtn.setBackgroundColor('#aa0000'));
        closeBtn.on('pointerout', () => closeBtn.setBackgroundColor('#8B0000'));
        closeBtn.on('pointerdown', () => {
            this.scene.stop();
        });
    }

    drawLocations() {
        const locations = this.gameSceneRef.locationManager.getAllLocations();
        const currentLocId = this.gameSceneRef.locationManager.currentLocation;

        locations.forEach(location => {
            const x = location.mapPosition.x;
            const y = location.mapPosition.y;

            if (!location.unlocked) {
                // Unknown location - show as ???
                const marker = this.add.text(x, y, '?', {
                    fontSize: '32px',
                    color: '#666666'
                }).setOrigin(0.5);
                return;
            }

            // Location marker
            const isCurrentLocation = location.id === currentLocId;
            const markerColor = isCurrentLocation ? 0x00ff00 : location.id === 'shelter' ? 0x4444ff : 0xff8800;

            const marker = this.add.circle(x, y, 12, markerColor).setStrokeStyle(2, 0xffffff);

            // Location name
            const nameText = this.add.text(x, y + 25, location.name, {
                fontSize: '12px',
                color: '#ffffff',
                backgroundColor: '#000000aa',
                padding: { x: 5, y: 2 }
            }).setOrigin(0.5);

            // Make clickable if not shelter and not current location
            if (location.id !== 'shelter' && location.id !== currentLocId) {
                marker.setInteractive({ useHandCursor: true });
                nameText.setInteractive({ useHandCursor: true });

                const showInfo = () => {
                    this.showLocationInfo(location);
                };

                marker.on('pointerover', () => {
                    marker.setScale(1.3);
                    nameText.setBackgroundColor('#333333');
                });

                marker.on('pointerout', () => {
                    marker.setScale(1);
                    nameText.setBackgroundColor('#000000aa');
                });

                marker.on('pointerdown', showInfo);
                nameText.on('pointerdown', showInfo);
            }

            // Draw path lines from shelter
            if (location.id !== 'shelter') {
                const shelterLoc = Locations['shelter'];
                const line = this.add.line(0, 0, shelterLoc.mapPosition.x, shelterLoc.mapPosition.y, x, y, 0x444444);
                line.setOrigin(0, 0);
                line.setDepth(-1);
            }
        });
    }

    drawLegend() {
        const legendX = 50;
        const legendY = 120;

        this.add.text(legendX, legendY, 'LEGEND:', {
            fontSize: '14px',
            color: '#ffaa00',
            fontStyle: 'bold'
        });

        const items = [
            { color: 0x00ff00, text: 'Current Location' },
            { color: 0x4444ff, text: 'Your Shelter' },
            { color: 0xff8800, text: 'Available Location' },
            { color: 0x666666, text: 'Undiscovered (???)' }
        ];

        items.forEach((item, index) => {
            const y = legendY + 25 + (index * 20);
            this.add.circle(legendX + 8, y + 6, 6, item.color);
            this.add.text(legendX + 20, y, item.text, {
                fontSize: '12px',
                color: '#cccccc'
            });
        });
    }

    showLocationInfo(location) {
        // Remove previous info if exists
        if (this.locationInfoContainer) {
            this.locationInfoContainer.forEach(obj => obj.destroy());
        }

        this.locationInfoContainer = [];

        // Info panel
        const panel = this.add.rectangle(400, 300, 500, 350, 0x2a2a2a, 0.95)
            .setStrokeStyle(3, 0xffaa00);
        this.locationInfoContainer.push(panel);

        // Title
        const title = this.add.text(400, 160, location.name.toUpperCase(), {
            fontSize: '24px',
            fontFamily: 'Arial Black',
            color: '#ffaa00'
        }).setOrigin(0.5);
        this.locationInfoContainer.push(title);

        // Description
        const desc = this.add.text(400, 200, location.description, {
            fontSize: '16px',
            color: '#cccccc',
            wordWrap: { width: 450 },
            align: 'center'
        }).setOrigin(0.5);
        this.locationInfoContainer.push(desc);

        // Stats
        const travelCost = this.gameSceneRef.locationManager.calculateTravelCost(location.id);
        const dangerStars = '⚠️'.repeat(location.danger);

        const statsText = [
            `Danger: ${dangerStars} (${location.danger}/5)`,
            `Travel Time: ${travelCost.time} hour(s)`,
            `Energy Cost: ${travelCost.energyCost}`,
            `Distance: ${location.distance} km`,
            `Encounter Chance: ${Math.floor(travelCost.encounterChance * 100)}%`
        ].join('\n');

        const stats = this.add.text(400, 280, statsText, {
            fontSize: '14px',
            color: '#aaaaaa',
            align: 'center',
            lineSpacing: 5
        }).setOrigin(0.5);
        this.locationInfoContainer.push(stats);

        // Check if player has enough energy
        const playerEnergy = this.gameSceneRef.gameState.energy;
        const canTravel = playerEnergy >= travelCost.energyCost;

        if (!canTravel) {
            const warning = this.add.text(400, 370, `Not enough energy! Need ${travelCost.energyCost}, have ${Math.floor(playerEnergy)}`, {
                fontSize: '14px',
                color: '#ff0000'
            }).setOrigin(0.5);
            this.locationInfoContainer.push(warning);
        }

        // Travel button
        const travelBtn = this.add.text(400, canTravel ? 390 : 410, 'TRAVEL HERE', {
            fontSize: '20px',
            color: canTravel ? '#ffffff' : '#666666',
            backgroundColor: canTravel ? '#00aa00' : '#333333',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5);

        if (canTravel) {
            travelBtn.setInteractive({ useHandCursor: true });
            travelBtn.on('pointerover', () => travelBtn.setBackgroundColor('#00cc00'));
            travelBtn.on('pointerout', () => travelBtn.setBackgroundColor('#00aa00'));
            travelBtn.on('pointerdown', () => {
                this.initiateTravel(location);
            });
        }

        this.locationInfoContainer.push(travelBtn);

        // Cancel button
        const cancelBtn = this.add.text(400, 440, 'CANCEL', {
            fontSize: '18px',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 20, y: 8 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        cancelBtn.on('pointerover', () => cancelBtn.setBackgroundColor('#aa0000'));
        cancelBtn.on('pointerout', () => cancelBtn.setBackgroundColor('#8B0000'));
        cancelBtn.on('pointerdown', () => {
            this.locationInfoContainer.forEach(obj => obj.destroy());
            this.locationInfoContainer = null;
        });

        this.locationInfoContainer.push(cancelBtn);
    }

    initiateTravel(location) {
        // Start expedition planning
        this.scene.stop();
        this.scene.launch('ExpeditionPlanScene', {
            gameScene: this.gameSceneRef,
            targetLocation: location,
            action: this.action
        });
    }
}
