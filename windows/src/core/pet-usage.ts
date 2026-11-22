/** Counts measured by the provider; null means unavailable, never zero usage. */
export interface PetUsage {
 inputTokens:number|null;outputTokens:number|null;totalTokens:number|null;
 durationMs:number|null;measured:boolean;scope:string;model?:string;updatedAt?:number;
}
