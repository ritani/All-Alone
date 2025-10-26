class ShelterStorageScene extends Phaser.Scene {
    constructor() {
        super({ key: 'ShelterStorageScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
    }

    create() {
        const centerX = this.cameras.main.width / 2;

        // Dark overlay
        this.add.rectangle(0, 0, 800, 600, 0x000000, 0.9).setOrigin(0, 0).setInteractive();

        // Main panel
        const panel = this.add.rectangle(centerX, 300, 750, 550, 0x2a2a2a)
            .setStrokeStyle(3, 0x4a8a4a);

        // Title
        this.add.text(centerX, 50, '🏠 SHELTER STORAGE', {
            fontSize: '32px',
            fontFamily: 'Arial Black',
            color: '#4a8a4a'
        }).setOrigin(0.5);

        // Storage capacity info
        const capacity = this.gameSceneRef.shelterManager.getStorageInfo();
        this.capacityText = this.add.text(centerX, 90,
            `Storage: ${capacity.used}/${capacity.max} items`, {
            fontSize: '16px',
            color: capacity.percentage > 90 ? '#ff8800' : '#cccccc'
        }).setOrigin(0.5);

        // Two columns: Inventory and Storage
        this.createInventoryColumn();
        this.createStorageColumn();

        // Close button
        const closeBtn = this.add.text(centerX, 540, 'CLOSE', {
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

    createInventoryColumn() {
        const columnX = 150;

        // Header
        this.add.text(columnX, 120, '📦 YOUR INVENTORY', {
            fontSize: '18px',
            fontFamily: 'Arial Bold',
            color: '#ffaa00'
        });

        // List area background
        const listBg = this.add.rectangle(columnX, 300, 300, 350, 0x1a1a1a)
            .setOrigin(0, 0);

        // Get inventory items
        const inventoryItems = this.gameSceneRef.inventoryManager.getInventoryList();

        if (inventoryItems.length === 0) {
            this.add.text(columnX + 150, 350, 'Inventory is empty', {
                fontSize: '14px',
                color: '#666666'
            }).setOrigin(0.5);
            return;
        }

        let yPos = 160;
        inventoryItems.forEach(item => {
            if (yPos > 480) return; // Prevent overflow

            // Item text
            const itemText = this.add.text(columnX, yPos,
                `${item.name} x${item.quantity}`, {
                fontSize: '14px',
                color: '#ffffff'
            });

            // Store button
            const storeBtn = this.add.text(columnX + 250, yPos, 'STORE', {
                fontSize: '12px',
                color: '#ffffff',
                backgroundColor: '#006600',
                padding: { x: 8, y: 3 }
            }).setInteractive({ useHandCursor: true });

            storeBtn.on('pointerover', () => storeBtn.setBackgroundColor('#008800'));
            storeBtn.on('pointerout', () => storeBtn.setBackgroundColor('#006600'));
            storeBtn.on('pointerdown', () => this.storeItem(item.id, 1));

            // Store All button if quantity > 1
            if (item.quantity > 1) {
                const storeAllBtn = this.add.text(columnX + 320, yPos, 'ALL', {
                    fontSize: '12px',
                    color: '#ffffff',
                    backgroundColor: '#005500',
                    padding: { x: 5, y: 3 }
                }).setInteractive({ useHandCursor: true });

                storeAllBtn.on('pointerover', () => storeAllBtn.setBackgroundColor('#007700'));
                storeAllBtn.on('pointerout', () => storeAllBtn.setBackgroundColor('#005500'));
                storeAllBtn.on('pointerdown', () => this.storeItem(item.id, item.quantity));
            }

            yPos += 25;
        });
    }

    createStorageColumn() {
        const columnX = 475;

        // Header
        this.add.text(columnX, 120, '🏠 SHELTER STORAGE', {
            fontSize: '18px',
            fontFamily: 'Arial Bold',
            color: '#4a8a4a'
        });

        // List area background
        const listBg = this.add.rectangle(columnX, 300, 300, 350, 0x1a1a1a)
            .setOrigin(0, 0);

        // Get storage items
        const storageItems = this.gameSceneRef.shelterManager.getStorageList();

        if (storageItems.length === 0) {
            this.add.text(columnX + 150, 350, 'Storage is empty', {
                fontSize: '14px',
                color: '#666666'
            }).setOrigin(0.5);
            return;
        }

        let yPos = 160;
        storageItems.forEach(item => {
            if (yPos > 480) return; // Prevent overflow

            // Item text
            const itemText = this.add.text(columnX, yPos,
                `${item.name} x${item.quantity}`, {
                fontSize: '14px',
                color: '#ffffff'
            });

            // Retrieve button
            const retrieveBtn = this.add.text(columnX + 220, yPos, 'RETRIEVE', {
                fontSize: '12px',
                color: '#ffffff',
                backgroundColor: '#8B4500',
                padding: { x: 8, y: 3 }
            }).setInteractive({ useHandCursor: true });

            retrieveBtn.on('pointerover', () => retrieveBtn.setBackgroundColor('#aa5500'));
            retrieveBtn.on('pointerout', () => retrieveBtn.setBackgroundColor('#8B4500'));
            retrieveBtn.on('pointerdown', () => this.retrieveItem(item.id, 1));

            // Retrieve All button if quantity > 1
            if (item.quantity > 1) {
                const retrieveAllBtn = this.add.text(columnX + 310, yPos, 'ALL', {
                    fontSize: '12px',
                    color: '#ffffff',
                    backgroundColor: '#6a3400',
                    padding: { x: 5, y: 3 }
                }).setInteractive({ useHandCursor: true });

                retrieveAllBtn.on('pointerover', () => retrieveAllBtn.setBackgroundColor('#8a4400'));
                retrieveAllBtn.on('pointerout', () => retrieveAllBtn.setBackgroundColor('#6a3400'));
                retrieveAllBtn.on('pointerdown', () => this.retrieveItem(item.id, item.quantity));
            }

            yPos += 25;
        });
    }

    storeItem(itemId, quantity) {
        const result = this.gameSceneRef.shelterManager.storeItem(itemId, quantity);

        if (result.success) {
            this.gameSceneRef.addMessage(result.message, '#00ff00');
            this.gameSceneRef.updateUI();
            // Refresh scene
            this.scene.restart();
        } else {
            this.gameSceneRef.addMessage(result.message, '#ff0000');
        }
    }

    retrieveItem(itemId, quantity) {
        const result = this.gameSceneRef.shelterManager.retrieveItem(itemId, quantity);

        if (result.success) {
            this.gameSceneRef.addMessage(result.message, '#00ff00');
            this.gameSceneRef.updateUI();
            // Refresh scene
            this.scene.restart();
        } else {
            this.gameSceneRef.addMessage(result.message, '#ff0000');
        }
    }
}
