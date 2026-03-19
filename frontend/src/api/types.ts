export type PipelineState = 'Stopped' | 'Starting' | 'Running' | 'Stopping' | 'Error';

export interface FlipOptions {
  horizontal: boolean;
  vertical: boolean;
}

export interface CropRegion {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface OutputTransform {
  crop?: CropRegion | null;
  flip: FlipOptions;
}

export interface OutputConfig {
  id: string;
  name: string;
  device_index: number;
  display_mode: string;
  pixel_format: string;
  transform: OutputTransform;
  enabled: boolean;
}

export type InputSource =
  | { type: 'DeckLink'; device_index: number; display_mode: string; pixel_format: string }
  | { type: 'Uvc'; device_path: string }
  | { type: 'Mock'; width: number; height: number; fps: number };

export interface PreviewConfig {
  width: number;
  height: number;
  fps: number;
  jpeg_quality: number;
}

export interface WebConfig {
  bind_address: string;
  port: number;
}

export interface Config {
  input: InputSource;
  outputs: OutputConfig[];
  preview: PreviewConfig;
  web: WebConfig;
}

export interface StatusResponse {
  state: PipelineState;
}

export interface PipelineEvent {
  type: 'StateChanged' | 'FpsUpdate' | 'DeviceEvent' | 'ConfigChanged' | 'Error';
  state?: PipelineState;
  fps?: number;
  device?: string;
  status?: string;
  message?: string;
}

export interface DeviceInfo {
  device_type: string;
  index: number;
  name: string;
  model_name: string;
  has_input: boolean;
  has_output: boolean;
  status: string;
}
