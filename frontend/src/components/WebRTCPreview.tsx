import { Component, Show, createEffect, onCleanup } from 'solid-js';

interface Props {
  stream: MediaStream | null;
  mjpegSrc: string;
  alt?: string;
  class?: string;
  onLoadedMetadata?: (e: Event) => void;
  onLoad?: (e: Event) => void;
  draggable?: boolean;
}

const WebRTCPreview: Component<Props> = (props) => {
  let videoRef!: HTMLVideoElement;

  createEffect(() => {
    if (videoRef) {
      videoRef.srcObject = props.stream ?? null;
    }
  });

  onCleanup(() => {
    if (videoRef) videoRef.srcObject = null;
  });

  return (
    <Show
      when={props.stream}
      fallback={
        <img
          class={props.class}
          src={props.mjpegSrc}
          alt={props.alt ?? 'Preview'}
          onLoad={props.onLoad}
          draggable={props.draggable}
        />
      }
    >
      <video
        ref={videoRef}
        class={props.class}
        autoplay
        playsinline
        muted
        onLoadedMetadata={props.onLoadedMetadata}
        draggable={props.draggable}
        style={{ "object-fit": "contain", width: "100%", height: "100%" }}
      />
    </Show>
  );
};

export default WebRTCPreview;
