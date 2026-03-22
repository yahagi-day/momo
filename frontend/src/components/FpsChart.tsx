import { Component, onMount, onCleanup, createEffect } from 'solid-js';

interface Props {
  fps: number;
  height?: number;
  maxPoints?: number;
  className?: string;
}

const FpsChart: Component<Props> = (props) => {
  let canvasRef!: HTMLCanvasElement;
  let animId = 0;
  const history: number[] = [];
  const MAX_POINTS = () => props.maxPoints ?? 120;

  createEffect(() => {
    const val = props.fps;
    history.push(val);
    if (history.length > MAX_POINTS()) {
      history.splice(0, history.length - MAX_POINTS());
    }
  });

  const draw = () => {
    const ctx = canvasRef.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const w = canvasRef.clientWidth;
    const h = canvasRef.clientHeight;

    const pw = Math.round(w * dpr);
    const ph = Math.round(h * dpr);
    if (canvasRef.width !== pw || canvasRef.height !== ph) {
      canvasRef.width = pw;
      canvasRef.height = ph;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    ctx.clearRect(0, 0, w, h);

    if (history.length < 2) {
      animId = requestAnimationFrame(draw);
      return;
    }

    // Determine Y range
    const maxFps = Math.max(...history, 1);
    const ceiling = Math.ceil(maxFps / 10) * 10 || 60;
    const padding = { top: 6, bottom: 6, left: 0, right: 0 };
    const plotW = w - padding.left - padding.right;
    const plotH = h - padding.top - padding.bottom;

    // Grid lines
    const gridSteps = ceiling <= 30 ? 10 : ceiling <= 60 ? 15 : 30;
    ctx.strokeStyle = '#ffffff10';
    ctx.lineWidth = 1;
    for (let v = gridSteps; v < ceiling; v += gridSteps) {
      const y = padding.top + plotH * (1 - v / ceiling);
      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(w - padding.right, y);
      ctx.stroke();
    }

    // Target line (e.g., 30fps or 60fps)
    const targetFps = ceiling <= 30 ? 30 : 60;
    if (targetFps <= ceiling) {
      const ty = padding.top + plotH * (1 - targetFps / ceiling);
      ctx.strokeStyle = '#ffffff20';
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(padding.left, ty);
      ctx.lineTo(w - padding.right, ty);
      ctx.stroke();
      ctx.setLineDash([]);

      ctx.fillStyle = '#ffffff40';
      ctx.font = '10px system-ui';
      ctx.textAlign = 'right';
      ctx.fillText(`${targetFps}`, w - padding.right - 2, ty - 3);
    }

    // Build points
    const points: [number, number][] = [];
    for (let i = 0; i < history.length; i++) {
      const x = padding.left + (i / (MAX_POINTS() - 1)) * plotW;
      const y = padding.top + plotH * (1 - history[i] / ceiling);
      points.push([x, y]);
    }

    // Fill under line
    const gradient = ctx.createLinearGradient(0, padding.top, 0, h - padding.bottom);
    gradient.addColorStop(0, '#0AB9E640');
    gradient.addColorStop(1, '#0AB9E600');
    ctx.beginPath();
    ctx.moveTo(points[0][0], h - padding.bottom);
    for (const [x, y] of points) ctx.lineTo(x, y);
    ctx.lineTo(points[points.length - 1][0], h - padding.bottom);
    ctx.closePath();
    ctx.fillStyle = gradient;
    ctx.fill();

    // Line
    ctx.beginPath();
    ctx.strokeStyle = '#0AB9E6';
    ctx.lineWidth = 1.5;
    ctx.lineJoin = 'round';
    for (let i = 0; i < points.length; i++) {
      i === 0 ? ctx.moveTo(points[i][0], points[i][1]) : ctx.lineTo(points[i][0], points[i][1]);
    }
    ctx.stroke();

    // Current value dot
    const last = points[points.length - 1];
    ctx.beginPath();
    ctx.arc(last[0], last[1], 3, 0, Math.PI * 2);
    ctx.fillStyle = '#0AB9E6';
    ctx.fill();

    // Current FPS label
    const currentFps = history[history.length - 1];
    ctx.fillStyle = '#ffffffcc';
    ctx.font = 'bold 11px system-ui';
    ctx.textAlign = 'right';
    ctx.fillText(`${currentFps.toFixed(1)} fps`, last[0] - 8, last[1] - 8);

    animId = requestAnimationFrame(draw);
  };

  onMount(() => {
    animId = requestAnimationFrame(draw);
  });

  onCleanup(() => {
    cancelAnimationFrame(animId);
  });

  return (
    <canvas
      ref={canvasRef}
      class={props.className ?? 'waveform'}
      style={{ width: '100%', height: `${props.height ?? 64}px` }}
    />
  );
};

export default FpsChart;
