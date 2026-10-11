import { Workload } from "../protocol";
import type { ComputeRuntime } from "./runtime";

import vectorAddShader from "../../../../../shaders/vector_add.wgsl?raw";
import vectorScaleShader from "../../../../../shaders/vector_scale.wgsl?raw";

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

    async execute(
        workload: Workload,
        payload: unknown
    ): Promise<unknown> {
        switch (workload) {
            case Workload.VectorScale:
                return this.executeVectorScale(payload);

            case Workload.VectorAdd:
                return this.executeVectorAdd(payload);

            default:
                throw new Error(
                    `Unsupported workload: ${workload}`,
                );
        }
    }

    private async executeVectorScale(
        payload: unknown
    ): Promise<unknown> {
        const { input, scalar } = parseVectorScalePayload(payload);

        if (input.length === 0) {
            return [];
        }

        const module = this.device.createShaderModule({
            label: "vector_scale",
            code: vectorScaleShader,
        });

        const pipeline = this.device.createComputePipeline({
            label: "vector_scale",
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

        const paramsBuffer = this.device.createBuffer({
            label: "params",
            size: 16, // uniform buffer size for a single float (4 bytes) with padding
            usage: 
                GPUBufferUsage.UNIFORM |
                GPUBufferUsage.COPY_DST,
        });

        const params = new Float32Array([scalar]);

        this.queue.writeBuffer(
            paramsBuffer,
            0,
            params.buffer,
            params.byteOffset,
            params.byteLength,
        );

        const readbackBuffer = this.device.createBuffer({
            label: "readback",
            size: input.byteLength,
            usage:
                GPUBufferUsage.COPY_DST |
                GPUBufferUsage.MAP_READ,
        });

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
                {
                    binding: 2,
                    resource: {
                        buffer: paramsBuffer,
                    },
                }
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

    private async executeVectorAdd(payload: unknown): Promise<unknown> {
        const input = parseFloatArray(payload, "input");

        if (input.length === 0) {
            return [];
        }

        const shader = vectorAddShader;

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

function parseFloatArray(
    value: unknown,
    name: string,
): Float32Array {
    if (!Array.isArray(value)) {
        throw new Error(`Expected ${name} to be an array`);
    }

    return new Float32Array(
        value.map((item) => {
            if (typeof item !== "number") {
                throw new Error(
                    `Expected numeric values in ${name}`,
                );
            }

            return item;
        }),
    );
}

function parseVectorScalePayload(
    payload: unknown,
): {
    input: Float32Array;
    scalar: number;
} {
    if (
        typeof payload !== "object" ||
        payload === null
    ) {
        throw new Error(
            "Expected VectorScale payload to be an object",
        );
    }

    const value = payload as Record<string, unknown>;

    if (typeof value.scalar !== "number") {
        throw new Error(
            "Expected scalar to be a number",
        );
    }

    return {
        input: parseFloatArray(value.input, "input"),
        scalar: value.scalar,
    };
}