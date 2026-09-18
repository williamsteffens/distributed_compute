import { Workload } from "../protocol";
import type { ComputeRuntime } from "./runtime";

const WORKGROUP_SIZE = 256;

export class WebGpuRuntime implements ComputeRuntime {
    readonly kind = "webgpu" as const;

    private readonly device: GPUDevice;
    private readonly queue: GPUQueue;

    private constructor(device: GPUDevice) {
        this.device = device;
        this.queue = device.queue;
    }

    static async create(): Promise<WebGpuRuntime> {
        if (!navigator.gpu) {
            throw new Error("WebGPU is not available");
        }

        const adapter = await navigator.gpu.requestAdapter({
            powerPreference: "high-performance",
        });

        if (!adapter) {
            throw new Error("No WebGPU adapter available");
        }

        const device = await adapter.requestDevice();

        return new WebGpuRuntime(device);
    }

    async execute(workload: Workload, payload: unknown): Promise<unknown> {
        switch (workload) {
            case Workload.VectorScale:
                return this.executeDouble(payload);

            case Workload.VectorAdd:
                throw new Error(
                    `Unsupported workload: ${workload}`,
                );

            default:
                throw new Error(
                    `Unsupported workload: ${workload}`,
                );
        }
    }

    private async executeDouble(payload: unknown): Promise<unknown> {
        const input = parseFloatArray(payload);

        if (input.length === 0) {
            return [];
        }

        const shader = `
      @group(0) @binding(0)
      var<storage, read> input: array<f32>;

      @group(0) @binding(1)
      var<storage, read_write> output: array<f32>;

      @compute @workgroup_size(256)
      fn main(@builtin(global_invocation_id) id: vec3<u32>) {
          let i = id.x;

          if (i < arrayLength(&output)) {
              output[i] = input[i] * 2.0;
          }
      }
    `;

        const module = this.device.createShaderModule({
            label: "double",
            code: shader,
        });

        const pipeline = this.device.createComputePipeline({
            label: "double",
            layout: "auto",
            compute: {
                module,
                entryPoint: "main",
            },
        });

        const inputBuffer = this.createStorageBuffer(
            "input",
            input,
            GPUBufferUsage.STORAGE,
        );

        const outputBuffer = this.device.createBuffer({
            label: "output",
            size: input.byteLength,
            usage:
                GPUBufferUsage.STORAGE |
                GPUBufferUsage.COPY_SRC,
        });

        const readbackBuffer = this.device.createBuffer({
            label: "readback",
            size: input.byteLength,
            usage:
                GPUBufferUsage.COPY_DST |
                GPUBufferUsage.MAP_READ,
        });

        this.queue.writeBuffer(
            inputBuffer,
            0,
            input.buffer,
            input.byteOffset,
            input.byteLength,
        );

        const bindGroup = this.device.createBindGroup({
            layout: pipeline.getBindGroupLayout(0),
            entries: [
                {
                    binding: 0,
                    resource: {
                        buffer: inputBuffer,
                    },
                },
                {
                    binding: 1,
                    resource: {
                        buffer: outputBuffer,
                    },
                },
            ],
        });

        const encoder = this.device.createCommandEncoder();

        const pass = encoder.beginComputePass();

        pass.setPipeline(pipeline);
        pass.setBindGroup(0, bindGroup);

        const workgroups = Math.ceil(
            input.length / WORKGROUP_SIZE,
        );

        pass.dispatchWorkgroups(workgroups);
        pass.end();

        encoder.copyBufferToBuffer(
            outputBuffer,
            0,
            readbackBuffer,
            0,
            input.byteLength,
        );

        this.queue.submit([encoder.finish()]);

        await readbackBuffer.mapAsync(
            GPUMapMode.READ,
        );

        const mapped = readbackBuffer.getMappedRange();

        const result = new Float32Array(
            mapped.slice(0),
        );

        readbackBuffer.unmap();

        inputBuffer.destroy();
        outputBuffer.destroy();
        readbackBuffer.destroy();

        return Array.from(result);
    }

    private createStorageBuffer(
        label: string,
        data: Float32Array,
        usage: GPUBufferUsageFlags,
    ): GPUBuffer {
        const buffer = this.device.createBuffer({
            label,
            size: data.byteLength,
            usage:
                usage |
                GPUBufferUsage.COPY_DST,
        });

        this.queue.writeBuffer(
            buffer,
            0,
            data.buffer,
            data.byteOffset,
            data.byteLength,
        );

        return buffer;
    }
}

function parseFloatArray(payload: unknown): Float32Array {
    if (!Array.isArray(payload)) {
        throw new Error("Expected payload to be an array");
    }

    return new Float32Array(
        payload.map((value) => {
            if (typeof value !== "number") {
                throw new Error("Expected numeric input");
            }

            return value;
        }),
    );
}