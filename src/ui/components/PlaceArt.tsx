/**
 * Place header illustrations: one band per place in the valley, drawn as it
 * stands now. Most have somebody in them who should not be standing so still.
 * Stroke only, hairline (non-scaling), square caps, mitre joins.
 */

type Scene = {
  label: string;
  line: string[];
  dim: string[];
  faint: string[];
};

/** A closed rectangle. */
const box = (x: number, y: number, w: number, h: number): string =>
  `M${x} ${y}H${x + w}V${y + h}H${x}Z`;

/** A circle as two true arcs. */
const ring = (cx: number, cy: number, r: number): string =>
  `M${cx} ${cy + r}A${r} ${r} 0 1 0 ${cx} ${cy - r}A${r} ${r} 0 1 0 ${cx} ${cy + r}`;

/** An opening with a round head: jambs up from the sill, an arc across. */
const arched = (x: number, y: number, w: number, h: number): string =>
  `M${x} ${y + h}V${y + w / 2}A${w / 2} ${w / 2} 0 0 1 ${x + w} ${y + w / 2}V${y + h}`;

/** One of the Stilled: a figure standing on ground level g, arms at its sides. */
const still = (x: number, g: number): string =>
  `${ring(x, g - 15, 1.5)}M${x} ${g - 13.5}V${g - 6}M${x} ${g - 6}L${x - 1} ${g}M${x} ${g - 6}L${x + 1} ${g}`;

/** Repeat a drawing across columns and rows. */
const grid = (
  cols: number,
  rows: number,
  dx: number,
  dy: number,
  draw: (x: number, y: number) => string,
): string => {
  let d = '';
  for (let r = 0; r < rows; r++)
    for (let c = 0; c < cols; c++) d += draw(c * dx, r * dy);
  return d;
};

/** Flowing water: dashed horizontals below the bank, from x0 to the right edge. */
const water = (x0: number, y0: number): string =>
  grid(
    1,
    3,
    0,
    7,
    (_, y) =>
      `M${x0 + (y % 14)} ${y0 + y}H${x0 + 40 + (y % 14)}M${x0 + 58} ${y0 + y}H${x0 + 120}M${x0 + 140 - y} ${y0 + y}H320`,
  );

const SCENES: Record<string, Scene> = {
  weir: {
    label:
      'A weir across the river, a keeper’s hut on the bank, and stone steps where someone waits.',
    line: [
      // Keeper's hut
      'M22 100V80H58V100M18 82L40 66L62 82M48 71.8V64H53V75.5',
      'M34 100V88H41V100' + box(46, 86, 6, 6),
      // Bank, upper pool, weir face
      'M0 100H66M66 100V86H140L180 100H196',
      // Sluice walk over the crest
      'M136 74H186M136 68H186M140 68V86M160 68V93M182 68V100',
      // Stone steps up the far side
      'M196 100V96H208V92H220V88H232V84H262V100',
      still(250, 84),
    ],
    dim: [
      // Water going over
      'M144 88L170 100M150 87L176 99M157 87L183 98M164 89L186 97',
      'M74 92H96M108 92H134M178 103H194',
      water(176, 106),
      'M226 88V84M238 84V100M250 84V100',
    ],
    faint: [
      'M66 66L104 58L150 63L196 55L240 62L282 54L320 60',
      'M262 100H320',
      'M0 106H160',
    ],
  },

  cottages: {
    label:
      'A terrace of six cottages along the towpath, most of the doors standing open.',
    line: [
      'M24 100V70H264V100M20 72L34 56H254L268 72',
      grid(5, 1, 40, 0, (x) => `M${66 + x} 56V47H${72 + x}V56`),
      // Doors: one shut, five open
      box(32, 84, 8, 16),
      grid(
        5,
        1,
        40,
        0,
        (x) => `M${72 + x} 100V84H${80 + x}V100M${80 + x} 84L${85 + x} 86V100`,
      ),
      'M0 100H320',
      still(288, 100),
    ],
    dim: [
      grid(5, 1, 40, 0, (x) => `M${64 + x} 70V100`),
      grid(6, 1, 40, 0, (x) => box(46 + x, 86, 10, 8) + box(44 + x, 74, 8, 6)),
      water(0, 106),
    ],
    faint: ['M0 64L22 60M270 58L296 52L320 56'],
  },

  chapel: {
    label: 'A grey chapel by the old ford. The bell tower is silent.',
    line: [
      // Tower, belfry, cap
      'M86 100V44H112V100M84 44L99 26L114 44',
      arched(93, 50, 12, 14),
      'M96 64L97 58A2 2 0 0 1 101 58L102 64Z',
      arched(93, 84, 12, 16),
      // Nave
      'M112 100V70H192V100M112 70L120 58H186L196 70',
      arched(124, 78, 8, 14) + arched(146, 78, 8, 14) + arched(168, 78, 8, 14),
      'M0 100H200',
      still(62, 100),
    ],
    dim: [
      // Graves, one leaning
      arched(24, 92, 6, 8) +
        arched(38, 90, 6, 10) +
        'M48 100L52 92A3 3 0 0 1 57 94L54 100',
      // The ford: stones in the shallows
      'M206 100H320',
      'M214 106A4 2 0 0 1 222 106M236 106A4 2 0 0 1 244 106M258 106A4 2 0 0 1 266 106M280 106A4 2 0 0 1 288 106',
      'M200 112H236M250 112H300M214 118H262M276 118H320',
    ],
    faint: ['M0 70L40 62L84 66M196 66L236 58L278 64L320 56'],
  },

  mill: {
    label:
      'A five-storey mill of black brick over the river, its chimney cold.',
    line: [
      'M70 100V30H202V100M66 30H206M66 26H206',
      // Engine house
      'M30 100V72H70M26 74L48 58L70 74',
      arched(42, 78, 12, 18),
      // Chimney
      'M222 100L225 6H233L236 100M223 10H235',
      'M0 100H320',
      still(262, 100),
    ],
    dim: [
      grid(8, 5, 16, 14, (x, y) => box(78 + x, 34 + y, 6, 7)),
      'M70 44H202M70 58H202M70 72H202M70 86H202',
      water(0, 106),
    ],
    faint: ['M0 62L30 56M240 58L280 50L320 56', 'M206 92H222M236 92H320'],
  },

  school: {
    label: 'A board school behind railings, two classrooms and an empty yard.',
    line: [
      // Two gables and the link with its bell-cote
      'M60 100V68L90 48L120 68V100M60 68H120',
      'M140 100V68L170 48L200 68V100M140 68H200',
      'M120 74H140M126 74V62H134V74M124 62L130 54L136 62',
      arched(70, 74, 8, 16) + arched(86, 70, 8, 20) + arched(102, 74, 8, 16),
      arched(150, 74, 8, 16) + arched(166, 70, 8, 20) + arched(182, 74, 8, 16),
      arched(126, 84, 8, 16),
      'M0 100H320',
      still(236, 100),
    ],
    dim: [
      // Railings
      'M16 94H118M142 94H304',
      grid(17, 1, 6, 0, (x) => `M${20 + x} 100V92`),
      grid(27, 1, 6, 0, (x) => `M${146 + x} 100V92`),
      'M118 100V90M142 100V90',
      'M128 66V64M132 66V64',
    ],
    faint: ['M0 66L40 58L58 62M204 62L250 54L290 60L320 56', 'M0 108H320'],
  },

  surgery: {
    label:
      'The doctor’s house on Quarry Lane, a brass plate by the door and a lamp beside it.',
    line: [
      'M90 100V52H190V100M86 52L106 38H174L194 52',
      'M112 38V30H120V38M160 38V30H168V38',
      // Door, fanlight, plate
      arched(134, 78, 12, 22),
      'M134 84H146M151 88H156V92H151Z',
      box(100, 60, 14, 12) + box(166, 60, 14, 12) + box(132, 60, 16, 12),
      box(100, 80, 14, 14) + box(166, 80, 14, 14),
      // The lamp
      'M70 100V62M66 62H74M67 62V55H73V62M66 55L70 51L74 55',
      'M0 100H200L320 94',
      still(238, 99),
    ],
    dim: [
      'M107 60V72M173 60V72M140 60V72M100 66H114M166 66H180',
      'M107 80V94M173 80V94',
      'M200 100L320 110',
    ],
    faint: [
      'M196 64L216 56L236 58L248 46H272L282 60L320 54',
      'M0 66L40 58L84 64',
    ],
  },

  boatyard: {
    label:
      'Boat sheds and slipways under the town bridge, hulls up on trestles.',
    line: [
      // Shed with its open front
      'M12 100V66H80V100M8 68L44 50L84 68M24 100V78H68V100',
      // Hull on trestles
      'M90 80H158L152 88L144 92H106L96 88Z',
      'M110 92L106 100M110 92L114 100M140 92L136 100M140 92L144 100',
      // Dry dock
      'M0 100H166V114H226V100H236',
      // The bridge
      'M236 58H320M236 64H320M236 64V100M248 100V82A42 42 0 0 1 304 82V100',
      still(286, 58),
    ],
    dim: [
      'M34 78L40 86H56L62 78',
      'M166 104H226M166 109H226',
      'M174 112L180 104H210L216 112',
      water(236, 106),
    ],
    faint: ['M84 70L120 62L160 66L200 54L236 58', 'M0 106H130'],
  },

  garage: {
    label:
      'A motor garage at the edge of town, two pumps, and a lorry nose-down in the ditch.',
    line: [
      'M34 100V66H138V100M40 66V56H132V66',
      'M50 100V74H82V100M90 100V74H122V100',
      // Pumps
      'M156 100V78H166V100' + ring(161, 70, 4) + 'M161 74V78',
      'M178 100V78H188V100' + ring(183, 70, 4) + 'M183 74V78',
      // Road and the ditch
      'M0 100H238L250 114H300L306 100H320',
      // The lorry: bed, cab, bonnet, wheels
      'M226 92L230.3 80.8L267.5 95.2L263.2 106.4Z',
      'M266.9 107.8L274.1 89.2L283.4 92.8L281.2 98.4L288.7 101.3L283.7 114.3',
      'M226 92L283.7 114.3',
      ring(233.4, 94.9, 5) + ring(272.5, 110, 5),
      still(204, 100),
    ],
    dim: [
      'M166 86H169V96M188 86H191V96',
      'M56 74V100M64 74V100M72 74V100M96 74V100M104 74V100M112 74V100',
      'M273 92L270 99',
    ],
    faint: ['M0 64L34 58M140 62L180 54L220 60L262 50L320 58', 'M0 108H220'],
  },

  station: {
    label:
      'A red-brick police station, a blue lamp over the door and bars on the low windows.',
    line: [
      'M80 100V44H220V100M76 44H224M76 40H224',
      // Door and steps
      'M142 100V74H158V100M136 100H164V96H136',
      // The blue lamp
      'M150 74V68M146 68H154L153 60H147ZM146 60L150 56L154 60',
      // Low windows, barred
      box(92, 78, 18, 14) +
        box(114, 78, 18, 14) +
        box(168, 78, 18, 14) +
        box(190, 78, 18, 14),
      'M0 100H320',
      still(266, 100),
    ],
    dim: [
      [92, 114, 168, 190]
        .map((x) => `M${x + 4} 78V92M${x + 9} 78V92M${x + 14} 78V92`)
        .join(''),
      box(92, 52, 18, 16) +
        box(114, 52, 18, 16) +
        box(168, 52, 18, 16) +
        box(190, 52, 18, 16),
      'M80 72H220M80 48H220',
    ],
    faint: ['M0 66L40 58L78 62M226 60L270 52L320 58', 'M0 108H320'],
  },

  wharf: {
    label:
      'The coal wharf: black heaps under a crane, and a barge half sunk at its moorings.',
    line: [
      'M0 96H184V104',
      // Heaps
      'M14 96L40 72L68 96M58 96L80 78L104 96',
      // Crane: mast, jib, stays, hook
      'M128 96V46M122 96L128 82L134 96M128 90L196 40M128 46L196 40',
      'M192 43V72M189 72A3 3 0 0 0 195 72',
      // The barge, going down by the stern
      'M214 104L292 94L297 92L294 104',
      'M236 101.2V98.2L276 93.1V96.1M258 98.8V90',
      still(104, 96),
    ],
    dim: [
      'M30 96L40 86L48 96M70 96L80 86L88 96',
      'M128 60L146 76M128 74L162 64',
      'M184 104H210M298 104H320M206 112H320M184 110H198M192 118H320',
    ],
    faint: ['M0 64L40 56L84 62L124 54', 'M200 58L240 50L282 58L320 52'],
  },

  farm: {
    label:
      'A farmhouse and barn high on the valley side, the orchard gone wild.',
    line: [
      // Farmhouse
      'M28 100V64H90V100M24 66L40 52H78L94 66M44 52V44H50V52',
      box(38, 72, 10, 10) + box(70, 72, 10, 10) + 'M55 100V84H63V100',
      // Barn
      'M112 100V64M184 64V100M108 66L148 42L188 66',
      'M134 100V72H162V100M134 72L162 100M162 72L134 100',
      'M0 100H320',
      still(256, 100),
    ],
    dim: [
      // Orchard: crooked trees
      'M216 100V82L208 72M216 86L226 74M216 82L214 70',
      'M236 100V84L244 74M236 90L226 80M244 74L250 72',
      'M280 100V80L272 70M280 86L292 76M292 76L296 78M280 80L284 68',
      'M300 100V88L308 80M300 92L294 84',
      'M0 104H40M60 104H110',
      'M94 100V96H108V100',
    ],
    faint: ['M0 58L30 50L70 48L104 44M192 38L250 28L320 24'],
  },
};

const FALLBACK: Scene = {
  label: 'A road leading to a low building, and someone standing on it.',
  line: [
    'M0 100H320',
    'M96 120L158 100M224 120L168 100',
    'M140 100V88H196V100M136 90L146 82H190L200 90',
    box(146, 92, 6, 4),
    still(186, 110),
  ],
  dim: ['M163 104V106M163 111V114M163 118V120', 'M60 100V92M54 94L60 88L66 94'],
  faint: ['M0 84L40 76L90 82L140 74L190 80L240 70L280 78L320 72'],
};

/** Ids of the places with their own drawing; any other id gets the road. */
export const PLACE_ART_IDS: string[] = Object.keys(SCENES);

export function PlaceArt({ place, label }: { place: string; label?: string }) {
  const scene = SCENES[place] ?? FALLBACK;
  return (
    <svg
      class="art art--place"
      viewBox="0 0 320 120"
      fill="none"
      stroke-linecap="square"
      stroke-linejoin="miter"
      role="img"
      aria-label={label ?? scene.label}
    >
      <g class="art__faint">
        {scene.faint.map((d) => (
          <path key={d} d={d} />
        ))}
      </g>
      <g class="art__dim">
        {scene.dim.map((d) => (
          <path key={d} d={d} />
        ))}
      </g>
      <g class="art__line">
        {scene.line.map((d) => (
          <path key={d} d={d} />
        ))}
      </g>
    </svg>
  );
}
