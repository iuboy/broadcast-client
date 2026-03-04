// AudioWorklet for audio processing
// 自适应版本：使用 AudioContext 的实际硬件采样率

class AudioProcessor extends (globalThis as any).AudioWorkletProcessor {
  private sampleRate: number = 0;
  private volume: number = 1.0;
  private frameSize: number = 1024;
  private sampleBuffer: Int16Array;
  private bufferOffset: number = 0;

  constructor() {
    super();
    this.sampleRate = (this as any).sampleRate;
    this.sampleBuffer = new Int16Array(this.frameSize);
    console.log(`[AudioProcessor] 初始化: ${this.sampleRate}Hz, 帧大小=${this.frameSize}`);

    if (this.port) {
      this.port.onmessage = (event: MessageEvent) => {
        if (event.data.volume !== undefined) {
          this.volume = event.data.volume;
        }
      };
    }
  }

  process(inputList: Float32Array[][]): boolean {
    const input = inputList[0];
    if (input.length === 0) return true;

    const inputData = input[0];
    for (let i = 0; i < inputData.length; i++) {
      let sample = inputData[i] * this.volume;
      if (Math.abs(sample) > 1.0) sample = Math.tanh(sample);
      this.sampleBuffer[this.bufferOffset++] = Math.max(-32768, Math.min(32767, Math.round(sample * 0x7FFF)));

      if (this.bufferOffset >= this.frameSize) {
        this.sendFrame();
        this.bufferOffset = 0;
      }
    }
    return true;
  }

  private sendFrame(): void {
    if (!this.port) return;

    const chunk = new ArrayBuffer(this.frameSize * 2);
    const view = new DataView(chunk);
    for (let i = 0; i < this.frameSize; i++) {
      view.setInt16(i * 2, this.sampleBuffer[i], true);
    }
    this.port.postMessage({ chunk, sampleRate: this.sampleRate }, [chunk]);
  }
}

// @ts-ignore
(globalThis as any).registerProcessor('audio-processor', AudioProcessor);
