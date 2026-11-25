/**
 * Title illustration: the lock house on the near bank, the lock gate, the river,
 * the far slip with the ferry, and one figure that is standing too still.
 * Stroke only, hairline (non-scaling), square caps, mitre joins.
 */
export function LockHouseArt() {
  return (
    <svg
      class="art"
      viewBox="0 0 320 200"
      fill="none"
      stroke-linecap="square"
      stroke-linejoin="miter"
      role="img"
      aria-label="A lock house beside a river at dusk. On the far bank, a ferry at its slip."
    >
      {/* Distant valley: hills and bare trees */}
      <g class="art__faint">
        <path d="M0 106L30 99M98 92L118 86L160 94L204 82L246 92L284 80L320 88" />
        <path d="M150 116H320" />
      </g>
      <g class="art__dim">
        <path d="M268 116V96M268 104L262 98M268 108L274 101M268 100L271 95" />
        <path d="M286 116V90M286 100L279 93M286 106L293 98M286 96L290 90" />
        <path d="M304 116V99M304 106L299 101M304 109L309 104" />
        {/* The far slip and the ferry, moored */}
        <path d="M196 116L218 130" />
        <path d="M222 128H262L257 135H226ZM231 128V122H242V128M247 128V118H250V128" />
      </g>
      {/* A figure on the far bank, standing still */}
      <g class="art__line">
        <path d="M296 112.5A1.5 1.5 0 1 0 296 109.5A1.5 1.5 0 1 0 296 112.5ZM296 112.5V120M296 120L295 126M296 120L297 126" />
      </g>

      {/* The lock house */}
      <g class="art__line">
        <path d="M40 132V92H88V132" />
        <path d="M34 96L64 62L94 96" />
        <path d="M74 73.3V56H80V80.1" />
        <path d="M58 132V112H68V132" />
        <path d="M74 104H82V114H74ZM78 104V114" />
        <path d="M60 77H67V85H60Z" />
        {/* Near bank and embankment stones */}
        <path d="M0 132H112M0 140H112" />
        <path d="M14 132V140M36 132V140M62 132V140M90 132V140" />
        {/* The lock gate: walkway, rails, gate leaves, balance beam */}
        <path d="M108 126H140M108 126V132M140 126V132M124 126V122" />
        <path d="M116 132V164M132 132V164M116 140L132 156M116 156L132 140" />
        <path d="M132 127L160 119" />
        <path d="M140 132H178L190 140" />
      </g>

      {/* The river */}
      <g class="art__dim">
        <path d="M0 150H24M38 150H104M146 150H186M204 150H250M262 150H320" />
        <path d="M12 162H72M86 162H110M140 162H230M248 162H300" />
        <path d="M30 176H128M160 176H252M272 176H320" />
        <path d="M0 190H54M82 190H196M226 190H320" />
      </g>
      {/* The lit window, reflected and broken by the current */}
      <g class="art__faint">
        <path d="M74 154H82M76 168H81M75 182H80" />
      </g>
    </svg>
  );
}
