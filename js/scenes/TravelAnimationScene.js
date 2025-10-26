class TravelAnimationScene extends Phaser.Scene {
    constructor() {
        super({ key: 'TravelAnimationScene' });
    }

    init(data) {
        this.gameSceneRef = data.gameScene;
        this.targetLocation = data.targetLocation;
        this.travelResult = data.travelResult;
        this.onComplete = data.onComplete;
        this.returning = data.returning || false;
    }

    create() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Dark background
        this.add.rectangle(0, 0, 800, 600, 0x0a0a0a).setOrigin(0, 0);

        // Title
        const direction = this.returning ? 'Returning to Shelter' : `Traveling to ${this.targetLocation.name}`;
        this.titleText = this.add.text(centerX, 80, direction, {
            fontSize: '28px',
            fontFamily: 'Arial Black',
            color: '#ffaa00'
        }).setOrigin(0.5);

        // Distance and time info
        const travelInfo = `Distance: ${this.targetLocation.distance}km  |  Time: ${this.targetLocation.travelTime}h`;
        this.add.text(centerX, 120, travelInfo, {
            fontSize: '16px',
            color: '#888888'
        }).setOrigin(0.5);

        // Create animated road
        this.createRoadAnimation();

        // Create character walking animation
        this.createCharacterAnimation();

        // Progress bar
        this.createProgressBar();

        // Status messages
        this.statusText = this.add.text(centerX, 450, '', {
            fontSize: '18px',
            color: '#ffffff',
            align: 'center'
        }).setOrigin(0.5);

        // Encounter warning
        this.encounterText = this.add.text(centerX, 490, '', {
            fontSize: '16px',
            color: '#ff0000',
            align: 'center'
        }).setOrigin(0.5).setAlpha(0);

        // Start travel sequence
        this.startTravelSequence();
    }

    createRoadAnimation() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Road background
        this.roadBg = this.add.rectangle(centerX, centerY, 600, 150, 0x333333);

        // Road lines (animated)
        this.roadLines = [];
        for (let i = 0; i < 8; i++) {
            const line = this.add.rectangle(
                50 + i * 90,
                centerY,
                50,
                5,
                0xffff00
            );
            this.roadLines.push(line);
        }

        // Animate road lines moving
        this.tweens.add({
            targets: this.roadLines,
            x: '+=90',
            duration: 800,
            repeat: -1,
            onRepeat: () => {
                this.roadLines.forEach(line => {
                    if (line.x > 800) {
                        line.x = -50;
                    }
                });
            }
        });

        // Background buildings/scenery
        this.createScenery();
    }

    createScenery() {
        // Create simple building silhouettes
        for (let i = 0; i < 5; i++) {
            const height = 40 + Math.random() * 60;
            const x = 100 + i * 150;
            const building = this.add.rectangle(x, 200 - height/2, 80, height, 0x1a1a1a);
            building.setAlpha(0.5);

            // Animate buildings moving (parallax effect)
            this.tweens.add({
                targets: building,
                x: '-=900',
                duration: 3000,
                repeat: -1,
                onRepeat: () => {
                    building.x = 850;
                }
            });
        }

        // Trees
        for (let i = 0; i < 6; i++) {
            const x = 50 + i * 130;
            const tree = this.add.text(x, 380, '🌲', {
                fontSize: '40px'
            }).setAlpha(0.6);

            this.tweens.add({
                targets: tree,
                x: '-=900',
                duration: 2000,
                repeat: -1,
                onRepeat: () => {
                    tree.x = 850;
                }
            });
        }
    }

    createCharacterAnimation() {
        const centerX = this.cameras.main.width / 2;
        const centerY = this.cameras.main.height / 2;

        // Character sprite (simplified)
        this.character = this.add.text(centerX, centerY + 20, '🚶', {
            fontSize: '48px'
        });

        // Walking animation (bobbing)
        this.tweens.add({
            targets: this.character,
            y: centerY + 15,
            duration: 400,
            yoyo: true,
            repeat: -1,
            ease: 'Sine.easeInOut'
        });

        // Character rotation for walking effect
        this.tweens.add({
            targets: this.character,
            angle: { from: -5, to: 5 },
            duration: 400,
            yoyo: true,
            repeat: -1,
            ease: 'Sine.easeInOut'
        });
    }

    createProgressBar() {
        const centerX = this.cameras.main.width / 2;

        // Progress bar background
        this.progressBg = this.add.rectangle(centerX, 520, 500, 30, 0x333333);
        this.progressBg.setStrokeStyle(2, 0x666666);

        // Progress bar fill
        this.progressBar = this.add.rectangle(centerX - 248, 520, 4, 26, 0x00ff00);
        this.progressBar.setOrigin(0, 0.5);

        // Progress text
        this.progressText = this.add.text(centerX, 520, '0%', {
            fontSize: '16px',
            color: '#ffffff'
        }).setOrigin(0.5);
    }

    startTravelSequence() {
        const travelDuration = this.targetLocation.travelTime * 2000; // 2 seconds per hour
        let progress = 0;

        // Update progress
        const progressTimer = this.time.addEvent({
            delay: 50,
            repeat: travelDuration / 50,
            callback: () => {
                progress += 50 / travelDuration;
                this.updateProgress(progress);
            }
        });

        // Random events during travel
        this.scheduleRandomEvents(travelDuration);

        // Complete travel
        this.time.delayedCall(travelDuration, () => {
            this.completeTravelSequence();
        });
    }

    scheduleRandomEvents(duration) {
        const eventCount = Math.floor(Math.random() * 3) + 2;

        for (let i = 0; i < eventCount; i++) {
            const eventTime = (duration / eventCount) * (i + 0.5);
            this.time.delayedCall(eventTime, () => {
                this.showRandomEvent();
            });
        }

        // Check for encounter at 80% mark
        if (this.travelResult && this.travelResult.travelEncounter && this.travelResult.travelEncounter.encountered) {
            this.time.delayedCall(duration * 0.8, () => {
                this.showEncounterWarning();
            });
        }
    }

    showRandomEvent() {
        const events = [
            'Passing abandoned cars...',
            'Avoiding debris on the road...',
            'Checking surroundings...',
            'Moving cautiously through area...',
            'Spotted movement in distance...',
            'Taking a quick breather...',
            'Clouds gathering overhead...',
            'Wind picking up...'
        ];

        const event = events[Math.floor(Math.random() * events.length)];
        this.statusText.setText(event);

        this.tweens.add({
            targets: this.statusText,
            alpha: { from: 1, to: 0 },
            duration: 2000,
            delay: 1000
        });
    }

    showEncounterWarning() {
        this.encounterText.setText('⚠️ DANGER AHEAD! ⚠️');
        this.encounterText.setAlpha(1);

        // Flash animation
        this.tweens.add({
            targets: this.encounterText,
            alpha: { from: 1, to: 0.3 },
            duration: 300,
            yoyo: true,
            repeat: 3
        });

        // Shake camera
        this.cameras.main.shake(500, 0.01);

        // Change character to alert
        this.character.setText('😰');
        this.statusText.setText('Something is approaching...');
        this.statusText.setAlpha(1);
        this.statusText.setColor('#ff0000');
    }

    updateProgress(progress) {
        progress = Math.min(progress, 1);
        this.progressBar.width = progress * 496;
        this.progressText.setText(`${Math.floor(progress * 100)}%`);

        // Change progress bar color based on progress
        if (progress < 0.3) {
            this.progressBar.setFillStyle(0x00ff00);
        } else if (progress < 0.7) {
            this.progressBar.setFillStyle(0xffaa00);
        } else {
            this.progressBar.setFillStyle(0xff0000);
        }
    }

    completeTravelSequence() {
        this.statusText.setText(this.returning ? 'Arrived at Shelter!' : 'Destination Reached!');
        this.statusText.setColor('#00ff00');
        this.statusText.setAlpha(1);

        // Stop animations
        this.tweens.killAll();

        // Character celebration
        this.tweens.add({
            targets: this.character,
            scale: 1.3,
            duration: 200,
            yoyo: true,
            repeat: 1
        });

        // Fade out and continue
        this.time.delayedCall(1500, () => {
            this.cameras.main.fadeOut(500, 0, 0, 0);
            this.cameras.main.once('camerafadeoutcomplete', () => {
                this.scene.stop();
                if (this.onComplete) {
                    this.onComplete();
                }
            });
        });
    }
}
