// Initialize the game
window.addEventListener('load', () => {
    // Create game configuration with scenes
    const config = {
        ...gameConfig,
        scene: [SplashScene, MenuScene, GameScene, MapScene, ExpeditionPlanScene, AreaExplorationScene, LootSelectionScene, TravelAnimationScene, ShelterStorageScene]
    };

    // Create game instance
    const game = new Phaser.Game(config);

    // Prevent accidental page refresh
    window.addEventListener('beforeunload', (e) => {
        // Don't show warning if on menu screen
        if (game.scene.getScene('GameScene') && game.scene.getScene('GameScene').scene.isActive()) {
            e.preventDefault();
            e.returnValue = '';
        }
    });

    // Handle visibility change to auto-pause
    document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
            game.scene.getScene('GameScene')?.saveGame();
        }
    });

    // Disable context menu on canvas
    game.canvas.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        return false;
    });

    // Mobile-specific adjustments
    if (/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)) {
        // Disable double-tap zoom
        document.addEventListener('touchstart', (e) => {
            if (e.touches.length > 1) {
                e.preventDefault();
            }
        }, { passive: false });

        let lastTouchEnd = 0;
        document.addEventListener('touchend', (e) => {
            const now = Date.now();
            if (now - lastTouchEnd <= 300) {
                e.preventDefault();
            }
            lastTouchEnd = now;
        }, false);

        // Lock orientation to portrait on mobile (optional)
        if (screen.orientation && screen.orientation.lock) {
            screen.orientation.lock('portrait').catch(err => {
                console.log('Orientation lock not supported:', err);
            });
        }
    }

    console.log('Buried City loaded successfully!');
});
