import { Component, onMount, onCleanup } from 'solid-js';

interface Props {
  src?: string;       // MJPEG URL — if provided, waveform is driven by video frames
  color1?: string;
  color2?: string;
  height?: number;
  className?: string;
}

const Waveform: Component<Props> = (props) => {
  let canvasRef!: HTMLCanvasElement;
  let animId = 0;
  let time = 0;

  // Video sampling state
  let sourceImg: HTMLImageElement | null = null;
  let sampleCanvas: HTMLCanvasElement | null = null;
  let smoothed: Float32Array | null = null;

  const SAMPLE_COUNT = 256;

  const sampleFromVideo = (): boolean => {
    if (!sourceImg || !sourceImg.naturalWidth || !sourceImg.naturalHeight) return false;

    if (!sampleCanvas) {
      sampleCanvas = document.createElement('canvas');
      sampleCanvas.width = SAMPLE_COUNT;
      sampleCanvas.height = 1;
    }

    const sCtx = sampleCanvas.getContext('2d', { willReadFrequently: true });
    if (!sCtx) return false;

    try {
      const midY = Math.floor(sourceImg.naturalHeight / 2);
      sCtx.drawImage(
        sourceImg,
        0, midY, sourceImg.naturalWidth, 1,
        0, 0, SAMPLE_COUNT, 1,
      );
      const px = sCtx.getImageData(0, 0, SAMPLE_COUNT, 1).data;

      if (!smoothed || smoothed.length !== SAMPLE_COUNT) {
        smoothed = new Float32Array(SAMPLE_COUNT);
      }

      for (let i = 0; i < SAMPLE_COUNT; i++) {
        const lum = (0.299 * px[i * 4] + 0.587 * px[i * 4 + 1] + 0.114 * px[i * 4 + 2]) / 255;
        smoothed[i] = smoothed[i] * 0.6 + lum * 0.4;
      }
      return true;
    } catch {
      return false;
    }
  };

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
    time += 0.018;

    const c1 = props.color1 ?? '#FF3C28';
    const c2 = props.color2 ?? '#0AB9E6';
    const mid = h / 2;

    const hasVideo = sampleFromVideo();

    if (hasVideo && smoothed) {
      // --- Video-driven waveform ---
      const grad = ctx.createLinearGradient(0, 0, w, 0);
      grad.addColorStop(0, c1);
      grad.addColorStop(0.5, '#ffffff60');
      grad.addColorStop(1, c2);

      // Top mirror fill
      const topGrad = ctx.createLinearGradient(0, 0, 0, mid);
      topGrad.addColorStop(0, c1 + '00');
      topGrad.addColorStop(1, c1 + '40');

      // Bottom mirror fill
      const botGrad = ctx.createLinearGradient(0, mid, 0, h);
      botGrad.addColorStop(0, c2 + '40');
      botGrad.addColorStop(1, c2 + '00');

      const amp = mid * 0.85;

      // Filled area (top half)
      ctx.beginPath();
      ctx.moveTo(0, mid);
      for (let i = 0; i < SAMPLE_COUNT; i++) {
        const x = (i / (SAMPLE_COUNT - 1)) * w;
        const y = mid - (smoothed[i] - 0.5) * amp * 2;
        i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.lineTo(w, mid);
      ctx.closePath();
      ctx.fillStyle = topGrad;
      ctx.fill();

      // Filled area (bottom mirror)
      ctx.beginPath();
      ctx.moveTo(0, mid);
      for (let i = 0; i < SAMPLE_COUNT; i++) {
        const x = (i / (SAMPLE_COUNT - 1)) * w;
        const y = mid + (smoothed[i] - 0.5) * amp * 2;
        i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.lineTo(w, mid);
      ctx.closePath();
      ctx.fillStyle = botGrad;
      ctx.fill();

      // Line on top
      ctx.beginPath();
      ctx.strokeStyle = grad as CanvasGradient;
      ctx.lineWidth = 1.5;
      ctx.globalAlpha = 0.9;
      for (let i = 0; i < SAMPLE_COUNT; i++) {
        const x = (i / (SAMPLE_COUNT - 1)) * w;
        const y = mid - (smoothed[i] - 0.5) * amp * 2;
        i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Mirror line
      ctx.beginPath();
      ctx.globalAlpha = 0.5;
      for (let i = 0; i < SAMPLE_COUNT; i++) {
        const x = (i / (SAMPLE_COUNT - 1)) * w;
        const y = mid + (smoothed[i] - 0.5) * amp * 2;
        i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.stroke();
      ctx.globalAlpha = 1;

    } else {
      // --- Fallback: animated sine waves ---
      const layers = [
        { color: c1, alpha: 0.5, freq: 0.015, amp: 0.35, speed: 1.0, phase: 0 },
        { color: c2, alpha: 0.5, freq: 0.018, amp: 0.3, speed: -0.8, phase: 1 },
      ];

      for (const layer of layers) {
        ctx.beginPath();
        ctx.strokeStyle = layer.color;
        ctx.globalAlpha = layer.alpha;
        ctx.lineWidth = 1.5;
        const amplitude = h * layer.amp;

        for (let x = 0; x <= w; x += 2) {
          const y = mid +
            Math.sin(x * layer.freq + time * layer.speed + layer.phase) * amplitude * 0.6 +
            Math.sin(x * layer.freq * 2.1 + time * layer.speed * 0.8) * amplitude * 0.4;
          x === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
        }
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
    }

    animId = requestAnimationFrame(draw);
  };

  onMount(() => {
    if (props.src) {
      sourceImg = new Image();
      sourceImg.crossOrigin = 'anonymous';
      sourceImg.src = props.src;
    }
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
