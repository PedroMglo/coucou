//! Acknowledged, field-specific Ollama model selection. Catalog reads never
//! execute a model; only the newest validated choice may update preferences.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::{chat::{Chat, ModelInfo}, settings::Settings};

const PROVIDER: &str = "ollama";
const SUPERSEDED: &str = "Esta escolha foi substituída por uma seleção mais recente.";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionAck {
    pub provider: &'static str,
    pub model: String,
    pub selection_id: String,
    pub generation: u64,
}

#[derive(Default)]
struct Selections {
    // This mutex also serializes registering a newer intent with the final
    // persistence/publication of the current one. Never hold it across await.
    generation: Mutex<u64>,
}

struct Selection {
    generation: u64,
    model: String,
    selection_id: String,
    url: String,
}

static SELECTIONS: Selections = Selections { generation: Mutex::new(0) };

fn selection_event(settings:&Settings,generation:u64)->Result<serde_json::Value,String>{
    let mut event=serde_json::to_value(settings).map_err(|_|"Não foi possível publicar a seleção do modelo.".to_string())?;
    event.as_object_mut().ok_or("A seleção não tem um formato válido.")?
        .insert("ollamaSelectionGeneration".into(),generation.into());
    Ok(event)
}

fn validate_model(model: &str, models: &[ModelInfo], profiles: Option<&crate::pet_config::Configuration>) -> Result<(), String> {
    let available = |id: &str| !id.is_empty() && !id.ends_with(":cloud") && !id.ends_with("-cloud")
        && models.iter().any(|model| model.id == id);
    if !model.starts_with('@') {
        return if available(model) { Ok(()) } else {
            Err("Este modelo já não está disponível localmente no Ollama. Atualiza a lista antes de escolher.".into())
        };
    }
    if !["@default", "@auto", "@fast", "@reasoning"].contains(&model) {
        return Err("Este modo de modelo não existe no perfil Ollama.".into());
    }
    let pet = profiles.and_then(|config| config.pets.get("ai_ollama"))
        .filter(|pet| pet.provider == PROVIDER).ok_or("Não foi possível validar o perfil Ollama.")?;
    let mode = if model == "@default" { pet.mode.as_str() } else { &model[1..] };
    let valid = match mode {
        "fast" => available(&pet.fast.model),
        "reasoning" => available(&pet.reasoning.model),
        "auto" => available(&pet.fast.model) && available(&pet.reasoning.model),
        _ => false,
    };
    if valid { Ok(()) } else {
        Err("O perfil escolhido refere modelos locais indisponíveis. Revê os modelos rápido e de raciocínio deste pet.".into())
    }
}

impl Selections {
    fn begin(&self, settings: &Mutex<Settings>, model: String, selection_id: String)
        -> Result<(Selection, Settings), String>
    {
        if model.is_empty() || model.len() > 512 || model.trim() != model
            || model.chars().any(char::is_control)
            || selection_id.is_empty() || selection_id.len() > 128
            || selection_id.trim() != selection_id || selection_id.chars().any(char::is_control)
        {
            return Err("A seleção do modelo é inválida.".into());
        }
        let mut generation = self.generation.lock().unwrap();
        let snapshot = settings.lock().unwrap().clone();
        *generation = generation.checked_add(1).ok_or("Não foi possível registar a seleção do modelo.")?;
        Ok((Selection {
            generation: *generation, model, selection_id, url: snapshot.ollama_url.clone(),
        }, snapshot))
    }

    fn finish(
        &self,
        choice: Selection,
        settings: &Mutex<Settings>,
        chat: &Chat,
        models: &[ModelInfo],
        profiles: Option<&crate::pet_config::Configuration>,
        persist: impl FnOnce(&Settings) -> Result<(), String>,
        publish: impl FnOnce(&Settings),
    ) -> Result<SelectionAck, String> {
        let generation = self.generation.lock().unwrap();
        if *generation != choice.generation { return Err(SUPERSEDED.into()); }
        validate_model(&choice.model, models, profiles)?;
        let mut current = settings.lock().unwrap();
        if current.ollama_url != choice.url {
            return Err("A ligação ao Ollama mudou durante a seleção. Atualiza a lista antes de escolher.".into());
        }
        // The reservation excludes text, voice and maintenance during the
        // actual commit. Metadata lookup does not hold a pet busy. Keep this
        // guard in the worker until publication, even if its caller goes away.
        let reservation = chat.reserve_conversation(PROVIDER, None)?;
        reservation.channel().check_admission(PROVIDER)?;
        let mut next = current.clone();
        next.chat_models.insert(PROVIDER.into(), choice.model.clone());
        persist(&next)?;
        *current = next;
        // Emit synchronously before releasing the serialization locks. An
        // accepted disk write must not be reported as a failed model choice
        // merely because an event listener/window has since disappeared.
        publish(&current);
        Ok(SelectionAck {
            provider: PROVIDER, model: choice.model, selection_id: choice.selection_id,
            generation: choice.generation,
        })
    }
}

/// Only changes chatModels.ollama. Does not reset/open a conversation, load a
/// model, run inference, or replace unrelated settings from a webview snapshot.
#[tauri::command]
pub async fn pet_model_select(
    window: tauri::WebviewWindow,
    shared: tauri::State<'_, crate::Shared>,
    chat: tauri::State<'_, Chat>,
    model: String,
    selection_id: String,
) -> Result<SelectionAck, String> {
    if ![
        crate::island::WINDOW_LABEL, crate::HUB_LABEL, crate::pet_panel::LABEL,
        crate::pet_control::LABEL, crate::pet_chat::LABEL,
    ].contains(&window.label()) {
        return Err("Abre o pet ou a gestão dos modelos para fazer esta seleção.".into());
    }
    let (choice, snapshot) = SELECTIONS.begin(&shared.settings, model, selection_id)?;
    let models = crate::chat::models(&snapshot, PROVIDER).await?;
    let app = window.app_handle().clone();
    let chat = chat.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let shared = app.state::<crate::Shared>();
        let profiles = if choice.model.starts_with('@') { Some(crate::pet_config::read()?) } else { None };
        let generation=choice.generation;
        SELECTIONS.finish(choice, &shared.settings, &chat, &models, profiles.as_ref(),
            |next| crate::settings::save(next).map_err(|_| "Não foi possível guardar o modelo. A seleção anterior foi preservada.".into()),
            |next| { match selection_event(next,generation){
                Ok(event)=>{let _=app.emit("settings-changed",event);},
                Err(error)=>crate::log::line(error),
            } },
        )
    }).await.map_err(|_| "A seleção do modelo parou inesperadamente. Confirma o modelo atual antes de enviar.".to_string())?
}

#[cfg(test)]
#[path = "pet_model_tests.rs"]
mod tests;
