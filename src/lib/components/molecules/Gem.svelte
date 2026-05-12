<!-- credit to <@538349263428780063> on svelte's discord server-->

<script lang="ts">
  import Zdog from 'zdog';

  export let className: string = '';

  const TAU = Zdog.TAU;
  const SIDES = 8;

  type Point = {
    x: number;
    y: number;
    z: number;
  };

  // Colors
  const tableColor = '#dff0ff';
  const crownColorA = '#b8d8f8';
  const crownColorB = '#7ab4ee';
  const crownColorC = '#4a8fd4';
  const pavilionColorA = '#5a9ee0';
  const pavilionColorB = '#2c6ab8';
  const pavilionColorC = '#1a4a90';
  const girdleColorA = '#a0c8f0';
  const girdleColorB = '#6aaae0';

  // Geometry
  const girdleRadius = 1;
  const tableRadius = 0.55;
  const crownHeight = 0.35;
  const pavilionDepth = 1;
  const girdleHeight = 0.02;

  const apothem = girdleRadius * Math.cos(Math.PI / SIDES);
  const halfWidth = girdleRadius * Math.sin(Math.PI / SIDES);

  function pt(index: number, radius: number, y: number): Point {
    const angle = (index / SIDES) * TAU;

    return {
      x: Math.cos(angle) * radius,
      y,
      z: Math.sin(angle) * radius
    };
  }

  function diamond(canvas: HTMLCanvasElement): () => void {
    const illo = new Zdog.Illustration({
      element: canvas,
      dragRotate: true,
      rotate: { x: -0.18 },
      scale: 30
    });

    const gem = new Zdog.Anchor({
      addTo: illo
    });

    const culet: Point = { x: 0, y: pavilionDepth, z: 0 };

    for (let i = 0; i < SIDES; i++) {
      const even = i % 2 === 0;

      // Girdle
      const midAngle = ((i + 0.5) / SIDES) * TAU;

      const girdleRotor = new Zdog.Anchor({
        addTo: gem,
        rotate: { y: midAngle }
      });

      new Zdog.Shape({
        addTo: girdleRotor,
        path: [
          { x: -halfWidth, y: -girdleHeight / 2, z: apothem },
          { x: halfWidth, y: -girdleHeight / 2, z: apothem },
          { x: halfWidth, y: girdleHeight / 2, z: apothem },
          { x: -halfWidth, y: girdleHeight / 2, z: apothem }
        ],
        stroke: 1,
        fill: true,
        color: even ? girdleColorA : girdleColorB,
        backface: even ? girdleColorA : girdleColorB
      });

      // Crown
      const crownGroup = new Zdog.Group({ addTo: gem });

      new Zdog.Shape({
        addTo: crownGroup,
        path: [
          pt(i, girdleRadius, 0),
          pt(i + 0.5, girdleRadius, 0),
          pt(i, tableRadius, -crownHeight)
        ],
        stroke: 1,
        fill: true,
        color: even ? crownColorA : crownColorB,
        backface: even ? crownColorA : crownColorB
      });

      new Zdog.Shape({
        addTo: crownGroup,
        path: [
          pt(i + 0.5, girdleRadius, 0),
          pt(i + 1, tableRadius, -crownHeight),
          pt(i, tableRadius, -crownHeight)
        ],
        stroke: false,
        fill: true,
        color: even ? crownColorB : crownColorC,
        backface: even ? crownColorB : crownColorC
      });

      new Zdog.Shape({
        addTo: crownGroup,
        path: [
          pt(i + 0.5, girdleRadius, 0),
          pt(i + 1, girdleRadius, 0),
          pt(i + 1, tableRadius, -crownHeight)
        ],
        stroke: false,
        fill: true,
        color: even ? crownColorA : crownColorB,
        backface: even ? crownColorA : crownColorB
      });

      // Pavilion
      const pavilionMid = pt(
        i + 0.5,
        girdleRadius * 0.55,
        pavilionDepth * 0.45
      );

      const pavGroup = new Zdog.Group({ addTo: gem });

      new Zdog.Shape({
        addTo: pavGroup,
        path: [
          pt(i, girdleRadius, 0),
          pt(i + 1, girdleRadius, 0),
          pavilionMid
        ],
        stroke: 1,
        fill: true,
        color: even ? pavilionColorA : pavilionColorB,
        backface: even ? pavilionColorA : pavilionColorB
      });

      new Zdog.Shape({
        addTo: pavGroup,
        path: [
          pt(i, girdleRadius, 0),
          pavilionMid,
          culet
        ],
        stroke: false,
        fill: true,
        color: even ? pavilionColorB : pavilionColorC,
        backface: even ? pavilionColorB : pavilionColorC
      });

      new Zdog.Shape({
        addTo: pavGroup,
        path: [
          pt(i + 1, girdleRadius, 0),
          culet,
          pavilionMid
        ],
        stroke: 1,
        fill: true,
        color: even ? pavilionColorB : pavilionColorC,
        backface: even ? pavilionColorB : pavilionColorC
      });
    }

    // Table
    new Zdog.Shape({
      addTo: gem,
      path: Array.from(
        { length: SIDES },
        (_, i): Point => pt(i, tableRadius, -crownHeight)
      ),
      stroke: 1,
      fill: true,
      color: tableColor,
      backface: tableColor
    });

    let frame: number;

    function animate(): void {
      illo.rotate.y += 0.01;
      illo.updateRenderGraph();

      frame = requestAnimationFrame(animate);
    }

    animate();

    // cleanup
    return (): void => {
      cancelAnimationFrame(frame);
    };
  }
</script>

<canvas
  width="500"
  height="60"
  class={className}
  {@attach diamond}
>
</canvas>