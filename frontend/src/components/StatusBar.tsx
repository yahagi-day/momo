import { Component } from 'solid-js';
import type { PipelineState } from '../api/types';

interface Props {
  state: PipelineState;
  fps: number;
  onStart: () => void;
  onStop: () => void;
}

const StatusBar: Component<Props> = (props) => {
  const isRunning = () => props.state === 'Running';
  const isBusy = () => props.state === 'Starting' || props.state === 'Stopping';

  return (
    <div class="status-bar">
      <h1>MOMO</h1>
      <div class="info">
        {isRunning() && <span>{props.fps.toFixed(1)} fps</span>}
        <span class={`status-badge ${props.state.toLowerCase()}`}>{props.state}</span>
        {isRunning() ? (
          <button class="btn-stop" onClick={props.onStop} disabled={isBusy()}>
            Stop
          </button>
        ) : (
          <button class="btn-start" onClick={props.onStart} disabled={isBusy()}>
            Start
          </button>
        )}
      </div>
    </div>
  );
};

export default StatusBar;
