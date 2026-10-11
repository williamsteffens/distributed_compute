import { WebGpuRuntime } from "./runtime/webgpu";
import type { ComputeRuntime } from "./runtime/runtime";
import type {
    JobResult,
    WorkUnit,
    WorkerRegistration,
} from "./protocol";

type WorkerMessage =
    | {
        type: "init";
    };

const SERVER_URL = "http://localhost:42069";
const workerId = crypto.randomUUID();

let runtime: ComputeRuntime | undefined;
let running = false;

self.onmessage = async (
    event: MessageEvent<WorkerMessage>,
) => {
    try {
        switch (event.data.type) {
            case "init":
                await initialize();
                break;
        }
    } catch (error) {
        postError(error);
    }
};

async function initialize(): Promise<void> {
    if (running) {
        return;
    }

    running = true;

    try {
        runtime = await WebGpuRuntime.create();
        await register();
        
        self.postMessage({
            type: "ready",
        });
        
        
        setInterval(() => {
            void heartbeat().catch(postError);
        }, 5_000);
        
        await run();
    } catch (error) {
        running = false;
        throw error;
    }
}

async function register(): Promise<void> {
    const registration: WorkerRegistration = {
        worker_id: workerId,
        hostname: "browser",
    };

    const response = await fetch(
        `${SERVER_URL}/workers/register`,
        {
            method: "POST",
            headers: {
                "content-type": "application/json",
            },
            body: JSON.stringify(registration),
        },
    );

    if (!response.ok) {
        throw new Error(
            `Worker registration failed: ${response.status}`,
        );
    }
}

async function heartbeat(): Promise<void> {
    const response = await fetch(
        `${SERVER_URL}/workers/${workerId}/heartbeat`,
        {
            method: "POST",
        },
    );

    if (!response.ok) {
        throw new Error(
            `Heartbeat failed: ${response.status}`,
        );
    }
}

async function getWork(): Promise<WorkUnit | null> {
    const response = await fetch(
        `${SERVER_URL}/workers/${workerId}/work`,
    );

    if (!response.ok) {
        throw new Error(
            `Work request failed: ${response.status}`,
        );
    }

    return response.json();
}

async function submitResult(
    result: JobResult,
): Promise<void> {
    const response = await fetch(
        `${SERVER_URL}/results`,
        {
            method: "POST",
            headers: {
                "content-type": "application/json",
            },
            body: JSON.stringify(result),
        },
    );

    if (!response.ok) {
        throw new Error(
            `Result submission failed: ${response.status}`,
        );
    }
}

async function run(): Promise<void> {
    if (!runtime) {
        throw new Error(
            "Runtime is not initialized",
        );
    }

    while (true) {
        const work = await getWork();

        if (!work) {
            await sleep(1_000);
            continue;
        }

        try {
            const result = await runtime.execute(
                work.workload,
                work.payload,
            );

            await submitResult({
                job_id: work.job_id,
                success: true,
                result,
            });
        } catch (error) {
            await submitResult({
                job_id: work.job_id,
                success: false,
                result: {
                    error: String(error),
                },
            });
        }
    }
}

function sleep(
    milliseconds: number,
): Promise<void> {
    return new Promise((resolve) => {
        setTimeout(resolve, milliseconds);
    });
}

function postError(error: unknown): void {
    const message =
        error instanceof Error
            ? error.message
            : String(error);

    self.postMessage({
        type: "error",
        message,
    });
}
