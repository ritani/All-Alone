class SplashScene extends Phaser.Scene {
    constructor() {
        super({ key: 'SplashScene' });
    }

    create() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Background
        this.add.rectangle(0, 0, this.cameras.main.width, this.cameras.main.height, 0x1a1a1a)
            .setOrigin(0, 0);

        // Title with dramatic effect
        const title = this.add.text(centerX, centerY - 80, 'BURIED CITY', {
            fontSize: '64px',
            fontFamily: 'Arial Black',
            color: '#8B0000',
            stroke: '#000000',
            strokeThickness: 8,
            shadow: {
                offsetX: 3,
                offsetY: 3,
                color: '#000',
                blur: 5,
                fill: true
            }
        }).setOrigin(0.5).setAlpha(0);

        // Subtitle
        const subtitle = this.add.text(centerX, centerY + 20, 'Survive the Apocalypse', {
            fontSize: '24px',
            fontFamily: 'Arial',
            color: '#cccccc',
            stroke: '#000000',
            strokeThickness: 4
        }).setOrigin(0.5).setAlpha(0);

        // Zombie emoji/icon
        const zombieIcon = this.add.text(centerX, centerY - 150, '🧟', {
            fontSize: '80px'
        }).setOrigin(0.5).setAlpha(0);

        // Blood drip effect
        const bloodDrip = this.add.graphics();
        bloodDrip.fillStyle(0x8B0000, 0.7);
        bloodDrip.fillRect(centerX - 100, 50, 200, 5);
        bloodDrip.setAlpha(0);

        // Loading text
        const loadingText = this.add.text(centerX, centerY + 100, 'Loading...', {
            fontSize: '20px',
            fontFamily: 'Arial',
            color: '#666666'
        }).setOrigin(0.5).setAlpha(0);

        // Animate splash screen
        this.tweens.add({
            targets: zombieIcon,
            alpha: 1,
            duration: 500,
            ease: 'Power2'
        });

        this.tweens.add({
            targets: title,
            alpha: 1,
            duration: 800,
            delay: 300,
            ease: 'Power2'
        });

        this.tweens.add({
            targets: bloodDrip,
            alpha: 1,
            duration: 600,
            delay: 500,
            ease: 'Power2'
        });

        this.tweens.add({
            targets: subtitle,
            alpha: 1,
            duration: 800,
            delay: 700,
            ease: 'Power2'
        });

        this.tweens.add({
            targets: loadingText,
            alpha: { from: 0, to: 1 },
            duration: 1000,
            delay: 1000,
            yoyo: true,
            repeat: -1,
            ease: 'Sine.easeInOut'
        });

        // Transition to menu after splash
        this.time.delayedCall(3500, () => {
            this.cameras.main.fadeOut(500, 0, 0, 0);
            this.cameras.main.once('camerafadeoutcomplete', () => {
                this.scene.start('MenuScene');
            });
        });

        // Skip splash on click/tap
        this.input.once('pointerdown', () => {
            this.scene.start('MenuScene');
        });

        // Warning message
        const warning = this.add.text(centerX, this.cameras.main.height - 40,
            'Tap anywhere to skip', {
            fontSize: '14px',
            fontFamily: 'Arial',
            color: '#888888'
        }).setOrigin(0.5).setAlpha(0);

        this.tweens.add({
            targets: warning,
            alpha: 1,
            duration: 800,
            delay: 1500
        });
    }
}
