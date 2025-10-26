# Buried City - Zombie Survival Game

A post-apocalyptic zombie survival game inspired by Buried Town. Manage your resources, explore dangerous locations, fight zombies, and try to survive as long as possible!

## Features

### Core Gameplay
- **Survival Stats Management**: Keep track of HP, Energy, Mood, and Hunger
- **Time Progression**: Dynamic day/night cycle with time-based stat decay
- **Map & Travel System**: Navigate through the city with realistic travel times
- **Expedition Planning**: Limited carrying capacity (20kg) - choose what to bring back!
- **Random Encounters**: Zombies may ambush you during travel
- **Location Discovery**: Find and unlock new locations while exploring
- **Resource Scavenging**: Explore 7 different locations with varying danger levels
- **Automatic Combat**: Fight zombies with equipped weapons
- **Crafting System**: Combine materials to create useful items
- **Inventory Management**: Collect and use items strategically with weight limits
- **Save System**: Auto-save and manual save functionality

### Locations to Explore
- **Your Shelter**: Safe haven where you can rest and recover (always unlocked)
- **Convenience Store**: Food and basic supplies (0.5km, 1h travel)
- **Pharmacy**: Medical supplies and medicine (1km, 2h travel) 🔒 *Discover to unlock*
- **Hardware Store**: Tools, weapons, and crafting materials (0.6km, 1h travel)
- **Residential Area**: Abandoned homes with mixed loot (1.2km, 2h travel) 🔒 *Discover to unlock*
- **Warehouse**: High-risk, high-reward location (2km, 3h travel) 🔒 *Discover to unlock*
- **Abandoned Hospital**: Extremely dangerous, rich medical supplies (2.5km, 3h travel) 🔒 *Discover to unlock*

### Items & Resources
- **Food**: Canned food, chocolate, MREs to reduce hunger
- **Medicine**: Bandages, first aid kits, pills to restore HP
- **Weapons**: Knives, axes, pipes, baseball bats for combat
- **Materials**: Wood, metal, cloth, rope for crafting

### Game Mechanics
- **Travel System**: Each location has travel time and energy cost
- **Weight Limits**: Can only carry 20kg per expedition - items have different weights
- **Random Encounters**: 15% base chance of zombie ambush during travel (scales with distance)
- **Location Discovery**: 15% chance to discover new location while traveling
- **Expedition Flow**:
  1. Select location on map
  2. Travel there (costs time & energy, risk of encounter)
  3. Explore/Scavenge (costs time, combat likely)
  4. Choose what to carry back (weight limit!)
  5. Travel home (costs time & energy, risk of encounter again)
- Stats decay over time automatically
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
1. **Plan Expeditions Carefully**: Account for travel time BOTH WAYS plus exploration time
2. **Energy Management**: Make sure you have enough energy to return home safely
3. **Weight Matters**: Lighter items (food, cloth, pills) = more items you can carry
4. **Random Encounters**: You can be attacked during travel - keep HP high before expeditions
5. **Location Discovery**: Travel often to unlock new locations (15% chance per trip)
6. **Prioritize Loot**: Medicine and food are lighter than materials and weapons
7. **Keep hunger below 80**: Or you'll lose HP over time
8. **Don't get stranded**: Always have enough energy to travel back to shelter
9. **Rest regularly**: Your shelter is the only place to recover
10. **Time is precious**: Every expedition takes 4-10+ hours total (travel + exploration + return)

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
