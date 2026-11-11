//! One finite signed System plan on the original common Driver. No inference,
//! generated plan, new human decision, replacement grant or scheduler.
use super::*;

impl Driver {
    pub(super)async fn drive_native_system_plan(&self)->model::Result<()> {
        let owner=&self.owner;let spec=&owner.live.spec;let lease=owner.lease();
        if !spec.binding_format()||!spec.native_system_requested(){return Err(model::Error::Conflict);}
        let goal=self.store.read_control(&lease.goal_id,&spec.owner.pet_id)?;
        if goal.state==model::State::Completed{return self.close_completed(&goal,0).await;}
        if !self.current().await?{return self.stopped(0).await;}
        let retained=self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained.clone();
        let retained=if let Some(retained)=retained{retained}else{
            let policy=spec.native_system_plan.as_ref().ok_or(model::Error::Conflict)?;
            if !policy.matches(&goal,spec){return Err(model::Error::Conflict);}
            let plan=&policy.steps[0].cause.plan_id;
            if self.authority()?.read_recipe(spec,&owner.live.proof,plan).map_err(|f|f.control_error())?.is_some(){
                return self.notice(Phase::RecoveryRequired,0,Some("A receita existe sem o executor original retido; os dados guardados não recriam autoridade nem repetem efeitos."));
            }
            let configuration=self.environment.configuration()?;
            let target=self.environment.system_target(&policy.steps[0].operation,&spec.owner.pet_id,&configuration,&||owner.current(crate::assistant_memory::now()))?;
            if !policy.target_matches(&target){return Err(model::Error::Conflict);}
            let review=owner.review_recipe_with_criteria(&goal,plan,"driver-system-recipe",target,&configuration,&[],crate::assistant_memory::now())?;
            let(approved,ready)=review.approve_native_system_plan(owner,&goal,&configuration,crate::assistant_memory::now())?;
            let retained=Arc::new(Retained{goal,approved,ready,registered:AtomicBool::new(false)});
            self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained=Some(retained.clone());retained
        };
        if !self.register_retained(&retained,0).await?{return Ok(());}
        // A second call may reconcile/close an existing completion, but cannot
        // re-enter execution after any possible effect. Preserve all receipts.
        if retained.ready.execution_entered(){return self.notice(Phase::RecoveryRequired,0,Some("A execução original já entrou no percurso de efeitos; reconciliar o recibo sem a repetir."));}
        loop {
            if !self.current().await?{return self.stopped(0).await;}
            self.notice(Phase::Executing,0,None)?;
            // The existing synchronous native brokers may use their own async
            // bridge (for example Sound cache invalidation after dispatch).
            // Run them outside the Driver's async context, retaining the same
            // owner/Ready until the blocking worker has actually joined. The
            // native Driver bounds this pool to one worker with a 4 MiB stack.
            let worker_owner=owner.clone();let store=self.store.clone();let recipe=retained.clone();let environment=self.environment.clone();
            let executed=tokio::task::spawn_blocking(move||worker_owner.run_recipe_for_store(&store,&recipe.goal,&recipe.ready,||environment.configuration(),|_|{})).await.map_err(|_|model::Error::StorageUnavailable)?;
            let result=match executed{
                Ok(result)=>result,
                // Only the original in-memory Ready proves no effect was
                // entered. Keep it while the shared physical resource is busy.
                Err(model::Error::Busy)if !retained.ready.execution_entered()=>{
                    self.notice(Phase::WaitingResource,0,Some("A aguardar o recurso físico original, com o mesmo plano, prazo e orçamento."))?;
                    tokio::time::sleep(Duration::from_millis(50)).await;continue;
                },
                Err(error)=>return Err(error),
            };
            if result.outcome==Outcome::GoalCompleted {
                let completed=self.store.read_control(&lease.goal_id,&spec.owner.pet_id)?;
                self.close_completed(&completed,0).await?;
                self.state.lock().map_err(|_|model::Error::StorageUnavailable)?.retained=None;return Ok(());
            }
            if (owner.live.closed.load(Ordering::Acquire)||owner.live.hold.stopped())&&owner.workers()==0{return self.stopped(0).await;}
            return self.notice(Phase::RecoveryRequired,0,Some("O plano finito terminou sem conclusão verificada; conservar resultados, limites e efeitos incertos, sem repetir ou replanear."));
        }
    }
}
