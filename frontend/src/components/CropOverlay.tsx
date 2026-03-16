import { Component, For, createSignal, onMount, onCleanup } from 'solid-js';
import type { OutputConfig, CropRegion } from '../api/types';
import { computePreviewRect, OUTPUT_COLORS, type Resolution, type PreviewRect } from '../utils/coordinates';
import CropRect from './CropRect';

interface Props {
  src: string;
  outputs: OutputConfig[];
  inputRes: Resolution;
  selectedOutputId: string | null;
  onSelectOutput: (id: string | null) => void;
  onCropChange: (id: string, crop: CropRegion) => void;
}

const CropOverlay: Component<Props> = (props) => {
  let containerRef!: HTMLDivElement;
  const [previewRect, setPreviewRect] = createSignal<PreviewRect>({ x: 0, y: 0, width: 0, height: 0 });

  const updateRect = () => {
    if (!containerRef) return;
    const w = containerRef.clientWidth;
    const h = containerRef.clientHeight;
    if (w > 0 && h > 0) {
      setPreviewRect(computePreviewRect(w, h, props.inputRes.width, props.inputRes.height));
    }
  };

  onMount(() => {
    const observer = new ResizeObserver(updateRect);
    observer.observe(containerRef);
    onCleanup(() => observer.disconnect());
  });

  const handleContainerClick = (e: MouseEvent) => {
    if (e.target === containerRef || (e.target as HTMLElement).tagName === 'IMG') {
      props.onSelectOutput(null);
    }
  };

  return (
    <div
      ref={containerRef}
      class="crop-overlay-container"
      onClick={handleContainerClick}
    >
      <img
        class="preview-img"
        src={props.src}
        alt="Input preview"
        onLoad={updateRect}
        draggable={false}
      />
      <For each={props.outputs}>
        {(output, index) => {
          const crop = () => output.transform.crop;
          const color = () => OUTPUT_COLORS[index() % OUTPUT_COLORS.length];
          return crop() ? (
            <CropRect
              crop={crop()!}
              color={color()}
              label={output.name}
              selected={props.selectedOutputId === output.id}
              inputRes={props.inputRes}
              previewRect={previewRect()}
              onMove={(c) => props.onCropChange(output.id, c)}
              onResize={(c) => props.onCropChange(output.id, c)}
              onSelect={() => props.onSelectOutput(output.id)}
            />
          ) : null;
        }}
      </For>
    </div>
  );
};

export default CropOverlay;
