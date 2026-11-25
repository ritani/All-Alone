/**
 * The ALL ALONE icon set. Drawn on a 24 x 24 grid, stroke only.
 * Each icon is a single SVG path; render it with <Icon name="..." />.
 * Rules: straight lines and true arcs, square caps, mitre joins, no fills.
 */

/** Full circle as path data (two half arcs). */
function ring(cx: number, cy: number, r: number): string {
  return `M${cx - r} ${cy}A${r} ${r} 0 1 0 ${cx + r} ${cy}A${r} ${r} 0 1 0 ${cx - r} ${cy}Z`;
}

export const ICON_PATHS = {
  // Places and stations
  house: 'M3 11L12 3.5L21 11M5 9.5V20.5H19V9.5M10 20.5V14.5H14V20.5',
  map: 'M3 6L9 4L15 6L21 4V18L15 20L9 18L3 20ZM9 4V18M15 6V20',
  pack: 'M6 7H18V21H6ZM9 7V4H15V7M6 11H18M11 11V14H13V11',
  journal: 'M5 3H18V21H5ZM8 3V21M11 8H15M11 11H15',
  stove: 'M4 9H20V19.5H4ZM8 12H16V16.5H8ZM14 9V3H17V9M5.5 19.5V21.5M18.5 19.5V21.5',
  cot: 'M3 13H21V15.5H3ZM4 13V10H9V13M5 15.5L9 20.5M9 15.5L5 20.5M15 15.5L19 20.5M19 15.5L15 20.5',
  workbench: 'M2 9H22V11H2ZM4 11V21M20 11V21M4 17H20M6 9V6H10V9M14 9L17 4',
  barrel: 'M7 3H17L18.5 12L17 21H7L5.5 12ZM6.3 7.5H17.7M6.3 16.5H17.7',
  garden:
    'M3 17H21V21H3ZM8 17V10M8 13L5 10M8 12L11 9M16 17V12M16 14.5L13 12.5M16 13.5L19 10.5',
  barricade: 'M5 3V21M19 3V21M3 6H21V9H3ZM3 15H21V18H3ZM6.5 9L17.5 15',
  radio: `M3 9H21V20H3ZM15 9L20 3M14 13H18M14 16H18${ring(8.5, 14.5, 3)}`,
  medicine: 'M9 3H15V7H9ZM8 7H16V21H8ZM12 11V17M9 14H15',
  store: 'M4 3V21M20 3V21M4 9H20M4 15H20M7 9V6H10V9M13 15V12H17V15M7 21V18H11V21',
  ferry: 'M2 15H22L19 19.5H5ZM7 15V10H14V15M16 15V7H18V15M2 22H6M9 22H15M18 22H22',

  // Vitals and state
  health: 'M2 12H7L9 7L12 18L15 4L17 12H22',
  food: 'M7 3V21M5 3V8H9V3M16 21V3L19 9V13H16',
  water: 'M12 3L17.2 12A6 6 0 1 1 6.8 12Z',
  rest: 'M11.05 3.05A9 9 0 1 0 20.95 12.95A7 7 0 0 1 11.05 3.05Z',
  nerve: `M2 12Q12 3 22 12Q12 21 2 12Z${ring(12, 12, 3)}`,
  warmth: 'M10 3H14V13.5A4 4 0 1 1 10 13.5ZM12 7V17M16.5 5H18.5M16.5 8H18.5M16.5 11H18.5',
  noise: 'M3 9H7L12 5V19L7 15H3ZM15 9L17 12L15 15M18 6L22 12L18 18',
  clock: `${ring(12, 12, 9)}M12 7V12H16`,
  lock: 'M8 10V5H16V10M5 10H19V21H5ZM12 14V17',
  light: 'M9 4H15L16 6H8ZM8 6V17H16V6M7 17H17V20H7ZM12 9L13.5 11.5L12 14L10.5 11.5ZM10 4V2H14V4',
  stilled: `${ring(12, 4.5, 2)}M12 6.5V15M12 8L9.5 14M12 8L14.5 14M12 15L10 21M12 15L14 21`,

  // Interface
  back: 'M15 5L8 12L15 19',
  close: 'M5 5L19 19M19 5L5 19',
} as const satisfies Record<string, string>;

export type IconName = keyof typeof ICON_PATHS;

export const ICON_NAMES = Object.keys(ICON_PATHS) as IconName[];
