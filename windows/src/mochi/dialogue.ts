import type { BotStateName } from "../core/layout";
import {sameSession,type SessionTarget} from '../core/session-chat';
/** Transient public progress, independent of approval cards and chat history. */
export interface PetActivity { pet: string; turn: number; phase: string; message: string; origin?:string;target?:SessionTarget }
export interface Dialogue extends PetActivity { changedAt: number; messageAt: number }
export const BUSY = new Set(["preparing", "transcribing", "received", "thinking", "working", "searching", "message", "speaking"]);
export const ATTENTION = new Set(["approval", "question", "error", "ratelimit"]);
export function phaseFor(activity: Dialogue | undefined, native: string): string {
  if (ATTENTION.has(native)) return native;
  return activity?.phase ?? native;
}
export function animationFor(phase: string, fallback: BotStateName): BotStateName {
  if (["preparing","transcribing","received"].includes(phase)) return "thinking";
  if (phase === "message" || phase === "speaking") return "working";
  if (["thinking", "working", "searching", "approval", "question", "error", "ratelimit"].includes(phase)) return phase as BotStateName;
  return fallback;
}
export class PetDialogues {
  readonly entries = new Map<string, Dialogue>();
  selected: string | null = null;
  update(update: PetActivity, now: number): boolean {
    const before = this.entries.get(update.pet);
    if (before && update.turn < before.turn) return false;
    if(update.target&&update.target.petId!==update.pet)return false;
    const message = update.message.trim().slice(-1200);
    const sameTurn = before?.turn === update.turn;
    if(sameTurn&&before.target&&update.target&&!sameSession(before.target,update.target))return false;
    const target=update.target??(sameTurn?before.target:undefined);
    const content=message || (sameTurn ? before.message : "");
    if(sameTurn&&before.phase===update.phase&&before.message===content&&before.origin===update.origin&&(!target&&!before.target||sameSession(target,before.target)))return false;
    // Completion starts the reading window even when the last streamed text
    // already matches the final response. Duplicate receipts cannot renew it.
    const completed=sameTurn&&before.phase!==update.phase&&["done","stopped","failed"].includes(update.phase);
    this.entries.set(update.pet, {
      ...update, message:content,target:target?Object.freeze({...target}):undefined,
      changedAt: now, messageAt:completed || message && message !== before?.message ? now : sameTurn ? before.messageAt : now,
    });
    this.selected = update.pet;
    return true;
  }
  current(pet: string, now: number): Dialogue | undefined {
    const value=this.entries.get(pet);
    return value && (!['done','stopped','failed'].includes(value.phase) || now-value.changedAt<=9000) ? value : undefined;
  }
  visible(pet: string, now: number, cardOpen: boolean): Dialogue | undefined {
    const value = this.entries.get(pet);
    if (!value || cardOpen || this.selected !== pet) return;
    if (["done", "stopped", "failed"].includes(value.phase) && now - value.changedAt > 9000) return;
    return value;
  }
}
