import type { InputSource, CropRegion } from '../api/types';

export const DISPLAY_MODE_RESOLUTIONS: Record<string, { width: number; height: number }> = {
  Hd720p50:     { width: 1280, height: 720 },
  Hd720p5994:   { width: 1280, height: 720 },
  Hd720p60:     { width: 1280, height: 720 },
  Hd1080i50:    { width: 1920, height: 1080 },
  Hd1080i5994:  { width: 1920, height: 1080 },
  Hd1080p24:    { width: 1920, height: 1080 },
  Hd1080p25:    { width: 1920, height: 1080 },
  Hd1080p2997:  { width: 1920, height: 1080 },
  Hd1080p30:    { width: 1920, height: 1080 },
  Hd1080p50:    { width: 1920, height: 1080 },
  Hd1080p5994:  { width: 1920, height: 1080 },
  Hd1080p60:    { width: 1920, height: 1080 },
  Uhd2160p24:   { width: 3840, height: 2160 },
  Uhd2160p25:   { width: 3840, height: 2160 },
  Uhd2160p2997: { width: 3840, height: 2160 },
  Uhd2160p30:   { width: 3840, height: 2160 },
  Uhd2160p50:   { width: 3840, height: 2160 },
  Uhd2160p5994: { width: 3840, height: 2160 },
  Uhd2160p60:   { width: 3840, height: 2160 },
};

export const OUTPUT_COLORS = ['#4dabf7', '#ffd43b', '#69db7c', '#ff6b6b', '#da77f2', '#ff922b'];

export interface Resolution {
  width: number;
  height: number;
}

export interface PreviewRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function getInputResolution(input: InputSource): Resolution {
  switch (input.type) {
    case 'Mock':
      return { width: input.width, height: input.height };
    case 'DeckLink': {
      const res = DISPLAY_MODE_RESOLUTIONS[input.display_mode];
      return res ?? { width: 1920, height: 1080 };
    }
    case 'Uvc':
      return { width: 1920, height: 1080 };
  }
}

export function computePreviewRect(
  containerW: number,
  containerH: number,
  inputW: number,
  inputH: number,
): PreviewRect {
  const scale = Math.min(containerW / inputW, containerH / inputH);
  const renderW = inputW * scale;
  const renderH = inputH * scale;
  return {
    x: (containerW - renderW) / 2,
    y: (containerH - renderH) / 2,
    width: renderW,
    height: renderH,
  };
}

export function inputToPreview(
  crop: CropRegion,
  inputRes: Resolution,
  previewRect: PreviewRect,
): { left: number; top: number; width: number; height: number } {
  const scaleX = previewRect.width / inputRes.width;
  const scaleY = previewRect.height / inputRes.height;
  return {
    left: crop.x * scaleX + previewRect.x,
    top: crop.y * scaleY + previewRect.y,
    width: crop.width * scaleX,
    height: crop.height * scaleY,
  };
}

export function previewToInput(
  left: number,
  top: number,
  w: number,
  h: number,
  inputRes: Resolution,
  previewRect: PreviewRect,
): CropRegion {
  const scaleX = previewRect.width / inputRes.width;
  const scaleY = previewRect.height / inputRes.height;
  const rawX = (left - previewRect.x) / scaleX;
  const rawY = (top - previewRect.y) / scaleY;
  const rawW = w / scaleX;
  const rawH = h / scaleY;
  return clampCrop(
    { x: rawX, y: rawY, width: rawW, height: rawH },
    inputRes,
  );
}

export function clampCrop(crop: CropRegion, inputRes: Resolution): CropRegion {
  let x = Math.round(crop.x / 2) * 2;
  let y = Math.round(crop.y);
  let w = Math.round(crop.width / 2) * 2;
  let h = Math.round(crop.height);

  // Minimum size
  if (w < 2) w = 2;
  if (h < 2) h = 2;

  // Clamp position
  if (x < 0) x = 0;
  if (y < 0) y = 0;

  // Clamp to bounds
  if (x + w > inputRes.width) {
    x = Math.round((inputRes.width - w) / 2) * 2;
    if (x < 0) { x = 0; w = Math.round(inputRes.width / 2) * 2; }
  }
  if (y + h > inputRes.height) {
    y = inputRes.height - h;
    if (y < 0) { y = 0; h = inputRes.height; }
  }

  return { x, y, width: w, height: h };
}
