export interface ComputeRuntime {
    execute(
        workload: string,
        payload: unknown
    ): Promise<unknown>;
}