// Expedition HUD Component - Shows consistent inventory info across all expedition scenes
class ExpeditionHUD {
    constructor(scene, expeditionManager) {
        this.scene = scene;
        this.expeditionManager = expeditionManager;
        this.container = [];
    }

    create() {
        const width = this.scene.cameras.main.width;

        // Create HUD panel at top
        const panel = this.scene.add.rectangle(0, 0, width, 60, 0x1a1a1a, 0.95)
            .setOrigin(0, 0)
            .setDepth(1000)
            .setScrollFactor(0);
        this.container.push(panel);

        const border = this.scene.add.rectangle(0, 60, width, 2, 0xffaa00)
            .setOrigin(0, 0)
            .setDepth(1000)
            .setScrollFactor(0);
        this.container.push(border);

        // Backpack icon
        const backpackIcon = this.scene.add.text(15, 15, '🎒', {
            fontSize: '32px'
        }).setDepth(1001).setScrollFactor(0);
        this.container.push(backpackIcon);

        // Weight display
        this.weightText = this.scene.add.text(55, 12, '', {
            fontSize: '16px',
            fontFamily: 'Arial Bold',
            color: '#ffffff'
        }).setDepth(1001).setScrollFactor(0);
        this.container.push(this.weightText);

        // Weight bar
        this.weightBarBg = this.scene.add.rectangle(55, 35, 200, 12, 0x333333)
            .setOrigin(0, 0)
            .setDepth(1001)
            .setScrollFactor(0);
        this.container.push(this.weightBarBg);

        this.weightBar = this.scene.add.rectangle(55, 35, 200, 12, 0x00ff00)
            .setOrigin(0, 0)
            .setDepth(1002)
            .setScrollFactor(0);
        this.container.push(this.weightBar);

        // Item count
        this.itemCountText = this.scene.add.text(270, 20, '', {
            fontSize: '14px',
            fontFamily: 'Arial',
            color: '#cccccc'
        }).setDepth(1001).setScrollFactor(0);
        this.container.push(this.itemCountText);

        // View button
        const viewBtn = this.scene.add.text(width - 120, 20, '📋 VIEW', {
            fontSize: '16px',
            fontFamily: 'Arial Bold',
            color: '#ffffff',
            backgroundColor: '#4a4a4a',
            padding: { x: 15, y: 8 }
        }).setOrigin(0, 0.5)
          .setDepth(1001)
          .setScrollFactor(0)
          .setInteractive({ useHandCursor: true });

        viewBtn.on('pointerover', () => viewBtn.setBackgroundColor('#6a6a6a'));
        viewBtn.on('pointerout', () => viewBtn.setBackgroundColor('#4a4a4a'));
        viewBtn.on('pointerdown', () => this.showInventoryDetail());

        this.container.push(viewBtn);

        this.update();
    }

    update() {
        if (!this.weightText) return;

        const capacity = this.expeditionManager.getExpeditionCapacity();
        const items = this.expeditionManager.getExpeditionItems();

        // Update weight text
        this.weightText.setText(`${capacity.current.toFixed(1)}/${capacity.max}kg`);

        // Update weight bar
        const percentage = capacity.current / capacity.max;
        this.weightBar.width = 200 * percentage;

        // Change color based on capacity
        if (percentage < 0.5) {
            this.weightBar.setFillStyle(0x00ff00);
        } else if (percentage < 0.8) {
            this.weightBar.setFillStyle(0xffaa00);
        } else {
            this.weightBar.setFillStyle(0xff0000);
        }

        // Update item count
        let totalItems = 0;
        items.forEach(item => totalItems += item.quantity);
        this.itemCountText.setText(`${totalItems} items`);
    }

    showInventoryDetail() {
        const items = this.expeditionManager.getExpeditionItems();

        if (items.length === 0) {
            return;
        }

        // Create modal overlay
        const overlay = this.scene.add.rectangle(0, 0, 800, 600, 0x000000, 0.85)
            .setOrigin(0, 0)
            .setDepth(2000)
            .setScrollFactor(0)
            .setInteractive();

        const panel = this.scene.add.rectangle(400, 300, 600, 500, 0x2a2a2a)
            .setDepth(2001)
            .setScrollFactor(0)
            .setStrokeStyle(3, 0xffaa00);

        const title = this.scene.add.text(400, 80, 'EXPEDITION INVENTORY', {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#ffaa00'
        }).setOrigin(0.5).setDepth(2002).setScrollFactor(0);

        const capacity = this.expeditionManager.getExpeditionCapacity();
        const capacityText = this.scene.add.text(400, 120, `Weight: ${capacity.current.toFixed(1)}/${capacity.max}kg`, {
            fontSize: '18px',
            color: '#ffffff'
        }).setOrigin(0.5).setDepth(2002).setScrollFactor(0);

        // List items
        let yPos = 160;
        items.forEach(item => {
            const itemText = this.scene.add.text(150, yPos,
                `${item.name} x${item.quantity} (${(item.weight * item.quantity).toFixed(1)}kg)`, {
                fontSize: '16px',
                color: '#cccccc'
            }).setDepth(2002).setScrollFactor(0);

            yPos += 25;
        });

        // Close button
        const closeBtn = this.scene.add.text(400, 520, 'CLOSE', {
            fontSize: '20px',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5).setDepth(2002).setScrollFactor(0)
          .setInteractive({ useHandCursor: true });

        const modalElements = [overlay, panel, title, capacityText, closeBtn];
        items.forEach((item, index) => {
            const itemText = this.scene.add.text(150, 160 + index * 25,
                `${item.name} x${item.quantity} (${(item.weight * item.quantity).toFixed(1)}kg)`, {
                fontSize: '16px',
                color: '#cccccc'
            }).setDepth(2002).setScrollFactor(0);
            modalElements.push(itemText);
        });

        closeBtn.on('pointerover', () => closeBtn.setBackgroundColor('#aa0000'));
        closeBtn.on('pointerout', () => closeBtn.setBackgroundColor('#8B0000'));
        closeBtn.on('pointerdown', () => {
            modalElements.forEach(el => el.destroy());
        });
    }

    destroy() {
        this.container.forEach(obj => obj.destroy());
        this.container = [];
    }
}
