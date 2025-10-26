class MenuScene extends Phaser.Scene {
    constructor() {
        super({ key: 'MenuScene' });
    }

    create() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Background
        this.add.rectangle(0, 0, this.cameras.main.width, this.cameras.main.height, 0x2d2d2d)
            .setOrigin(0, 0);

        // Create dark vignette effect
        const vignette = this.add.graphics();
        vignette.fillGradientStyle(0x000000, 0x000000, 0x000000, 0x000000, 0, 0.5, 0.5, 0);
        vignette.fillRect(0, 0, this.cameras.main.width, this.cameras.main.height);

        // Title
        this.add.text(centerX, 100, 'BURIED CITY', {
            fontSize: '56px',
            fontFamily: 'Arial Black',
            color: '#8B0000',
            stroke: '#000000',
            strokeThickness: 6
        }).setOrigin(0.5);

        // Subtitle with atmosphere
        this.add.text(centerX, 160, 'Day 1 of the Outbreak', {
            fontSize: '20px',
            fontFamily: 'Arial',
            color: '#999999',
            fontStyle: 'italic'
        }).setOrigin(0.5);

        // Menu buttons
        const buttonStyle = {
            fontSize: '28px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: '#4a4a4a',
            padding: { x: 40, y: 15 },
            stroke: '#000000',
            strokeThickness: 3
        };

        const buttonHoverStyle = {
            backgroundColor: '#6a6a6a'
        };

        // New Game button
        const newGameBtn = this.createButton(centerX, centerY - 30, 'NEW GAME', buttonStyle);
        newGameBtn.on('pointerdown', () => {
            this.cameras.main.fadeOut(300, 0, 0, 0);
            this.cameras.main.once('camerafadeoutcomplete', () => {
                this.scene.start('GameScene', { newGame: true });
            });
        });

        // Continue button (disabled if no save)
        const continueBtn = this.createButton(centerX, centerY + 40, 'CONTINUE', buttonStyle);
        const hasSave = this.checkSaveGame();
        if (!hasSave) {
            continueBtn.setAlpha(0.5);
            continueBtn.disableInteractive();
        } else {
            continueBtn.on('pointerdown', () => {
                this.cameras.main.fadeOut(300, 0, 0, 0);
                this.cameras.main.once('camerafadeoutcomplete', () => {
                    this.scene.start('GameScene', { newGame: false });
                });
            });
        }

        // How to Play button
        const howToPlayBtn = this.createButton(centerX, centerY + 110, 'HOW TO PLAY', buttonStyle);
        howToPlayBtn.on('pointerdown', () => {
            this.showHowToPlay();
        });

        // Game description
        this.add.text(centerX, this.cameras.main.height - 80,
            'Scavenge, craft, fight zombies, and survive in a post-apocalyptic city.\nManage your health, energy, and mood while exploring dangerous locations.', {
            fontSize: '16px',
            fontFamily: 'Arial',
            color: '#888888',
            align: 'center'
        }).setOrigin(0.5);

        // Credits
        this.add.text(centerX, this.cameras.main.height - 20,
            'Created with Phaser 3', {
            fontSize: '12px',
            fontFamily: 'Arial',
            color: '#666666'
        }).setOrigin(0.5);

        // Fade in
        this.cameras.main.fadeIn(500, 0, 0, 0);
    }

    createButton(x, y, text, style) {
        const button = this.add.text(x, y, text, style)
            .setOrigin(0.5)
            .setInteractive({ useHandCursor: true });

        button.on('pointerover', () => {
            button.setBackgroundColor('#6a6a6a');
            button.setScale(1.05);
        });

        button.on('pointerout', () => {
            button.setBackgroundColor('#4a4a4a');
            button.setScale(1);
        });

        return button;
    }

    checkSaveGame() {
        try {
            const save = localStorage.getItem('buriedCitySave');
            return save !== null;
        } catch (e) {
            return false;
        }
    }

    showHowToPlay() {
        // Create modal overlay
        const overlay = this.add.rectangle(0, 0, this.cameras.main.width, this.cameras.main.height, 0x000000, 0.8)
            .setOrigin(0, 0)
            .setInteractive();

        const panel = this.add.rectangle(this.cameras.main.width / 2, this.cameras.main.height / 2,
            700, 500, 0x3a3a3a)
            .setStrokeStyle(3, 0x8B0000);

        const title = this.add.text(this.cameras.main.width / 2, 120, 'HOW TO PLAY', {
            fontSize: '32px',
            fontFamily: 'Arial Black',
            color: '#ffffff'
        }).setOrigin(0.5);

        const instructions = `
🧟 SURVIVE THE ZOMBIE APOCALYPSE 🧟

📊 STATS TO MANAGE:
• HP: Your health. Reaches 0 = Game Over
• Energy: Decreases with actions. Rest to recover
• Mood: Stay positive to survive. Eat chocolate!
• Hunger: Eat food before you starve

🗺️ EXPLORATION:
• Visit different locations to scavenge supplies
• Higher danger = better loot but more zombies
• Each action costs time and energy

⚔️ COMBAT:
• Combat is automatic when exploring
• Better weapons = more damage
• Heal up before risky expeditions

🔨 CRAFTING:
• Combine materials to create useful items
• Build shelter upgrades for safety
• Craft better weapons and medicine

💡 TIPS:
• Rest at your shelter to recover
• Balance risk vs. reward when exploring
• Stock up on food and medicine
• Time is precious - plan carefully
        `;

        const instructionText = this.add.text(this.cameras.main.width / 2, 340, instructions, {
            fontSize: '15px',
            fontFamily: 'Arial',
            color: '#cccccc',
            align: 'center',
            lineSpacing: 5
        }).setOrigin(0.5);

        const closeBtn = this.add.text(this.cameras.main.width / 2, 540, 'CLOSE', {
            fontSize: '24px',
            fontFamily: 'Arial',
            color: '#ffffff',
            backgroundColor: '#8B0000',
            padding: { x: 30, y: 10 }
        }).setOrigin(0.5).setInteractive({ useHandCursor: true });

        closeBtn.on('pointerover', () => closeBtn.setBackgroundColor('#aa0000'));
        closeBtn.on('pointerout', () => closeBtn.setBackgroundColor('#8B0000'));
        closeBtn.on('pointerdown', () => {
            overlay.destroy();
            panel.destroy();
            title.destroy();
            instructionText.destroy();
            closeBtn.destroy();
        });
    }
}
