// AudioWorklet for audio processing
//
// 核心逻辑：
// 1. AudioWorklet 的 process() 每次回调的采样点数是动态的（128、256、512等）
// 2. 我们需要累积采样点直到凑够一帧
// 3. 凑够后立即发送，不要等待
// 4. 使用 5ms 帧大小以降低延迟（根据采样率动态计算）

class AudioProcessor extends (globalThis as any).AudioWorkletProcessor {
  private codec: string = 'pcm';
  private sampleRate: number = 0; // 从主线程接收，默认为0
  private volume: number = 1.0; // 音量，默认为1.0（100%）

  // 动态帧大小：根据采样率计算，根据编码模式调整
  // PCM模式：20ms帧（平衡延迟和性能）
  // Opus模式：40ms帧（优化性能，减少卡顿）
  private frameSize: number = 960; // 默认值，20ms @ 48kHz

  // 采样点缓冲区
  private sampleBuffer: Int16Array | null = null;
  private bufferOffset: number = 0;

  // 调试统计
  private frameCount: number = 0;
  private totalSamplesProcessed: number = 0;

  constructor() {
    super();

    // 监听来自主线程的消息（添加 port 检查）
    if (this.port) {
      this.port.onmessage = (event: MessageEvent) => {
        if (event.data.codec) {
          this.codec = event.data.codec;
        }

        if (event.data.sampleRate) {
          this.sampleRate = event.data.sampleRate;

          // 根据编码模式计算帧大小
          if (this.codec === 'opus') {
            // Opus模式：40ms帧（优化性能，减少75%的消息数量）
            this.frameSize = Math.floor(this.sampleRate * 0.04); // 40ms
          } else {
            // PCM模式：20ms帧（平衡延迟和性能，减少50%的消息数量）
            this.frameSize = Math.floor(this.sampleRate * 0.02); // 20ms
          }

          // 初始化缓冲区
          this.sampleBuffer = new Int16Array(this.frameSize);
        }

        if (event.data.volume !== undefined) {
          this.volume = event.data.volume;
        }
      };
    } else {
      console.warn('[AudioProcessor] port 为 null，无法监听主线程消息');
    }
  }

  process(
    inputList: Float32Array[][],
    _outputList: Float32Array[][],
    _parameters: Record<string, Float32Array>,
  ): boolean {
    const input = inputList[0];
    if (input.length === 0) return true;

    const inputData = input[0];

    // 安全的空值检查：如果缓冲区未初始化，跳过处理
    if (!this.sampleBuffer) {
      console.warn('[AudioProcessor] sampleBuffer 未初始化，跳过音频帧');
      return true;
    }

    const buffer = this.sampleBuffer;

    // 累积采样点到缓冲区
    for (let i = 0; i < inputData.length; i++) {
      // 应用音量
      let sample = inputData[i] * this.volume;

      // 软限幅：当音量 > 100% 时使用平滑的限幅曲线防止硬削波失真
      // 使用双曲正切函数实现软限幅，保持动态范围
      if (Math.abs(sample) > 1.0) {
        // 软限幅公式：tanh(x) 在大值时趋近于 ±1，产生平滑的限幅效果
        sample = Math.tanh(sample);
      }

      // Convert float (-1.0 to 1.0) to 16-bit PCM (-32768 to 32767)
      const pcmSample = Math.max(-32768, Math.min(32767, sample * 0x7FFF));
      buffer[this.bufferOffset] = pcmSample;
      this.bufferOffset++;

      // 缓冲区满了，立即发送
      if (this.bufferOffset >= this.frameSize) {
        this.sendFrame();
        this.bufferOffset = 0; // 重置缓冲区
      }
    }

    this.totalSamplesProcessed += inputData.length;

    return true;
  }

  private sendFrame(): void {
    // 安全的空值检查
    if (!this.sampleBuffer) {
      console.warn('[AudioProcessor] sampleBuffer 为空，无法发送帧');
      return;
    }

    // 检查 port 是否可用
    if (!this.port) {
      console.warn('[AudioProcessor] port 为空，无法发送帧');
      return;
    }

    this.frameCount++;

    // PCM 模式：将缓冲区的采样点转换为 PCM
    const chunk = new ArrayBuffer(this.frameSize * 2); // 16-bit = 2 bytes per sample
    const view = new DataView(chunk);

    for (let i = 0; i < this.frameSize; i++) {
      view.setInt16(i * 2, this.sampleBuffer[i], true); // true = little-endian
    }

    // 添加错误处理
    try {
      this.port.postMessage({
        chunk: chunk,
        frameNumber: this.frameCount,
        sampleRate: this.sampleRate
      }, [chunk]);
    } catch (err) {
      console.error('[AudioProcessor] 发送帧失败:', err);
    }
  }
}

// @ts-ignore
(globalThis as any).registerProcessor('audio-processor', AudioProcessor);
