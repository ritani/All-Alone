class VitalStatsHUD {
    constructor(scene, gameState) {
        this.scene = scene;
        this.gameState = gameState;
        this.elements = [];
    }

    create() {
        const width = this.scene.cameras.main.width;

        // Background bar
        const bg = this.scene.add.rectangle(0, 0, width, 50, 0x1a1a1a, 0.95)
            .setOrigin(0, 0)
            .setDepth(999)
            .setScrollFactor(0);
        this.elements.push(bg);

        // Border
        const border = this.scene.add.rectangle(0, 50, width, 2, 0x8B0000)
            .setOrigin(0, 0)
            .setDepth(999)
            .setScrollFactor(0);
        this.elements.push(border);

        // Stats text (will be updated)
        this.statsText = this.scene.add.text(10, 8, '', {
            fontSize: '14px',
            fontFamily: 'Arial',
            color: '#ffffff'
        }).setDepth(1000).setScrollFactor(0);
        this.elements.push(this.statsText);

        // Time/Day text
        this.timeText = this.scene.add.text(width - 10, 8, '', {
            fontSize: '14px',
            fontFamily: 'Arial Bold',
            color: '#ffaa00'
        }).setDepth(1000).setScrollFactor(0).setOrigin(1, 0);
        this.elements.push(this.timeText);

        // Initial update
        this.update();
    }

    update() {
        if (!this.statsText || !this.gameState) return;

        // Create stat display with icons
        const hp = Math.floor(this.gameState.hp);
        const energy = Math.floor(this.gameState.energy);
        const mood = Math.floor(this.gameState.mood);
        const hunger = Math.floor(this.gameState.hunger);
        const thirst = Math.floor(this.gameState.thirst);

        this.statsText.setText(
            `❤️ ${hp}  ⚡ ${energy}  😊 ${mood}  🍖 ${hunger}  💧 ${thirst}`
        );

        // Update time display
        if (this.timeText) {
            const timeOfDay = this.gameState.hour < 12 ? '🌅' :
                             this.gameState.hour < 18 ? '☀️' :
                             this.gameState.hour < 22 ? '🌆' : '🌙';
            this.timeText.setText(`Day ${this.gameState.day} | ${this.gameState.hour}:00 ${timeOfDay}`);
        }
    }

    destroy() {
        this.elements.forEach(element => {
            if (element && element.destroy) {
                element.destroy();
            }
        });
        this.elements = [];
        this.statsText = null;
        this.timeText = null;
    }
}
