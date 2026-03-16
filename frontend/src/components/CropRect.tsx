import { Component, Show, createMemo } from 'solid-js';
import type { CropRegion } from '../api/types';
import { inputToPreview, previewToInput, type Resolution, type PreviewRect } from '../utils/coordinates';

interface Props {
  crop: CropRegion;
  color: string;
  label: string;
  selected: boolean;
  inputRes: Resolution;
  previewRect: PreviewRect;
  onMove: (crop: CropRegion) => void;
  onResize: (crop: CropRegion) => void;
  onSelect: () => void;
}

const HANDLE_SIZE = 8;

type HandlePos = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

const HANDLE_CURSORS: Record<HandlePos, string> = {
  nw: 'nwse-resize', n: 'ns-resize', ne: 'nesw-resize',
  w: 'ew-resize', e: 'ew-resize',
  sw: 'nesw-resize', s: 'ns-resize', se: 'nwse-resize',
};

const CropRect: Component<Props> = (props) => {
  const pos = createMemo(() =>
    inputToPreview(props.crop, props.inputRes, props.previewRect)
  );

  const startDrag = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    props.onSelect();

    const startX = e.clientX;
    const startY = e.clientY;
    const startPos = pos();

    const onMouseMove = (ev: MouseEvent) => {
      const dx = ev.clientX - startX;
      const dy = ev.clientY - startY;
      const newCrop = previewToInput(
        startPos.left + dx, startPos.top + dy,
        startPos.width, startPos.height,
        props.inputRes, props.previewRect,
      );
      props.onMove(newCrop);
    };

    const onMouseUp = () => {
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
    };

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  };

  const startResize = (handle: HandlePos, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const startX = e.clientX;
    const startY = e.clientY;
    const startPos = pos();
    const startLeft = startPos.left;
    const startTop = startPos.top;
    const startRight = startPos.left + startPos.width;
    const startBottom = startPos.top + startPos.height;

    const onMouseMove = (ev: MouseEvent) => {
      const dx = ev.clientX - startX;
      const dy = ev.clientY - startY;

      let left = startLeft;
      let top = startTop;
      let right = startRight;
      let bottom = startBottom;

      if (handle.includes('w')) left += dx;
      if (handle.includes('e')) right += dx;
      if (handle.includes('n')) top += dy;
      if (handle.includes('s')) bottom += dy;

      // Ensure minimum size in preview pixels
      if (right - left < 4) right = left + 4;
      if (bottom - top < 4) bottom = top + 4;

      const newCrop = previewToInput(
        left, top, right - left, bottom - top,
        props.inputRes, props.previewRect,
      );
      props.onResize(newCrop);
    };

    const onMouseUp = () => {
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
    };

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  };

  const handlePositions = createMemo((): { pos: HandlePos; left: string; top: string }[] => {
    const p = pos();
    const half = HANDLE_SIZE / 2;
    return [
      { pos: 'nw', left: `${-half}px`, top: `${-half}px` },
      { pos: 'n',  left: `${p.width / 2 - half}px`, top: `${-half}px` },
      { pos: 'ne', left: `${p.width - half}px`, top: `${-half}px` },
      { pos: 'e',  left: `${p.width - half}px`, top: `${p.height / 2 - half}px` },
      { pos: 'se', left: `${p.width - half}px`, top: `${p.height - half}px` },
      { pos: 's',  left: `${p.width / 2 - half}px`, top: `${p.height - half}px` },
      { pos: 'sw', left: `${-half}px`, top: `${p.height - half}px` },
      { pos: 'w',  left: `${-half}px`, top: `${p.height / 2 - half}px` },
    ];
  });

  return (
    <div
      class={`crop-rect${props.selected ? ' selected' : ''}`}
      style={{
        left: `${pos().left}px`,
        top: `${pos().top}px`,
        width: `${pos().width}px`,
        height: `${pos().height}px`,
        "border-color": props.color,
        "background-color": `${props.color}22`,
      }}
      onMouseDown={startDrag}
    >
      <div
        class="crop-label"
        style={{ "background-color": props.color }}
      >
        {props.label}
      </div>
      <Show when={props.selected}>
        {handlePositions().map((h) => (
          <div
            class="crop-handle"
            style={{
              left: h.left,
              top: h.top,
              width: `${HANDLE_SIZE}px`,
              height: `${HANDLE_SIZE}px`,
              cursor: HANDLE_CURSORS[h.pos],
            }}
            onMouseDown={(e) => startResize(h.pos, e)}
          />
        ))}
      </Show>
    </div>
  );
};

export default CropRect;
