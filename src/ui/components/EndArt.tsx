/**
 * Illustrations for the ends of a run that are not death: the ferry going
 * downriver, and the river frozen with someone walking out on it.
 * Stroke only, hairline (non-scaling), square caps, mitre joins.
 */

/** The Kittiwake downriver: the banks open toward the sea; the town is behind. */
function Departure() {
  return (
    <svg
      class="art art--end"
      viewBox="0 0 320 140"
      fill="none"
      stroke-linecap="square"
      stroke-linejoin="miter"
      role="img"
      aria-label="A small ferry on a wide river, going away toward open water. Behind it, a town under a low sky."
    >
      {/* The town behind, small and still: roofs, the mill chimney */}
      <g class="art__faint">
        <path d="M0 62H22L28 56L34 62H48V54H58V62H70L76 58L82 62H96" />
        <path d="M40 54V40H44V54" />
        <path d="M0 70H96" />
      </g>
      {/* The banks drawing apart toward the sea */}
      <g class="art__dim">
        <path d="M0 78L96 70L176 64L240 60L320 58" />
        <path d="M0 132L120 104L200 86L260 76L320 72" />
        <path d="M230 62L236 60M262 60L270 59M290 59H300" />
      </g>
      {/* The far horizon: open water */}
      <g class="art__faint">
        <path d="M250 66H320M272 69H320M292 71H320" />
      </g>
      {/* The river's surface, lengthening as it goes */}
      <g class="art__faint">
        <path d="M40 96H70M96 90H126M150 84H170M188 80H202" />
        <path d="M24 112H66M110 100H150M176 92H196" />
      </g>
      {/* The Kittiwake: hull, wheelhouse, a figure at the wheel, its wake */}
      <g class="art__line" transform="translate(-50 2)">
        <path d="M196 78H232L228 84H200Z" />
        <path d="M206 78V71H216V78" />
        <path d="M222 78V68H224V78" />
        <path d="M211 74.5A1 1 0 1 0 211 72.5A1 1 0 1 0 211 74.5Z" />
      </g>
      <g class="art__dim" transform="translate(-50 2)">
        <path d="M196 82L170 88M196 84L176 92M168 90L150 94" />
      </g>
    </svg>
  );
}

/** The river frozen across, and one set of steps going out over it. */
function FrozenIn() {
  return (
    <svg
      class="art art--end"
      viewBox="0 0 320 140"
      fill="none"
      stroke-linecap="square"
      stroke-linejoin="miter"
      role="img"
      aria-label="The river frozen from bank to bank. The ferry held in the ice at its slip. A figure walks away downstream over the ice."
    >
      {/* Hills and the far bank */}
      <g class="art__faint">
        <path d="M0 50L40 44L84 50L130 40L180 48L230 38L280 46L320 42" />
      </g>
      <g class="art__dim">
        <path d="M0 64H320" />
        <path d="M0 124H320" />
      </g>
      {/* Plates and cracks in the ice */}
      <g class="art__faint">
        <path d="M20 76L60 80L84 74M110 90L150 86L170 94L210 88M240 104L280 98L310 106" />
        <path d="M40 108L72 100L96 112M150 112L190 104M60 80L64 92M170 94L166 106M280 98L286 86" />
      </g>
      {/* The ferry, held fast at the slip */}
      <g class="art__line">
        <path d="M24 64L36 72" />
        <path d="M38 70H74L70 77H42Z" />
        <path d="M46 70V64H56V70M62 70V60H64V70" />
      </g>
      {/* Steps over the ice, and the one walking away */}
      <g class="art__dim">
        <path d="M92 104H95M112 100H115M132 96H135M152 92H155M172 88H175M192 84H195M212 80H215" />
      </g>
      <g class="art__line" transform="translate(4 8)">
        <path d="M236 63.5A1.5 1.5 0 1 0 236 60.5A1.5 1.5 0 1 0 236 63.5ZM236 63.5V71M236 71L234 77M236 71L238 76M236 66L233 70M236 66L239 69" />
      </g>
    </svg>
  );
}

export function EndArt({ kind }: { kind: 'departure' | 'freeze_out' }) {
  return kind === 'departure' ? <Departure /> : <FrozenIn />;
}
