export interface InventoryObservation<T>{source:string;path:string|null;sha256:string|null;available:boolean;value:T|null;error:'unavailable'|'invalidData'|'limitExceeded'|null}
export interface SystemHostInventory{
 schemaVersion:1;kind:'systemHostInventory';petId:string;grantRevision:string;host:import('./bridge').RuntimeHealthHost;
 sampleStartedAt:number;observedAt:number;sampleDurationMs:number;
 distribution:InventoryObservation<{id:string;versionId:string|null;prettyName:string|null}>;
 kernelType:InventoryObservation<string>;kernelRelease:InventoryObservation<string>;
 distributionManagement:'externalObserved';kernelManagement:'dynamicObserved';desiredState:'unknownNotCompared';trust:'untrustedObservedData';
 maxSourceBytes:8451;maxResultBytes:8192;inferenceUsed:false;limitations:string[];
}
