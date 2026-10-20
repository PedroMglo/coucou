import {PetPreferences,petName} from "../mochi/pet-preferences";
import {isCliProvider,providerDef, providerForPill } from "../core/providers";
import {delegatedRequest} from "../mochi/delegation";
import {PILL_CATALOG,pillDefinition} from "../core/pills";
import type {ManagedInventory,ManagedUsage} from '../mochi/management';
import {QuotaNotices,quotaForPet} from '../mochi/quota-notices';
import {OllamaModelSelection} from '../core/ollama-selection';
import {refreshRequestDestinations,requestDestinations} from '../core/request-destinations';
import {presentRequest} from './request-routing';
// Island-side adapter. The island still owns every integration and permission;
// the habitat receives a small presentation snapshot and explicit user clicks.
import { Bridge, emitToWindow, onEvent } from "../core/bridge";
import { Spotify, SPOTIFY_ID } from "../core/spotify";
import { State } from "../core/state";
import type {DroppedFile} from '../core/bridge';
import { PET_EVENTS, companionIdentities, type CompanionsSnapshot } from "../mochi/companions";
import type { Outfit } from "../mochi/wardrobe";

export class CompanionsLink {
  supported = false;
  private launched = false;
  private key = "";
  private configuration = "";
  private configurationEpoch = 0;
  private pending = false;
  private dirty = false;
  private showing = false;
  private band: CompanionsSnapshot["band"] = null;
  private presence: {running:string[];activity:Record<string,number>} = {running:[],activity:{}};
  private inventoryPending=false;
  private quotaUsage:ManagedUsage|undefined;
  private quotaNotices=new QuotaNotices();

  private open: (id: string, view?: string, manual?: boolean) => void;
  private outfit: () => Outfit;

  constructor(open: (id: string, view?: string, manual?: boolean) => void, outfit: () => Outfit) {
    this.open = open;
    this.outfit = outfit;
  }

  async init() {
    await PetPreferences.init();
    PetPreferences.subscribe(()=>{this.key="";void this.sync();});
    // A missing surface (Windows, GNOME Wayland or a creation error) uses the
    // original single desktop companion. Nothing disappears without a fallback.
    this.supported = await Bridge.companionsConfigure(false, 0) != null;
    if (!this.supported) return;
    const inventory=async()=>{
      if(this.inventoryPending||(!State.settings.companionMode&&!State.petCardsOnly))return;this.inventoryPending=true;
      try{
        const [value,tasks]=await Promise.all([Bridge.petManagement<ManagedInventory>('list'),Bridge.assistantTasks()]);
        if(!value?.sessions)return;
        this.quotaUsage=(value as ManagedInventory&{usage?:ManagedUsage}).usage;
        State.petSessions.merge(value.sessions,tasks??[],pet=>{
          const def=pillDefinition(pet);if(!def||!State.isKept(pet))return;
          State.upsertExternalAgent(pet,petName(pet,def.name),def.color);return State.tasks.find(t=>t.id===pet);
        });
        for(const t of State.tasks)State.petSessions.project(t,State.pendingApproval);
        State.notify();
      }catch{/* Hooks keep their exact origin while the metadata service is unavailable. */}finally{this.inventoryPending=false;}
    };
    void inventory();window.setInterval(()=>void inventory(),15000);
    await onEvent<{pet:string;backend:string;id:string;cwd?:string;title?:string;owned?:boolean;view?:string}>('pet-session-select',value=>{
      if(State.pendingApproval?.pillId===value.pet){this.open(value.pet,State.pendingApproval.questions?'question':'approval');return;}
      const def=pillDefinition(value.pet);if(!def||!State.isKept(value.pet)||typeof value.id!=='string'||value.id.length>512)return;
      State.upsertExternalAgent(def.id,petName(def.id,def.name),def.color);
      if(!State.petSessions.entries.has(JSON.stringify([value.pet,value.backend,value.id]))){
        const task=State.tasks.find(t=>t.id===def.id)!;
        const s=State.petSessions.observe(task,value.backend,value.id,value.cwd??'',!!value.owned);s.sessionTitle=value.title??'';s.hookAt=undefined;
      }
      if(State.petSessions.choose(value.pet,value.backend,value.id)){State.projectSessions(value.pet);this.open(value.pet,value.view==='finished'?'finished':'overview',true);}
    });
    await onEvent<{pillId:string;view?:string}>(PET_EVENTS.summon,async value=>{
      const def=pillDefinition(value.pillId);
      if((!State.settings.companionMode&&!State.petCardsOnly) || !def || !State.isKept(def.id))return;
      const pending=State.petCardsOnly&&value.view!=='upload'?requestDestinations.forPet(def.id):null;
      if(pending){void presentRequest(pending);return;}
      State.upsertExternalAgent(def.id,petName(def.id,def.name),def.color);State.notify();
      const task=State.tasks.find(t=>t.id===def.id);
      if(State.petCardsOnly&&value.view!=='upload'&&task&&['approval','question'].includes(task.state)&&State.pendingApproval?.pillId!==def.id){
        await refreshRequestDestinations();
        const destination=requestDestinations.forPet(def.id);
        if(destination)void presentRequest(destination);
        else if(task.decisionInTerminal&&task.sessionId&&task.sessionBackend)void Bridge.petFocusSession(task.sessionBackend,task.sessionId).catch(()=>Bridge.petPanelOpen(360,50,def.id,'sessions'));
        else void Bridge.petPanelOpen(360,50,def.id,'sessions');
        return;
      }
      this.open(def.id,['upload','prompt','overview'].includes(value.view??'')?value.view:undefined,true);
      // Wake the actual pet as well as its card. A dormant catalog entry has
      // no position until the new snapshot reaches the habitat.
      if(State.settings.companionMode)void emitToWindow('companions','companions-present',{id:def.id,view:value.view??'overview',manual:true});
    });
    await onEvent<{pet:string;file:DroppedFile}>('pet-document',async value=>{
      const def=pillDefinition(value.pet),provider=providerForPill(value.pet);
      if((!State.settings.companionMode&&!State.petCardsOnly) || !def || !provider || !value.file?.path || !value.file.name)return;
      if(value.file.conversationId){const current=await Bridge.chatConversation(provider);if(current.id!==value.file.conversationId)return;}
      const previous=State.fileFor(provider);if(previous?.sequence!==undefined&&value.file.sequence!==undefined&&previous.sequence>value.file.sequence)return;
      State.setFileContext(provider,value.file);
      State.chatFeedback.set(provider,`${value.file.name} recebido por ${petName(def.id,def.name)}.`);
      State.upsertExternalAgent(def.id,petName(def.id,def.name),def.color);
      if(State.pendingApproval?.pillId!==value.pet){
        this.open(def.id,'choose',true);
        // Replace any older summon still waiting for this pet to appear, and
        // anchor the document actions to its actual current position.
        if(State.settings.companionMode)void emitToWindow('companions','companions-present',{id:def.id,view:'choose',manual:true});
      }
      State.notify();
    });
    await onEvent<{provider:string}>("pet-use-profile",async value=>{
      if(!isCliProvider(value.provider) && value.provider!=="ollama")return;
      if(value.provider==='ollama'){
        try{await OllamaModelSelection.select('@default');}
        catch(error){State.chatFeedback.set('ollama',String(error).replace(/^Error:\s*/,''));State.notify();}
        return;
      }
      State.settings={...State.settings,chatModels:{...State.settings.chatModels,[providerDef(value.provider).id]:"@default"}};void Bridge.saveSettings(State.settings).catch(Bridge.reportSettingsError);State.notify();
    });
    await onEvent(PET_EVENTS.ready, () => { this.key = ""; void this.sync(); });
    await onEvent<{ pillId: string; view?: string; manual?:boolean }>(PET_EVENTS.open, e => {
      if ((!State.settings.companionMode&&!State.petCardsOnly) || !State.tasks.some(t => t.id === e.pillId)) return;
      this.open(e.pillId, e.view,e.manual===true);
    });
    await onEvent<{ action: string }>(PET_EVENTS.action, e => {
      if (e.action !== "pause" || !State.settings.companionMode) return;
      State.paused = !State.paused;
      State.notify();
    });
    await onEvent<{pet:string}>(PET_EVENTS.delegate,e=>{
      if(!State.settings.companionMode || !State.tasks.some(t=>t.id===e.pet))return;
      const draft=delegatedRequest(e.pet),codex=pillDefinition("agent_codex");if(!draft || !codex)return;
      State.preparedChat={provider:"codex",...draft};State.upsertExternalAgent(codex.id,codex.name,codex.color);
      this.open(codex.id,"prompt",true);State.notify();
    });
    await onEvent("companions-layout-changed", () => this.invalidateConfiguration());
    await onEvent("screen-changed", () => this.invalidateConfiguration());
    await onEvent<typeof this.presence>("companions-presence", value => {
      const closed=this.presence.running.filter(id=>!value.running.includes(id));
      this.presence=value;
      for(const id of closed) {
        const task=State.tasks.find(t=>t.id===id);
        // A crashed/closed client may never send SessionEnd. Another live
        // client, an active native conversation or a real decision wins.
        if(!task || State.chatBusyProviders.has(providerForPill(id) ?? "") || State.pendingApproval?.pillId===id)continue;
        task.sessionId=null;task.sessionCwd=null;State.removeTask(id);
      }
      void this.sync();
    });
    this.presence = await Bridge.companionsPresence() ?? this.presence;
    State.subscribe(() => void this.sync());
  }

  async launch() { this.launched = true; await this.sync(); }
  private get enabled() { return this.supported && this.launched && State.settings.companionMode; }
  get active() { return this.enabled && this.showing; }

  private invalidateConfiguration() {
    this.configurationEpoch++;
    this.configuration = "";
    void this.sync();
  }

  async sync() {
    if (!this.supported || !this.launched) return;
    if (this.pending) { this.dirty = true; return; }
    this.pending = true;
    try {
      do {
        this.dirty = false;
        const now=Date.now();
        const configuredPets=new Set(PILL_CATALOG.filter(p=>State.isKept(p.id)).map(p=>p.id));
        const quotaUsage:ManagedUsage={providers:{...this.quotaUsage?.providers}};
        for(const [provider,plan]of Object.entries({codex:State.codexPlanUsage,claude:State.planUsage})){
          if(!plan)continue;
          const cached=quotaUsage.providers[provider];
          const cachedAt=typeof cached?.updated==='number'&&Number.isFinite(cached.updated)&&cached.updated>0&&cached.updated*1000<=now?cached.updated*1000:0;
          const knownBlock=cached?.blocked===true&&cached.stale!==true&&now-cachedAt<=300000;
          if(knownBlock||plan.updatedAt<=cachedAt)continue;
          const windows:{label:string;used:number;reset_at:number}[]=[];
          if(plan.fiveHour)windows.push({label:'5 h',used:plan.fiveHour.usedPct,reset_at:plan.fiveHour.resetsAt/1000});
          if(plan.sevenDay)windows.push({label:'7 dias',used:plan.sevenDay.usedPct,reset_at:plan.sevenDay.resetsAt/1000});
          quotaUsage.providers[provider]={updated:plan.updatedAt/1000,windows};
        }
        const quotas={usage:quotaUsage,configuredPets,now};
        const snapshot: CompanionsSnapshot = {
          control:{enabled:State.settings.petControlBar,systemCenter:State.settings.petSystemCenter,autoCloseInterval:State.settings.autoCloseInterval,openOnHover:State.settings.openOnHover,
            tasks:State.tasks.filter(t=>State.isKept(t.id)).map(t=>({...t,steps:t.steps.slice(-64).map(s=>s.slice(0,2048)),stepIndex:Math.max(0,Math.min(63,t.stepIndex))})),
            focusId:State.focusId,integrations:State.integrations,spotify:Spotify.state},
          enabled: this.enabled, paused: State.paused, motion: State.settings.companionMotion,
          language: State.settings.language, presented: State.mode === "expanded" ? State.focusId : null,
          pets: companionIdentities(State.tasks.filter(t=>State.isKept(t.id)||t.sessionId||State.pendingApproval?.pillId===t.id||State.chatBusyProviders.has(providerForPill(t.id)??'')).map(t => State.chatBusyProviders.has(providerForPill(t.id) ?? "") && !["approval","question","ratelimit","error"].includes(t.state) ? {...t,state:"thinking" as const} : t), State.pendingApproval, State.spotifyPlaying, this.outfit(), new Set([...this.presence.running,...(Spotify.state.running ? [SPOTIFY_ID] : [])]), new Map([...Object.entries(this.presence.activity).map(([id,seq])=>[id,String(seq)] as [string,string]),[SPOTIFY_ID, Spotify.state.track?.id ?? ""]]),quotas),
        };
        const configuration = JSON.stringify([snapshot.enabled, snapshot.pets.length, State.settings.screen, State.settings.companionTopClearance, State.settings.companionBarMode, State.settings.petControlBar,State.settings.petSystemCenter]);
        if (configuration !== this.configuration) {
          const epoch = this.configurationEpoch;
          await Bridge.petControlConfigure();
          if (epoch !== this.configurationEpoch) { this.dirty = true; continue; }
          const configured = await Bridge.companionsConfigure(snapshot.enabled, snapshot.pets.length);
          // A display/bar event can arrive while GTK is placing the previous
          // surface. Only its own generation may commit geometry or a failure.
          if (epoch !== this.configurationEpoch) { this.dirty = true; continue; }
          if (!configured) {
            // Keep the singleton fallback for this launch. Retrying later could
            // leave both renderers on screen after the singleton has flown out.
            this.supported = false;
            snapshot.enabled = false;
          } else { this.configuration = configuration; this.band = configured.band; }
        }
        snapshot.pets=snapshot.pets.map(p=>({...p,name:petName(p.id,p.name),sessions:State.petSessions.list(p.id).length}));
        snapshot.catalog=[...snapshot.pets,...PILL_CATALOG.filter(def=>State.isKept(def.id) && !snapshot.pets.some(p=>p.id===def.id)).map(def=>({id:def.id,name:petName(def.id,def.name),color:def.color,state:"idle" as const,active:false,busy:false,attention:false,dancing:false,activity:"catalog",outfit:this.outfit()}))];
        snapshot.catalog=snapshot.catalog.map(p=>{const quota=configuredPets.has(p.id)?quotaForPet(p.id,quotaUsage,now):undefined;return quota?{...p,quota}:p;});
        snapshot.band = this.band;
        this.showing = snapshot.enabled;
        const key = JSON.stringify(snapshot);
        if (key !== this.key) {
          this.key = key;
          await emitToWindow("companions", PET_EVENTS.state, snapshot);
          await emitToWindow("pet-control", PET_EVENTS.state, snapshot);
        }
        if(snapshot.enabled&&!snapshot.paused&&!State.pendingApproval){
          const targets=new Set(snapshot.pets.filter(p=>configuredPets.has(p.id)).map(p=>p.id));
          for(const notice of this.quotaNotices.observe(quotaUsage,targets,now)){
            await emitToWindow("companions","companions-notice",notice);
          }
        }
      } while (this.dirty);
    } finally { this.pending = false; }
  }
}
