# Buried City - Zombie Survival Game

A post-apocalyptic zombie survival game inspired by Buried Town. Manage your resources, explore dangerous locations, fight zombies, and try to survive as long as possible!

## Features

### Core Gameplay
- **Survival Stats Management**: Keep track of HP, Energy, Mood, and Hunger
- **Time Progression**: Dynamic day/night cycle with time-based stat decay
- **Resource Scavenging**: Explore 6 different locations with varying danger levels
- **Automatic Combat**: Fight zombies with equipped weapons
- **Crafting System**: Combine materials to create useful items
- **Inventory Management**: Collect and use items strategically
- **Save System**: Auto-save and manual save functionality

### Locations to Explore
- **Your Shelter**: Safe haven where you can rest and recover
- **Convenience Store**: Food and basic supplies
- **Pharmacy**: Medical supplies and medicine
- **Hardware Store**: Tools, weapons, and crafting materials
- **Residential Area**: Abandoned homes with mixed loot
- **Warehouse**: High-risk, high-reward location

### Items & Resources
- **Food**: Canned food, chocolate, MREs to reduce hunger
- **Medicine**: Bandages, first aid kits, pills to restore HP
- **Weapons**: Knives, axes, pipes, baseball bats for combat
- **Materials**: Wood, metal, cloth, rope for crafting

### Game Mechanics
- Each action costs time and energy
- Stats decay over time
- Higher danger locations have better loot but more zombies
- Better weapons deal more damage in combat
- Rest at your shelter to recover energy and HP
- Craft items to improve your survival chances

## How to Play

### Getting Started
1. Open `index.html` in a web browser
2. Click "NEW GAME" to start
3. Read the "HOW TO PLAY" guide for detailed instructions

### Controls
- **Click/Tap** buttons to perform actions
- **Explore**: Visit locations for thorough scavenging (2 hours, high loot)
- **Scavenge**: Quick search of locations (1 hour, less loot)
- **Rest**: Recover energy and HP at your shelter (4 hours)
- **Craft**: Create items from materials
- **Use Item**: Consume food or medicine
- **Menu**: Save game or return to main menu

### Tips for Survival
1. Keep your hunger below 80 to avoid HP loss
2. Don't let energy drop too low or you'll be exhausted
3. Stock up on food and medicine before exploring dangerous areas
4. Better weapons make combat much safer
5. Balance risk vs. reward when choosing locations
6. Rest regularly at your shelter
7. Time is precious - plan your actions carefully

## Technical Details

### Built With
- **Phaser 3**: HTML5 game framework
- **JavaScript**: ES6+ features
- **LocalStorage**: For save game functionality

### Browser Compatibility
- Modern browsers (Chrome, Firefox, Safari, Edge)
- Mobile browsers (iOS Safari, Chrome Mobile)
- Requires JavaScript enabled

### Mobile Support
- Fully responsive design
- Touch controls optimized
- Works on phones and tablets
- Portrait orientation recommended

## Development

### File Structure
```
buried-city/
├── index.html                 # Main HTML file
├── js/
│   ├── config.js             # Game configuration and constants
│   ├── main.js               # Game initialization
│   ├── scenes/
│   │   ├── SplashScene.js    # Splash screen
│   │   ├── MenuScene.js      # Main menu
│   │   └── GameScene.js      # Main game scene
│   └── managers/
│       ├── InventoryManager.js   # Inventory system
│       ├── CraftingManager.js    # Crafting system
│       └── LocationManager.js    # Location/exploration system
└── README.md
```

### Running Locally
Simply open `index.html` in your browser, or use a local web server:

```bash
# Python 3
python -m http.server 8000

# Node.js (if you have http-server installed)
npx http-server

# PHP
php -S localhost:8000
```

Then navigate to `http://localhost:8000`

### Customization
You can easily customize the game by editing `js/config.js`:
- Add new items to the `Items` object
- Create new crafting recipes in `Recipes`
- Add more locations in `Locations`
- Adjust game difficulty via `GameConstants`

## License

This is an original game inspired by Buried Town/Buried City. Created for educational and entertainment purposes.

## Credits

- Game Framework: Phaser 3
- Inspired by: Buried Town by Dice7/Locojoy
- Created with: Claude Code

---

**Survive the apocalypse. How long can you last?**
