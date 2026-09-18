import { Workload } from "../protocol";

export interface ComputeRuntime {
    execute(
        workload: Workload,
        payload: unknown
    ): Promise<unknown>;
}