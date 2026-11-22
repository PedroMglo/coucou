/** Native bounded metrics only; never model output or an execution permit. */
type Observation<T>={source:string;available:boolean;value:T|null;error:'unavailable'|'invalidData'|'limitExceeded'|null};
export interface SystemHealthSnapshot{
 schemaVersion:1;petId:string;policyRevision:string;management:'dynamicObserved';
 sampleStartedAt:number;observedAt:number;sampleDurationMs:number;
 memory:Observation<{totalBytes:number;availableBytes:number}>;
 cpu:Observation<{totalTicks:number;idleTicks:number}>;
 uptime:Observation<{seconds:number}>;
 findings:{id:string;severity:string;confidence:string;status:string;source:string;thresholdPercent:number;availableBytes:number;totalBytes:number;recommendation:string}[];
 maxReadBytes:number;maxResultBytes:number;inferenceUsed:false;limitations:string[];
}

/** Two original bounded samples; null metrics are indeterminate, never zero. */
export interface SystemHealthBaseline{
 schemaVersion:1;kind:'systemHostHealthBaseline';before:import('./bridge').RuntimeHealthEvidence;after:import('./bridge').RuntimeHealthEvidence;
 intervalMs:number;elapsedMs:number;cpuBusyBasisPoints:number|null;memoryAvailableDeltaBytes:number|null;maxMetricReadBytes:17670;inferenceUsed:false;
}
