import { Component, onMount, onCleanup } from 'solid-js';

interface Props {
  color1?: string;
  color2?: string;
  height?: number;
  className?: string;
}

const Waveform: Component<Props> = (props) => {
  let canvasRef!: HTMLCanvasElement;
  let animId = 0;
  let time = 0;

  const draw = () => {
    const ctx = canvasRef.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const w = canvasRef.clientWidth;
    const h = canvasRef.clientHeight;

    if (canvasRef.width !== w * dpr || canvasRef.height !== h * dpr) {
      canvasRef.width = w * dpr;
      canvasRef.height = h * dpr;
      ctx.scale(dpr, dpr);
    }

    ctx.clearRect(0, 0, w, h);
    time += 0.02;

    const c1 = props.color1 ?? '#FF3C28';
    const c2 = props.color2 ?? '#0AB9E6';

    // Draw multiple wave layers
    const layers = [
      { color: c1, alpha: 0.6, freq: 0.015, amp: 0.35, speed: 1.0, phase: 0 },
      { color: c1, alpha: 0.3, freq: 0.025, amp: 0.25, speed: 1.4, phase: 2 },
      { color: c2, alpha: 0.6, freq: 0.018, amp: 0.3, speed: -0.8, phase: 1 },
      { color: c2, alpha: 0.3, freq: 0.03, amp: 0.2, speed: -1.2, phase: 3 },
    ];

    for (const layer of layers) {
      ctx.beginPath();
      ctx.strokeStyle = layer.color;
      ctx.globalAlpha = layer.alpha;
      ctx.lineWidth = 2;

      const midY = h / 2;
      const amplitude = h * layer.amp;

      for (let x = 0; x <= w; x += 2) {
        const y = midY +
          Math.sin(x * layer.freq + time * layer.speed + layer.phase) * amplitude * 0.5 +
          Math.sin(x * layer.freq * 2.3 + time * layer.speed * 0.7 + layer.phase * 1.5) * amplitude * 0.3 +
          Math.sin(x * layer.freq * 0.5 + time * layer.speed * 1.3) * amplitude * 0.2;

        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
    }

    // Draw center frequency bars (audio visualizer style)
    ctx.globalAlpha = 1;
    const barCount = Math.floor(w / 6);
    for (let i = 0; i < barCount; i++) {
      const x = i * 6;
      const barH = Math.abs(
        Math.sin(i * 0.15 + time * 2) *
        Math.cos(i * 0.08 + time * 1.5) *
        h * 0.4
      );

      const grad = ctx.createLinearGradient(x, h / 2 - barH / 2, x, h / 2 + barH / 2);
      const ratio = i / barCount;
      if (ratio < 0.5) {
        grad.addColorStop(0, c1 + '60');
        grad.addColorStop(1, c1 + '10');
      } else {
        grad.addColorStop(0, c2 + '60');
        grad.addColorStop(1, c2 + '10');
      }

      ctx.fillStyle = grad;
      ctx.fillRect(x, h / 2 - barH / 2, 4, barH);
    }

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
      style={{ width: '100%', height: `${props.height ?? 80}px` }}
    />
  );
};

export default Waveform;
